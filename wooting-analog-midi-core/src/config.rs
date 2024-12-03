use rustc_hash::FxHashMap;
use wooting_analog_wrapper::HIDCodes;

use crate::{Channel, NoteID};

#[derive(Debug, Clone)]
pub struct KeyConfig {
    pub note_id: NoteID,
    pub channel: Channel,
    pub actuation_point: f32,
    pub threshold: f32,
    pub velocity_scale: f32,
    pub aftertouch: bool,
    pub shift_amount: i8,
}

impl Default for KeyConfig {
    fn default() -> Self {
        Self {
            note_id: 60, // Middle C
            channel: 0,
            actuation_point: 0.0,
            threshold: 0.8,
            velocity_scale: 5.0,
            aftertouch: true,
            shift_amount: 12,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Config {
    pub toggle_keys: Vec<HIDCodes>,
    pub modifier_keys: Vec<HIDCodes>,
    pub key_configs: FxHashMap<HIDCodes, KeyConfig>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            toggle_keys: vec![],
            modifier_keys: vec![HIDCodes::LeftShift, HIDCodes::RightShift],
            key_configs: FxHashMap::default(),
        }
    }
}
