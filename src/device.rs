use crate::{SmartSocket, SmartThermometer};

pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl SmartDevice {
    pub fn print_state(&self) {
        match self {
            SmartDevice::Thermometer(thermometer) => {
                println!(
                    "Умный термометр. Температура: {}°C",
                    thermometer.get_temperature()
                );
            }
            SmartDevice::Socket(socket) => {
                println!(
                    "Умная розетка. Включена: {}. Текущая мощность: {} W",
                    socket.is_on(),
                    socket.get_power()
                );
            }
        }
    }
}
