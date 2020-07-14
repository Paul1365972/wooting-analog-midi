# Wooting Analog MIDI

Experimental cross-platform Rust implementation for a Virtual MIDI device using the [Wooting Analog SDK](https://github.com/WootingKb/wooting-analog-sdk)!

## Development Setup

### Dependencies

- [yarn](https://yarnpkg.com/getting-started) Is our preferred Node package manager
- [Rust & Tauri](https://tauri.studio/docs/getting-started/intro#setting-up-your-environment)

#### Linux

The `libasound2-dev` package may be required to be installed:

```bash
sudo apt install libasound2-dev
```

For packaging `AppImage` `squashfs-tools` may be required:

```bash
sudo apt install squashfs-tools
```

### Directory Structure

- `src` - React Frontend source code
- `wooting-analog-midi` - Rust source for the virtual MIDI device using the [Wooting Analog SDK](https://github.com/WootingKb/wooting-analog-sdk)!
- `src-tauri` - The Tauri host process code which bootstraps the web view & contains the glue code between the React frontend and the Rust backend

### Get going

First you gotta install dependencies of the project

```bash
yarn
```

Then you should be able to run the application in development mode, which includes hot reloading automatically on save:
```bash
yarn tauri dev
```

If you want to build a distributable binary/package run:
```bash
yarn tauri build
```

For more details & other commands, Tauri has a good reference for [development commands here](https://tauri.studio/docs/usage/development/development)