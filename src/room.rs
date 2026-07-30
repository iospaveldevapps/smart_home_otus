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
    subscribers: Vec<Box<dyn Subscriber + Send>>,
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
    pub fn subscribe(&mut self, subscriber: impl Subscriber + Send + 'static) {
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

    /// Iterates over `(name, device)` pairs in arbitrary order.
    pub fn devices(&self) -> impl Iterator<Item = (&str, &SmartDevice)> {
        self.devices
            .iter()
            .map(|(name, device)| (name.as_str(), device))
    }

    /// Device names sorted alphabetically.
    pub fn device_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.devices.keys().cloned().collect();
        names.sort();
        names
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
    use std::sync::{Arc, Mutex};

    use super::{Room, Subscriber};
    use crate::{Report, SmartDevice, SmartSocket, SmartThermometer};

    #[test]
    fn notifies_closure_subscriber_on_device_added() {
        let events = Arc::new(Mutex::new(Vec::new()));

        let mut room = Room::new();
        room.subscribe({
            let events = Arc::clone(&events);
            move |name: &str, _device: &SmartDevice| {
                events.lock().unwrap().push(name.to_string());
            }
        });

        room.add_device("socket", SmartSocket::new(true, 100.0));
        room.add_device("thermometer", SmartThermometer::new(20.0));

        assert_eq!(*events.lock().unwrap(), ["socket", "thermometer"]);
    }

    #[test]
    fn notifies_object_subscriber_on_device_added() {
        struct EventLog(Arc<Mutex<Vec<String>>>);

        impl Subscriber for EventLog {
            fn on_device_added(&mut self, name: &str, device: &SmartDevice) {
                self.0
                    .lock()
                    .unwrap()
                    .push(format!("{name}: {}", device.report()));
            }
        }

        let events = Arc::new(Mutex::new(Vec::new()));

        let mut room = Room::new();
        room.subscribe(EventLog(Arc::clone(&events)));
        room.add_device("socket", SmartSocket::new(true, 100.0));

        let events = events.lock().unwrap();
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
