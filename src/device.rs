use crate::{Report, SmartSocket, SmartThermometer};

#[derive(Debug)]
pub enum SmartDevice {
    Thermometer(SmartThermometer),
    Socket(SmartSocket),
}

impl From<SmartSocket> for SmartDevice {
    fn from(socket: SmartSocket) -> Self {
        Self::Socket(socket)
    }
}

impl From<SmartThermometer> for SmartDevice {
    fn from(thermometer: SmartThermometer) -> Self {
        Self::Thermometer(thermometer)
    }
}

impl Report for SmartDevice {
    fn report(&self) -> String {
        match self {
            SmartDevice::Thermometer(thermometer) => thermometer.report(),
            SmartDevice::Socket(socket) => socket.report(),
        }
    }
}
