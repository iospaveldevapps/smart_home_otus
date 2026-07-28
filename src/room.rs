use std::collections::HashMap;

use crate::{Report, SmartDevice};

#[derive(Debug, Default)]
pub struct Room {
    devices: HashMap<String, SmartDevice>,
}

impl Room {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_devices(devices: HashMap<String, SmartDevice>) -> Self {
        Self { devices }
    }

    pub fn add_device(
        &mut self,
        name: impl Into<String>,
        device: impl Into<SmartDevice>,
    ) -> Option<SmartDevice> {
        self.devices.insert(name.into(), device.into())
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
    use super::Room;
    use crate::{Report, SmartDevice, SmartSocket, SmartThermometer};

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
