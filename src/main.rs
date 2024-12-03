use anyhow::Result;
use env_logger::Env;
use image::{load_from_memory_with_format, ImageFormat};
use log::info;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    thread::{self, JoinHandle},
    time::Duration,
};
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
use wooting_analog_midi_core::{
    config::{Config, KeyConfig},
    HIDCodes, MidiService, NoteID, REFRESH_RATE,
};

struct Service {
    midi: MidiService,
    stop: bool,
}

impl Service {
    fn new() -> Self {
        Self {
            midi: MidiService::new(),
            stop: false,
        }
    }
}

fn spawn_polling_loop(service: &Arc<Mutex<Service>>) -> JoinHandle<Result<()>> {
    let service = service.clone();
    thread::spawn(move || {
        info!("Starting polling loop");

        let duration = Duration::from_secs_f32(1.0 / REFRESH_RATE);
        let mut interval = spin_sleep_util::interval(duration)
            .with_missed_tick_behavior(spin_sleep_util::MissedTickBehavior::Delay);
        let mut reporter = spin_sleep_util::RateReporter::new(Duration::from_secs_f64(1.0));

        loop {
            interval.tick();
            if let Some(tps) = reporter.increment_and_report() {
                info!("Current polling rate: {:.2}Hz", tps);
            }
            let mut service = service.lock().unwrap();
            if service.stop {
                return Ok(());
            }
            service.midi.poll()?;
        }
    })
}

fn run_event_loop(service: Arc<Mutex<Service>>, handle: JoinHandle<Result<()>>) -> Result<()> {
    let mut service = Some(service);

    let event_loop = EventLoopBuilder::new().build();

    let tray_menu = Menu::new();
    let quit_i = MenuItem::new("Quit", true, None);
    tray_menu
        .append_items(&[
            &PredefinedMenuItem::about(
                None,
                Some(AboutMetadata {
                    name: Some("TODO".to_string()),
                    copyright: Some("Copyright TODO".to_string()),
                    ..Default::default()
                }),
            ),
            &PredefinedMenuItem::separator(),
            &quit_i,
        ])
        .expect("Failed to add item to tray menu");

    let icon = load_icon();
    let mut tray_icon = Some(
        TrayIconBuilder::new()
            .with_menu(Box::new(tray_menu))
            .with_tooltip("wooing-analog-midi")
            .with_icon(icon)
            .build()
            .unwrap(),
    );

    TrayIconEvent::set_event_handler(Some(|_| {}));
    let menu_channel = MenuEvent::receiver();

    let mut handle = Some(handle);
    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        if let Ok(event) = menu_channel.try_recv() {
            println!("{event:?}");
            if event.id == quit_i.id() {
                tray_icon.take();
                service.take().unwrap().lock().unwrap().stop = true;
                handle.take().unwrap().join().unwrap().unwrap();

                *control_flow = ControlFlow::Exit;
            }
        }
    })
}

fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let service = Arc::new(Mutex::new(Service::new()));
    {
        let mut service = service.lock().unwrap();
        service.midi.init()?;
        service.midi.select_port(0)?;
        // info!("Ports: {:#?}", service.midi.port_options);
        let config = create_config();
        service.midi.set_config(config)?;
    }

    let handle = spawn_polling_loop(&service);

    run_event_loop(service, handle)
}

fn create_config() -> Config {
    let mut key_configs = HashMap::default();
    for (index, code) in [
        HIDCodes::Q,
        HIDCodes::N2,
        HIDCodes::W,
        HIDCodes::E,
        HIDCodes::R,
        HIDCodes::N5,
        HIDCodes::T,
        HIDCodes::N6,
        HIDCodes::T,
        HIDCodes::N7,
        HIDCodes::U,
        HIDCodes::I,
        HIDCodes::N9,
        HIDCodes::O,
        HIDCodes::N0,
        HIDCodes::P,
    ]
    .into_iter()
    .enumerate()
    {
        key_configs.insert(
            code,
            KeyConfig {
                note_id: 60 + index as NoteID,
                ..Default::default()
            },
        );
    }

    return Config {
        key_configs,
        toggle_keys: vec![HIDCodes::F12],
        ..Default::default()
    };
}

fn load_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("icon.png");

    let (icon_rgba, icon_width, icon_height) = {
        let image = load_from_memory_with_format(bytes, ImageFormat::Png)
            .expect("Failed to load icon image")
            .into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };

    tray_icon::Icon::from_rgba(icon_rgba, icon_width, icon_height).expect("Failed to create icon")
}
