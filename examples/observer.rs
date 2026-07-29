//! Observer pattern demo: `cargo run --example observer`.
//!
//! Callbacks are fired when a device is added to a room. Both a
//! subscriber object and a closure can be registered (dynamic
//! polymorphism via trait objects).

use smart_home::{Report, Room, SmartDevice, SmartSocket, SmartThermometer, Subscriber};

/// Subscriber object that counts added devices.
#[derive(Debug, Default)]
struct DeviceCounter {
    count: u32,
}

impl Subscriber for DeviceCounter {
    fn on_device_added(&mut self, name: &str, _device: &SmartDevice) {
        self.count += 1;
        println!(
            "Счётчик: устройств добавлено — {}, последнее — `{name}`",
            self.count
        );
    }
}

fn main() {
    let mut room = Room::new();

    room.subscribe(DeviceCounter::default());
    room.subscribe(|name: &str, device: &SmartDevice| {
        println!(
            "Замыкание: добавлено устройство `{name}`: {}",
            device.report()
        );
    });

    room.add_device("socket", SmartSocket::new(true, 100.0));
    room.add_device("thermometer", SmartThermometer::new(22.0));
}
