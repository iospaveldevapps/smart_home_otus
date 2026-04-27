use crate::SmartDevice;

pub struct Room {
    devices: Vec<SmartDevice>,
}

impl Room {
    pub fn new(devices: Vec<SmartDevice>) -> Self {
        Self { devices }
    }

    pub fn get_device(&self, index: usize) -> &SmartDevice {
        self.devices
            .get(index)
            .unwrap_or_else(|| panic!("Устройства с индексом {} не существует", index))
    }

    pub fn get_device_mut(&mut self, index: usize) -> &mut SmartDevice {
        self.devices
            .get_mut(index)
            .unwrap_or_else(|| panic!("Устройства с индексом {} не существует", index))
    }

    pub fn print_report(&self) {
        for (index, device) in self.devices.iter().enumerate() {
            print!("Устройство {}: ", index);
            device.print_state();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Room;
    use crate::{SmartDevice, SmartSocket, SmartThermometer};

    #[test]
    fn returns_device_by_index() {
        let room = Room::new(vec![
            SmartDevice::Thermometer(SmartThermometer::new(22.0)),
            SmartDevice::Socket(SmartSocket::new(true, 100.0)),
        ]);

        match room.get_device(0) {
            SmartDevice::Thermometer(thermometer) => {
                assert_eq!(thermometer.get_temperature(), 22.0);
            }
            SmartDevice::Socket(_) => panic!("expected thermometer"),
        }
    }

    #[test]
    fn allows_mutating_device_by_index() {
        let mut room = Room::new(vec![SmartDevice::Socket(SmartSocket::new(true, 100.0))]);

        match room.get_device_mut(0) {
            SmartDevice::Socket(socket) => socket.turn_off(),
            SmartDevice::Thermometer(_) => panic!("expected socket"),
        }

        match room.get_device(0) {
            SmartDevice::Socket(socket) => {
                assert!(!socket.is_on());
                assert_eq!(socket.get_power(), 0.0);
            }
            SmartDevice::Thermometer(_) => panic!("expected socket"),
        }
    }
}
