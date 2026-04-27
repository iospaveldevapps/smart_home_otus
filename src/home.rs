use crate::Room;

pub struct SmartHome {
    rooms: Vec<Room>,
}

impl SmartHome {
    pub fn new(rooms: Vec<Room>) -> Self {
        Self { rooms }
    }

    pub fn get_room(&self, index: usize) -> &Room {
        self.rooms
            .get(index)
            .unwrap_or_else(|| panic!("Комнаты с индексом {} не существует", index))
    }

    pub fn get_room_mut(&mut self, index: usize) -> &mut Room {
        self.rooms
            .get_mut(index)
            .unwrap_or_else(|| panic!("Комнаты с индексом {} не существует", index))
    }

    pub fn print_report(&self) {
        println!("Отчёт по умному дому:");

        for (index, room) in self.rooms.iter().enumerate() {
            println!("Комната {}:", index);
            room.print_report();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SmartHome;
    use crate::{Room, SmartDevice, SmartSocket};

    #[test]
    fn allows_mutating_devices_inside_room() {
        let mut home = SmartHome::new(vec![Room::new(vec![SmartDevice::Socket(
            SmartSocket::new(true, 75.0),
        )])]);

        match home.get_room_mut(0).get_device_mut(0) {
            SmartDevice::Socket(socket) => socket.turn_off(),
            SmartDevice::Thermometer(_) => panic!("expected socket"),
        }

        match home.get_room(0).get_device(0) {
            SmartDevice::Socket(socket) => {
                assert!(!socket.is_on());
                assert_eq!(socket.get_power(), 0.0);
            }
            SmartDevice::Thermometer(_) => panic!("expected socket"),
        }
    }
}
