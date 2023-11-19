use std::{
    collections::HashMap,
    sync::{atomic::AtomicBool, Arc, Mutex},
    thread::{self, JoinHandle},
};

use anyhow::Result;
use env_logger::Env;
use image::{load_from_memory_with_format, ImageFormat};
use log::info;
use spin_sleep::LoopHelper;
use tao::event_loop::{ControlFlow, EventLoopBuilder};
use tray_icon::{
    menu::{AboutMetadata, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder, TrayIconEvent,
};
use wooting_analog_midi_core::{HIDCodes, MidiService};

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
    thread::spawn(move || -> Result<()> {
        info!("Starting polling loop");
        let mut loop_helper = LoopHelper::builder()
            .report_interval_s(1.0)
            .build_with_target_rate(200.0);

        loop {
            loop_helper.loop_start();
            if let Some(fps) = loop_helper.report_rate() {
                info!("Current polling rate: {:.2}/s", fps);
            }

            let mut service = service.lock().unwrap();
            if service.stop {
                return Ok(());
            }
            service.midi.poll()?;
            drop(service);

            loop_helper.loop_sleep();
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
        service.midi.select_port(1)?;
        // info!("Ports: {:#?}", service.midi.port_options);
        let mapping = [
            (HIDCodes::F13, vec![(0, 60)]),
            (HIDCodes::F14, vec![(0, 61)]),
            (HIDCodes::F15, vec![(0, 62)]),
            (HIDCodes::F16, vec![(0, 63)]),
            (HIDCodes::F17, vec![(0, 64)]),
            (HIDCodes::F18, vec![(0, 65)]),
            (HIDCodes::F19, vec![(0, 66)]),
            (HIDCodes::F20, vec![(0, 67)]),
            (HIDCodes::F21, vec![(0, 68)]),
            (HIDCodes::F22, vec![(0, 69)]),
            (HIDCodes::F23, vec![(0, 70)]),
            (HIDCodes::F24, vec![(0, 71)]),
            (HIDCodes::Numpad1, vec![(0, 72)]),
            (HIDCodes::Numpad2, vec![(0, 73)]),
            (HIDCodes::Numpad3, vec![(0, 74)]),
            (HIDCodes::Numpad4, vec![(0, 75)]),
            (HIDCodes::Numpad5, vec![(0, 76)]),
            (HIDCodes::Numpad6, vec![(0, 77)]),
            (HIDCodes::Numpad7, vec![(0, 78)]),
        ]
        .iter()
        .cloned()
        .collect();
        service.midi.update_mapping(&mapping)?;
        service.midi.amount_to_shift = 12;
    }

    let handle = spawn_polling_loop(&service);

    run_event_loop(service, handle)
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
