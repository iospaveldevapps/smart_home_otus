use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

use crate::{DeviceError, Report};

const TCP_TIMEOUT: Duration = Duration::from_secs(1);

#[derive(Debug)]
enum SocketBackend {
    Emulated { is_on: bool, power: f32 },
    Tcp { addr: SocketAddr },
}

#[derive(Debug)]
pub struct SmartSocket {
    backend: SocketBackend,
}

impl SmartSocket {
    /// Local socket that imitates real behaviour (for tests).
    pub fn new(is_on: bool, power: f32) -> Self {
        Self {
            backend: SocketBackend::Emulated { is_on, power },
        }
    }

    /// Socket controlled over TCP. Performs a verification connection.
    pub fn connect(addr: impl ToSocketAddrs) -> Result<Self, DeviceError> {
        let addr = addr
            .to_socket_addrs()?
            .next()
            .ok_or_else(|| DeviceError::Protocol("не удалось разобрать адрес".to_string()))?;

        TcpStream::connect_timeout(&addr, TCP_TIMEOUT)?;

        Ok(Self {
            backend: SocketBackend::Tcp { addr },
        })
    }

    pub fn turn_on(&mut self) -> Result<(), DeviceError> {
        self.set_state("TURN_ON", true)
    }

    pub fn turn_off(&mut self) -> Result<(), DeviceError> {
        self.set_state("TURN_OFF", false)
    }

    pub fn is_on(&self) -> Result<bool, DeviceError> {
        self.state().map(|(is_on, _)| is_on)
    }

    pub fn get_power(&self) -> Result<f32, DeviceError> {
        self.state().map(|(_, power)| power)
    }

    fn set_state(&mut self, command: &str, on: bool) -> Result<(), DeviceError> {
        match &mut self.backend {
            SocketBackend::Emulated { is_on, .. } => {
                *is_on = on;
                Ok(())
            }
            SocketBackend::Tcp { addr } => expect_ok(&send_command(*addr, command)?),
        }
    }

    fn state(&self) -> Result<(bool, f32), DeviceError> {
        match &self.backend {
            SocketBackend::Emulated { is_on, power } => {
                Ok((*is_on, if *is_on { *power } else { 0.0 }))
            }
            SocketBackend::Tcp { addr } => parse_state(&send_command(*addr, "GET_STATE")?),
        }
    }
}

fn send_command(addr: SocketAddr, command: &str) -> Result<String, DeviceError> {
    let mut stream = TcpStream::connect_timeout(&addr, TCP_TIMEOUT)?;
    stream.set_read_timeout(Some(TCP_TIMEOUT))?;
    stream.set_write_timeout(Some(TCP_TIMEOUT))?;

    stream.write_all(command.as_bytes())?;
    stream.write_all(b"\n")?;

    let mut response = String::new();
    BufReader::new(stream).read_line(&mut response)?;

    Ok(response.trim().to_string())
}

fn expect_ok(response: &str) -> Result<(), DeviceError> {
    if response == "OK" {
        Ok(())
    } else {
        Err(DeviceError::Protocol(format!(
            "неожиданный ответ `{response}`"
        )))
    }
}

fn parse_state(response: &str) -> Result<(bool, f32), DeviceError> {
    let mut parts = response.split_whitespace();

    if let (Some("STATE"), Some(state @ ("on" | "off")), Some(power)) =
        (parts.next(), parts.next(), parts.next())
    {
        let power = power.parse().map_err(|_| {
            DeviceError::Protocol(format!("некорректное значение мощности `{power}`"))
        })?;
        Ok((state == "on", power))
    } else {
        Err(DeviceError::Protocol(format!(
            "неожиданный ответ `{response}`"
        )))
    }
}

impl Report for SmartSocket {
    fn report(&self) -> String {
        match self.state() {
            Ok((is_on, power)) => {
                format!("Умная розетка. Включена: {is_on}. Текущая мощность: {power} W")
            }
            Err(error) => format!("Умная розетка. Не удалось получить данные: {error}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::{BufRead, BufReader, Write};
    use std::net::{SocketAddr, TcpListener};
    use std::thread;

    use super::SmartSocket;
    use crate::Report;

    #[test]
    fn reports_zero_power_when_turned_off() {
        let socket = SmartSocket::new(false, 150.0);

        assert!(!socket.is_on().unwrap());
        assert_eq!(socket.get_power().unwrap(), 0.0);
    }

    #[test]
    fn changes_state_with_turn_on_and_turn_off() {
        let mut socket = SmartSocket::new(false, 200.0);

        socket.turn_on().unwrap();
        assert!(socket.is_on().unwrap());
        assert_eq!(socket.get_power().unwrap(), 200.0);

        socket.turn_off().unwrap();
        assert!(!socket.is_on().unwrap());
        assert_eq!(socket.get_power().unwrap(), 0.0);
    }

    fn spawn_test_server() -> SocketAddr {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();

        thread::spawn(move || {
            let mut is_on = false;

            for stream in listener.incoming() {
                let Ok(stream) = stream else { continue };

                let mut line = String::new();
                if BufReader::new(&stream).read_line(&mut line).unwrap_or(0) == 0 {
                    continue;
                }

                let response = match line.trim() {
                    "TURN_ON" => {
                        is_on = true;
                        "OK".to_string()
                    }
                    "TURN_OFF" => {
                        is_on = false;
                        "OK".to_string()
                    }
                    "GET_STATE" => {
                        let power = if is_on { 123.0 } else { 0.0 };
                        format!("STATE {} {power}", if is_on { "on" } else { "off" })
                    }
                    _ => "ERR unknown command".to_string(),
                };

                writeln!(&stream, "{response}").ok();
            }
        });

        addr
    }

    #[test]
    fn controls_socket_over_tcp() {
        let addr = spawn_test_server();
        let mut socket = SmartSocket::connect(addr).unwrap();

        socket.turn_on().unwrap();
        assert!(socket.is_on().unwrap());
        assert_eq!(socket.get_power().unwrap(), 123.0);

        socket.turn_off().unwrap();
        assert!(!socket.is_on().unwrap());
        assert_eq!(socket.get_power().unwrap(), 0.0);
    }

    #[test]
    fn connect_fails_for_unreachable_address() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        drop(listener);

        assert!(SmartSocket::connect(addr).is_err());
    }

    #[test]
    fn tcp_report_contains_error_when_server_is_down() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let socket = SmartSocket::connect(addr).unwrap();
        drop(listener);

        assert!(socket.report().contains("Не удалось получить данные"));
    }
}
