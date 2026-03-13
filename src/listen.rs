use std::sync::mpsc::SyncSender;

use rdev::listen as listen_event;

pub fn listen_keyboard(sender: SyncSender<rdev::Event>) {
    if let Err(error) = listen_event(move |event: rdev::Event| {
        callback(event, sender.clone());
    }) {
        println!("Error: {:?}", error)
    }
}

fn callback(event: rdev::Event, sender: SyncSender<rdev::Event>) {
    if let rdev::EventType::KeyPress(_) = event.event_type {
        sender.send(event).unwrap();
    }
}
