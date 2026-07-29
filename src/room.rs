use std::collections::HashMap;
use std::fmt;

use crate::{Report, SmartDevice};

/// Observer notified about room events.
///
/// Implemented automatically for closures, so both a subscriber object
/// and a closure can be passed to [`Room::subscribe`].
pub trait Subscriber {
    fn on_device_added(&mut self, name: &str, device: &SmartDevice);
}

impl<F: FnMut(&str, &SmartDevice)> Subscriber for F {
    fn on_device_added(&mut self, name: &str, device: &SmartDevice) {
        self(name, device);
    }
}

#[derive(Default)]
pub struct Room {
    devices: HashMap<String, SmartDevice>,
    subscribers: Vec<Box<dyn Subscriber>>,
}

impl fmt::Debug for Room {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Room")
            .field("devices", &self.devices)
            .field("subscribers", &self.subscribers.len())
            .finish()
    }
}

impl Room {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_devices(devices: HashMap<String, SmartDevice>) -> Self {
        Self {
            devices,
            subscribers: Vec::new(),
        }
    }

    /// Registers a callback fired every time a device is added to the room.
    pub fn subscribe(&mut self, subscriber: impl Subscriber + 'static) {
        self.subscribers.push(Box::new(subscriber));
    }

    pub fn add_device(
        &mut self,
        name: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        let name = name.into();
        let previous = self.devices.insert(name.clone(), device.into());

        let device = &self.devices[&name];
        for subscriber in &mut self.subscribers {
            subscriber.on_device_added(&name, device);
        }

        previous
    }

    pub fn remove_device(&mut self, name: &str) -> Option<SmartDevice> {
        self.devices.remove(name)
    }

    pub fn get_device(&self, name: &str) -> Option<&SmartDevice> {
        self.devices.get(name)
    }

    pub fn get_device_mut(&mut self, name: &str) -> Option<&mut SmartDevice> {
        self.devices.get_mut(name)
    }
}

impl Report for Room {
    fn report(&self) -> String {
        if self.devices.is_empty() {
            return "  Нет устройств.".to_string();
        }

        let mut devices: Vec<_> = self.devices.iter().collect();
        devices.sort_by(|(left_name, _), (right_name, _)| left_name.cmp(right_name));

        devices
            .into_iter()
            .map(|(name, device)| format!("  Устройство `{}`: {}", name, device.report()))
            .collect::<Vec<_>>()
            .join("\n")
    }
}

#[macro_export]
macro_rules! smart_room {
    ($($name:expr => $device:expr),* $(,)?) => {{
        let mut room = $crate::Room::new();
        $(
            room.add_device($name, $device);
        )*
        room
    }};
    ($(($name:expr, $device:expr)),* $(,)?) => {{
        let mut room = $crate::Room::new();
        $(
            room.add_device($name, $device);
        )*
        room
    }};
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::rc::Rc;

    use super::{Room, Subscriber};
    use crate::{Report, SmartDevice, SmartSocket, SmartThermometer};

    #[test]
    fn notifies_closure_subscriber_on_device_added() {
        let events = Rc::new(RefCell::new(Vec::new()));

        let mut room = Room::new();
        room.subscribe({
            let events = Rc::clone(&events);
            move |name: &str, _device: &SmartDevice| {
                events.borrow_mut().push(name.to_string());
            }
        });

        room.add_device("socket", SmartSocket::new(true, 100.0));
        room.add_device("thermometer", SmartThermometer::new(20.0));

        assert_eq!(*events.borrow(), ["socket", "thermometer"]);
    }

    #[test]
    fn notifies_object_subscriber_on_device_added() {
        struct EventLog(Rc<RefCell<Vec<String>>>);

        impl Subscriber for EventLog {
            fn on_device_added(&mut self, name: &str, device: &SmartDevice) {
                self.0
                    .borrow_mut()
                    .push(format!("{name}: {}", device.report()));
            }
        }

        let events = Rc::new(RefCell::new(Vec::new()));

        let mut room = Room::new();
        room.subscribe(EventLog(Rc::clone(&events)));
        room.add_device("socket", SmartSocket::new(true, 100.0));

        let events = events.borrow();
        assert_eq!(events.len(), 1);
        assert!(events[0].starts_with("socket: Умная розетка"));
    }

    #[test]
    fn returns_device_by_key() {
        let mut room = Room::new();
        room.add_device("thermometer", SmartThermometer::new(22.0));
        room.add_device("socket", SmartSocket::new(true, 100.0));

        match room.get_device("thermometer") {
            Some(SmartDevice::Thermometer(thermometer)) => {
                assert_eq!(thermometer.get_temperature().unwrap(), 22.0);
            }
            _ => panic!("expected thermometer"),
        }

        assert!(room.get_device("unknown").is_none());
    }

    #[test]
    fn allows_mutating_device_by_key() {
        let mut room = Room::new();
        room.add_device("socket", SmartSocket::new(true, 100.0));

        match room.get_device_mut("socket") {
            Some(SmartDevice::Socket(socket)) => socket.turn_off().unwrap(),
            _ => panic!("expected socket"),
        }

        match room.get_device("socket") {
            Some(SmartDevice::Socket(socket)) => {
                assert!(!socket.is_on().unwrap());
                assert_eq!(socket.get_power().unwrap(), 0.0);
            }
            _ => panic!("expected socket"),
        }
    }

    #[test]
    fn removes_device_by_key() {
        let mut room = Room::new();
        room.add_device("socket", SmartSocket::new(true, 100.0));

        assert!(room.remove_device("socket").is_some());
        assert!(room.get_device("socket").is_none());
    }

    #[test]
    fn macro_builds_room_from_devices() {
        let room = crate::smart_room!(
            ("thermometer", SmartThermometer::new(22.0)),
            ("socket", SmartSocket::new(true, 100.0)),
        );

        match room.get_device("thermometer") {
            Some(SmartDevice::Thermometer(thermometer)) => {
                assert_eq!(thermometer.get_temperature().unwrap(), 22.0);
            }
            _ => panic!("expected thermometer"),
        }
    }

    #[test]
    fn report_contains_all_devices() {
        let room = crate::smart_room! {
            "socket" => SmartSocket::new(true, 100.0),
            "thermometer" => SmartThermometer::new(22.0),
        };

        let report = room.report();
        assert!(report.contains("Устройство `socket`"));
        assert!(report.contains("Устройство `thermometer`"));
    }
}
