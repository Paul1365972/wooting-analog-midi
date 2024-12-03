use crate::{Channel, NoteID};
use anyhow::Result;
use midir::MidiOutputConnection;

const NOTE_ON_MSG: u8 = 0x90;
const NOTE_OFF_MSG: u8 = 0x80;
const POLY_AFTERTOUCH_MSG: u8 = 0xA0;
pub(crate) const MIDI_NOTE_MAX: NoteID = 108;
pub(crate) const MIDI_NOTE_MIN: NoteID = 21;

pub(crate) trait NoteSink {
    fn note_on(&mut self, note_id: NoteID, velocity: f32, channel: Channel) -> Result<()>;
    fn note_off(&mut self, note_id: NoteID, velocity: f32, channel: Channel) -> Result<()>;
    fn polyphonic_aftertouch(
        &mut self,
        note_id: NoteID,
        pressure: f32,
        channel: Channel,
    ) -> Result<()>;
}

impl NoteSink for MidiOutputConnection {
    fn note_on(&mut self, note_id: NoteID, velocity: f32, channel: Channel) -> Result<()> {
        let vbyte = (f32::min(velocity, 1.0) * 127.0) as u8;
        self.send(&[NOTE_ON_MSG | channel, note_id, vbyte])?;
        Ok(())
    }

    fn note_off(&mut self, note_id: NoteID, velocity: f32, channel: Channel) -> Result<()> {
        let vbyte = (f32::min(velocity, 1.0) * 127.0) as u8;
        self.send(&[NOTE_OFF_MSG | channel, note_id, vbyte])?;
        Ok(())
    }

    fn polyphonic_aftertouch(
        &mut self,
        note_id: NoteID,
        pressure: f32,
        channel: Channel,
    ) -> Result<()> {
        self.send(&[
            POLY_AFTERTOUCH_MSG | channel,
            note_id,
            (f32::min(pressure, 1.0) * 127.0) as u8,
        ])?;
        Ok(())
    }
}
