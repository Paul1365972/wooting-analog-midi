pub mod config;
pub mod note;

use anyhow::{anyhow, bail, Context, Result};
use config::{Config, KeyConfig};
use log::{info, trace};
use midir::{MidiOutput, MidiOutputConnection, MidiOutputPort};
use note::{NoteSink, MIDI_NOTE_MAX, MIDI_NOTE_MIN};
use rustc_hash::FxHashMap;
use sdk::SDKResult;
pub use sdk::{DeviceInfo, FromPrimitive, HIDCodes, ToPrimitive, WootingAnalogResult};
use std::collections::HashMap;
use std::time::Instant;
use wooting_analog_wrapper as sdk;

pub const REFRESH_RATE: f32 = 200.0; //Hz
const AFTERTOUCH: bool = true;

const MIDI_CLIENT_NAME: &str = "Wooting Analog MIDI Output";
const MIDI_PORT_NAME: &str = "wooting-analog-midi";

const DEVICE_BUFFER_MAX: usize = 5;
const ANALOG_BUFFER_READ_MAX: usize = 40;

pub type NoteID = u8;
pub type Channel = u8;

#[derive(Debug)]
struct KeyState {
    pressed: bool,
    shifted_amount: i8,
    velocity: f32,
    current_value: f32,
    lower_press: Option<(Instant, f32)>,
}

impl KeyState {
    fn new() -> Self {
        Self {
            pressed: false,
            shifted_amount: 0,
            velocity: 0.0,
            current_value: 0.0,
            lower_press: None,
        }
    }

    fn update_value(
        &mut self,
        key_config: &KeyConfig,
        new_value: f32,
        sink: &mut impl NoteSink,
        shifted_amount: i8,
    ) -> Result<()> {
        if (self.current_value <= key_config.actuation_point
            && new_value > key_config.actuation_point
            && new_value < key_config.threshold)
            || new_value <= key_config.actuation_point
        {
            self.lower_press = Some((Instant::now(), new_value));
            self.velocity = 0.0;
        } else if let Some((prev_time, prev_depth)) = self.lower_press {
            let duration = prev_time.elapsed().as_secs_f32();
            self.velocity = if new_value != prev_depth {
                ((new_value - prev_depth) / duration * key_config.velocity_scale / 100.0)
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };
            if (prev_depth - new_value).abs() < 0.01 || new_value < self.current_value - 0.01 {
                self.lower_press = Some((Instant::now(), new_value));
            }
        }

        if shifted_amount != self.shifted_amount && !self.pressed {
            self.shifted_amount = shifted_amount;
        }

        if let Some(effective_note) = self.get_effective_note(key_config.note_id) {
            if new_value > key_config.threshold {
                if !self.pressed {
                    info!(
                        "Triggering with velocity {:.3}, prev {:?}, new_val {:?}, elapsed {:?}",
                        self.velocity,
                        self.lower_press,
                        new_value,
                        self.lower_press.unwrap().0.elapsed()
                    );
                    sink.note_on(effective_note, self.velocity, key_config.channel)?;
                    self.pressed = true;
                } else if AFTERTOUCH && new_value != self.current_value {
                    sink.polyphonic_aftertouch(effective_note, new_value, key_config.channel)?;
                }
            } else if self.pressed {
                sink.note_off(effective_note, self.velocity, key_config.channel)?;
                self.pressed = false;
            }
        }

        self.current_value = new_value;
        Ok(())
    }

    fn get_effective_note(&self, base_note: NoteID) -> Option<NoteID> {
        let computed = base_note as i16 + self.shifted_amount as i16;
        if computed >= MIDI_NOTE_MIN.into() && computed <= MIDI_NOTE_MAX.into() {
            Some(computed as NoteID)
        } else {
            None
        }
    }
}

pub struct MidiService {
    port_options: Vec<PortOption>,
    connection: Option<MidiOutputConnection>,
    config: Config,
    key_states: FxHashMap<HIDCodes, KeyState>,
    enabled: bool,
    enabled_key_state: bool,
}

pub struct PortOption {
    port: MidiOutputPort,
    name: String,
}

impl MidiService {
    pub fn new() -> Self {
        MidiService {
            port_options: Vec::new(),
            connection: None,
            config: Config::default(),
            key_states: FxHashMap::default(),
            enabled: false,
            enabled_key_state: false,
        }
    }

    pub fn set_config(&mut self, config: Config) -> Result<()> {
        // Clean up existing notes if needed
        if let Some(sink) = &mut self.connection {
            for (hid_code, state) in &mut self.key_states {
                if state.pressed {
                    if let Some(key_config) = self.config.key_configs.get(hid_code) {
                        if let Some(effective_note) = state.get_effective_note(key_config.note_id) {
                            sink.note_off(effective_note, state.velocity, key_config.channel)?;
                        }
                    }
                }
            }
        }

        self.config = config;
        self.key_states.clear();

        // Initialize states for all configured keys
        for hid_code in self.config.key_configs.keys() {
            self.key_states.insert(hid_code.clone(), KeyState::new());
        }

        Ok(())
    }

    pub fn poll(&mut self) -> Result<()> {
        let connection = self
            .connection
            .as_mut()
            .ok_or_else(|| anyhow!("No MIDI connection!"))?;

        let read_result: SDKResult<HashMap<u16, f32>> =
            sdk::read_full_buffer(ANALOG_BUFFER_READ_MAX);
        let analog_data = read_result.0.context("Failed to read buffer")?;

        let toggle_pressed = self.config.toggle_keys.iter().any(|code| {
            analog_data
                .get(&code.to_u16().unwrap())
                .map_or(false, |&v| v > 0.0)
        });
        if toggle_pressed != self.enabled_key_state {
            self.enabled_key_state = toggle_pressed;
            if toggle_pressed {
                self.enabled = !self.enabled;
                if self.enabled {
                    info!("Enabled keyboard");
                } else {
                    info!("Disabled keyboard");
                }
            }
        }
        if !self.enabled {
            return Ok(());
        }

        let modifier_pressed = self.config.modifier_keys.iter().any(|code| {
            analog_data
                .get(&code.to_u16().unwrap())
                .map_or(false, |&v| v > 0.0)
        });

        for (hid_code, state) in &mut self.key_states {
            if let Some(key_config) = self.config.key_configs.get(hid_code) {
                let new_value = analog_data
                    .get(&hid_code.to_u16().unwrap())
                    .copied()
                    .unwrap_or(0.0);

                let shifted_amount = modifier_pressed as i8 * key_config.shift_amount;

                state.update_value(key_config, new_value, connection, shifted_amount)?;
            }
        }

        Ok(())
    }

    pub fn init(&mut self) -> Result<u32> {
        info!("Starting Wooting Analog SDK!");
        let init_result: SDKResult<u32> = sdk::initialise();
        let device_num = init_result
            .0
            .context("Wooting Analog SDK Failed to initialise")?;

        info!("Analog SDK Successfully initialised with {device_num} devices");
        let devices = sdk::get_connected_devices_info(DEVICE_BUFFER_MAX).0?;
        for (i, device) in devices.iter().enumerate() {
            info!("Device {} is {:?}", i, device);
        }

        self.refresh_port_options();

        if !self.port_options.is_empty() {
            info!("Opening connection");
            self.select_port(0)?;
        } else {
            info!("No output ports available!");
        }

        Ok(device_num)
    }

    pub fn refresh_port_options(&mut self) {
        let midi_output = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();
        self.port_options = midi_output
            .ports()
            .into_iter()
            .map(|port| {
                let name = midi_output.port_name(&port).unwrap();
                PortOption { port, name }
            })
            .collect();
        info!(
            "We have {} ports available! ({:?})",
            self.port_options.len(),
            self.port_options
                .iter()
                .map(|port| &port.name)
                .collect::<Vec<_>>()
        );
    }

    pub fn select_port(&mut self, option: usize) -> Result<()> {
        if option >= self.port_options.len() {
            bail!("Port option out of range!");
        }

        drop(self.connection.take());

        let selection = &self.port_options[option];
        info!("Connecting to Port {}: \"{}\"!", option, selection.name);

        let midi_output = MidiOutput::new(MIDI_CLIENT_NAME).unwrap();
        self.connection = Some(
            midi_output
                .connect(&selection.port, MIDI_PORT_NAME)
                .map_err(|e| anyhow!("Error: {}", e))?,
        );

        Ok(())
    }

    pub fn uninit(&mut self) {
        info!("Uninitialising MidiService");
        sdk::uninitialise();
        trace!("Sdk uninit done");
        if let Some(output) = self.connection.take() {
            output.close();
        }
        trace!("MidiService uninit complete");
    }
}

impl Drop for MidiService {
    fn drop(&mut self) {
        self.uninit();
    }
}
