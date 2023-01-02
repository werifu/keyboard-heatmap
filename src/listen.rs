use std::{sync::mpsc::SyncSender, time::SystemTime};

use rdev::listen as listen_event;

pub fn listen_keyboard(sender: SyncSender<rdev::EventType>) {
    if let Err(error) = listen_event(move |event: rdev::Event| {
        callback(event, sender.clone());
    }) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: rdev::Event, sender: SyncSender<rdev::EventType>) {
    let event_type = event.event_type;
    match event_type {
        rdev::EventType::KeyPress(_) => {
            sender.send(event_type).unwrap();
        }
        _ => {}
    }
}
