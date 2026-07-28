use std::net::{SocketAddr, ToSocketAddrs, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;
use std::time::Duration;
use std::{io, thread};

use crate::{DeviceError, Report};

const UDP_READ_TIMEOUT: Duration = Duration::from_millis(100);

#[derive(Debug)]
enum ThermometerBackend {
    Emulated(f32),
    Udp {
        temperature: Arc<Mutex<Option<f32>>>,
        stop: Arc<AtomicBool>,
        handle: Option<JoinHandle<()>>,
        local_addr: SocketAddr,
    },
}

#[derive(Debug)]
pub struct SmartThermometer {
    backend: ThermometerBackend,
}

impl SmartThermometer {
    /// Local thermometer that imitates receiving remote data (for tests).
    pub fn new(temperature: f32) -> Self {
        Self {
            backend: ThermometerBackend::Emulated(temperature),
        }
    }

    /// Thermometer that receives temperature values as UDP packets on `addr`.
    /// The background thread starts on creation and stops when the object is dropped.
    pub fn listen(addr: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        let socket = UdpSocket::bind(addr)?;
        socket.set_read_timeout(Some(UDP_READ_TIMEOUT))?;
        let local_addr = socket.local_addr()?;

        let temperature = Arc::new(Mutex::new(None));
        let stop = Arc::new(AtomicBool::new(false));

        let handle = thread::spawn({
            let temperature = Arc::clone(&temperature);
            let stop = Arc::clone(&stop);

            move || {
                let mut buffer = [0u8; 64];

                while !stop.load(Ordering::Relaxed) {
                    match socket.recv_from(&mut buffer) {
                        Ok((length, _)) => {
                            let Ok(text) = str::from_utf8(&buffer[..length]) else {
                                continue;
                            };
                            let Ok(value) = text.trim().parse::<f32>() else {
                                continue;
                            };
                            *temperature.lock().unwrap() = Some(value);
                        }
                        Err(error)
                            if matches!(
                                error.kind(),
                                io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
                            ) => {}
                        Err(_) => break,
                    }
                }
            }
        });

        Ok(Self {
            backend: ThermometerBackend::Udp {
                temperature,
                stop,
                handle: Some(handle),
                local_addr,
            },
        })
    }

    pub fn get_temperature(&self) -> Result<f32, DeviceError> {
        match &self.backend {
            ThermometerBackend::Emulated(temperature) => Ok(*temperature),
            ThermometerBackend::Udp { temperature, .. } => {
                temperature.lock().unwrap().ok_or(DeviceError::NoData)
            }
        }
    }

    /// Address the thermometer receives UDP packets on (UDP mode only).
    pub fn local_addr(&self) -> Option<SocketAddr> {
        match &self.backend {
            ThermometerBackend::Emulated(_) => None,
            ThermometerBackend::Udp { local_addr, .. } => Some(*local_addr),
        }
    }
}

impl Drop for SmartThermometer {
    fn drop(&mut self) {
        if let ThermometerBackend::Udp { stop, handle, .. } = &mut self.backend {
            stop.store(true, Ordering::Relaxed);
            if let Some(handle) = handle.take() {
                handle.join().ok();
            }
        }
    }
}

impl Report for SmartThermometer {
    fn report(&self) -> String {
        match self.get_temperature() {
            Ok(temperature) => format!("Умный термометр. Температура: {temperature}°C"),
            Err(error) => format!("Умный термометр. Не удалось получить данные: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::net::UdpSocket;
    use std::thread;
    use std::time::{Duration, Instant};

    use super::SmartThermometer;

    #[test]
    fn returns_stored_temperature() {
        let thermometer = SmartThermometer::new(21.5);

        assert_eq!(thermometer.get_temperature().unwrap(), 21.5);
    }

    #[test]
    fn returns_no_data_before_first_packet() {
        let thermometer = SmartThermometer::listen("127.0.0.1:0").unwrap();

        assert!(thermometer.get_temperature().is_err());
    }

    #[test]
    fn receives_temperature_over_udp() {
        let thermometer = SmartThermometer::listen("127.0.0.1:0").unwrap();
        let addr = thermometer.local_addr().unwrap();

        let sender = UdpSocket::bind("127.0.0.1:0").unwrap();
        sender.send_to(b"23.5", addr).unwrap();

        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            match thermometer.get_temperature() {
                Ok(value) => {
                    assert_eq!(value, 23.5);
                    break;
                }
                Err(error) => {
                    assert!(
                        Instant::now() < deadline,
                        "температура не получена: {error}"
                    );
                    thread::sleep(Duration::from_millis(20));
                }
            }
        }
    }

    #[test]
    fn drop_stops_background_thread() {
        let thermometer = SmartThermometer::listen("127.0.0.1:0").unwrap();

        // Drop must not hang: the thread exits via the stop flag.
        drop(thermometer);
    }
}
