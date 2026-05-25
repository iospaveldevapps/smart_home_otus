use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use crate::{Report, Room, SmartDevice};

#[derive(Debug, PartialEq, Eq)]
pub enum SmartHomeError {
    RoomNotFound { room: String },
    DeviceNotFound { room: String, device: String },
}

impl fmt::Display for SmartHomeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SmartHomeError::RoomNotFound { room } => {
                write!(f, "Комната `{room}` не найдена")
            }
            SmartHomeError::DeviceNotFound { room, device } => {
                write!(f, "Устройство `{device}` в комнате `{room}` не найдено")
            }
        }
    }
}

impl Error for SmartHomeError {}

#[derive(Debug, Default)]
pub struct SmartHome {
    rooms: HashMap<String, Room>,
}

impl SmartHome {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_rooms(rooms: HashMap<String, Room>) -> Self {
        Self { rooms }
    }

    pub fn add_room(&mut self, name: impl Into<String>, room: Room) -> Option<Room> {
        self.rooms.insert(name.into(), room)
    }

    pub fn remove_room(&mut self, name: &str) -> Option<Room> {
        self.rooms.remove(name)
    }

    pub fn get_room(&self, name: &str) -> Option<&Room> {
        self.rooms.get(name)
    }

    pub fn get_room_mut(&mut self, name: &str) -> Option<&mut Room> {
        self.rooms.get_mut(name)
    }

    pub fn get_smart_device(
        &self,
        room_name: &str,
        device_name: &str,
    ) -> Result<&SmartDevice, SmartHomeError> {
        let room = self
            .get_room(room_name)
            .ok_or_else(|| SmartHomeError::RoomNotFound {
                room: room_name.to_string(),
            })?;

        room.get_device(device_name)
            .ok_or_else(|| SmartHomeError::DeviceNotFound {
                room: room_name.to_string(),
                device: device_name.to_string(),
            })
    }

    pub fn get_smart_device_mut(
        &mut self,
        room_name: &str,
        device_name: &str,
    ) -> Result<&mut SmartDevice, SmartHomeError> {
        let room = self
            .get_room_mut(room_name)
            .ok_or_else(|| SmartHomeError::RoomNotFound {
                room: room_name.to_string(),
            })?;

        room.get_device_mut(device_name)
            .ok_or_else(|| SmartHomeError::DeviceNotFound {
                room: room_name.to_string(),
                device: device_name.to_string(),
            })
    }
}

impl Report for SmartHome {
    fn report(&self) -> String {
        let mut report = String::from("Отчёт по умному дому:");

        if self.rooms.is_empty() {
            report.push_str("\nНет комнат.");
            return report;
        }

        let mut room_names: Vec<_> = self.rooms.keys().collect();
        room_names.sort();

        for name in room_names {
            let room = self
                .rooms
                .get(name)
                .expect("room name was read from the same map");
            report.push_str(&format!("\nКомната `{}`:\n{}", name, room.report()));
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::{SmartHome, SmartHomeError};
    use crate::{Report, Room, SmartDevice, SmartSocket};

    #[test]
    fn allows_mutating_devices_inside_room() {
        let mut room = Room::new();
        room.add_device("socket", SmartSocket::new(true, 75.0));

        let mut home = SmartHome::new();
        home.add_room("living room", room);

        match home.get_smart_device_mut("living room", "socket") {
            Ok(SmartDevice::Socket(socket)) => socket.turn_off(),
            _ => panic!("expected socket"),
        }

        match home.get_smart_device("living room", "socket") {
            Ok(SmartDevice::Socket(socket)) => {
                assert!(!socket.is_on());
                assert_eq!(socket.get_power(), 0.0);
            }
            _ => panic!("expected socket"),
        }
    }

    #[test]
    fn adds_and_removes_rooms() {
        let mut home = SmartHome::new();

        home.add_room("living room", Room::new());
        assert!(home.get_room("living room").is_some());

        assert!(home.remove_room("living room").is_some());
        assert!(home.get_room("living room").is_none());
    }

    #[test]
    fn returns_specific_error_for_missing_room_or_device() {
        let mut home = SmartHome::new();
        home.add_room("living room", Room::new());

        assert_eq!(
            home.get_smart_device("kitchen", "socket").unwrap_err(),
            SmartHomeError::RoomNotFound {
                room: "kitchen".to_string()
            }
        );

        assert_eq!(
            home.get_smart_device("living room", "socket").unwrap_err(),
            SmartHomeError::DeviceNotFound {
                room: "living room".to_string(),
                device: "socket".to_string(),
            }
        );
    }

    #[test]
    fn report_contains_room_names() {
        let mut home = SmartHome::new();
        home.add_room("living room", Room::new());

        assert!(home.report().contains("Комната `living room`"));
    }
}
