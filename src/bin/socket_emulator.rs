//! Smart socket emulator: a TCP server with non-blocking I/O.
//!
//! Usage: `cargo run --bin socket_emulator -- 127.0.0.1:55331`

use std::env;
use std::error::Error;
use std::io::{ErrorKind, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

const DEFAULT_ADDR: &str = "127.0.0.1:55331";
const NOMINAL_POWER: f32 = 2000.0;
const POLL_INTERVAL: Duration = Duration::from_millis(10);

struct SocketState {
    is_on: bool,
    power: f32,
}

impl SocketState {
    fn handle_command(&mut self, command: &str) -> String {
        match command {
            "TURN_ON" => {
                self.is_on = true;
                "OK".to_string()
            }
            "TURN_OFF" => {
                self.is_on = false;
                "OK".to_string()
            }
            "GET_STATE" => {
                let power = if self.is_on { self.power } else { 0.0 };
                format!("STATE {} {power}", if self.is_on { "on" } else { "off" })
            }
            _ => "ERR unknown command".to_string(),
        }
    }
}

struct Client {
    stream: TcpStream,
    peer: SocketAddr,
    incoming: Vec<u8>,
    outgoing: Vec<u8>,
}

impl Client {
    fn new(stream: TcpStream, peer: SocketAddr) -> Self {
        Self {
            stream,
            peer,
            incoming: Vec::new(),
            outgoing: Vec::new(),
        }
    }

    /// Returns `false` if the client disconnected or an error occurred.
    fn poll(&mut self, state: &mut SocketState) -> bool {
        let mut buffer = [0u8; 1024];

        loop {
            match self.stream.read(&mut buffer) {
                Ok(0) => {
                    println!("Клиент отключился: {}", self.peer);
                    return false;
                }
                Ok(length) => self.incoming.extend_from_slice(&buffer[..length]),
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => {
                    eprintln!("Ошибка чтения ({}): {error}", self.peer);
                    return false;
                }
            }
        }

        while let Some(position) = self.incoming.iter().position(|&byte| byte == b'\n') {
            let line: Vec<u8> = self.incoming.drain(..=position).collect();
            let command = String::from_utf8_lossy(&line).trim().to_string();
            let response = state.handle_command(&command);

            println!("{}: {command} -> {response}", self.peer);
            self.outgoing.extend_from_slice(response.as_bytes());
            self.outgoing.push(b'\n');
        }

        while !self.outgoing.is_empty() {
            match self.stream.write(&self.outgoing) {
                Ok(0) => return false,
                Ok(length) => {
                    self.outgoing.drain(..length);
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => break,
                Err(error) if error.kind() == ErrorKind::Interrupted => continue,
                Err(error) => {
                    eprintln!("Ошибка записи ({}): {error}", self.peer);
                    return false;
                }
            }
        }

        true
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let addr = env::args().nth(1).unwrap_or_else(|| {
        println!("Адрес не указан, используется адрес по умолчанию: {DEFAULT_ADDR}");
        DEFAULT_ADDR.to_string()
    });

    let listener = TcpListener::bind(&addr)?;
    listener.set_nonblocking(true)?;
    println!("Имитатор умной розетки слушает {addr}");

    let mut state = SocketState {
        is_on: false,
        power: NOMINAL_POWER,
    };
    let mut clients: Vec<Client> = Vec::new();

    loop {
        match listener.accept() {
            Ok((stream, peer)) => {
                stream.set_nonblocking(true)?;
                println!("Клиент подключился: {peer}");
                clients.push(Client::new(stream, peer));
            }
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            Err(error) => eprintln!("Ошибка приёма соединения: {error}"),
        }

        clients.retain_mut(|client| client.poll(&mut state));

        thread::sleep(POLL_INTERVAL);
    }
}
