# Wooting Analog MIDI

This is a fork of [WootingKb/wooting-analog-midi](https://github.com/WootingKb/wooting-analog-midi) that drastically changes the original, only the wooting-analog-midi-core crate is left nearly as is.

The featues are simmilar to the original, it is however not run as an application, but as a system tray service.

Also includes an AutoHotkey script to disable numpad keys, which are used to extend the range of unassigned keys (F13-F24).

## TODO

For now everthing has to be configured in the source code itself.

- [ ] Select MIDI output port
- [ ] Select MIDI channel
- [ ] Configure threshold, velocity scaling etc.
