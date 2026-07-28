//! Smart thermometer emulator: periodically sends temperature as UDP packets.
//!
//! The destination address and send period are read from a configuration file
//! (the first command-line argument, `thermometer_emulator.conf` by default):
//!
//! ```text
//! address=127.0.0.1:55441
//! period_ms=1000
//! ```
//!
//! Usage: `cargo run --bin thermometer_emulator -- thermometer_emulator.conf`

use std::error::Error;
use std::io::ErrorKind;
use std::net::UdpSocket;
use std::time::Duration;
use std::{env, fs, thread};

const DEFAULT_CONFIG_PATH: &str = "thermometer_emulator.conf";
const BASE_TEMPERATURE: f32 = 20.0;
const TEMPERATURE_AMPLITUDE: f32 = 5.0;

struct Config {
    address: String,
    period_ms: u64,
}

fn load_config(path: &str) -> Result<Config, Box<dyn Error>> {
    let content = fs::read_to_string(path)
        .map_err(|error| format!("не удалось прочитать конфиг `{path}`: {error}"))?;

    let mut address = None;
    let mut period_ms = None;

    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| format!("некорректная строка конфига: `{line}`"))?;

        match key.trim() {
            "address" => address = Some(value.trim().to_string()),
            "period_ms" => {
                let value = value.trim();
                period_ms = Some(value.parse::<u64>().map_err(|error| {
                    format!("некорректное значение `period_ms={value}`: {error}")
                })?);
            }
            other => return Err(format!("неизвестный ключ конфига `{other}`").into()),
        }
    }

    Ok(Config {
        address: address.ok_or("в конфиге отсутствует ключ `address`")?,
        period_ms: period_ms.ok_or("в конфиге отсутствует ключ `period_ms`")?,
    })
}

fn main() -> Result<(), Box<dyn Error>> {
    let config_path = env::args()
        .nth(1)
        .unwrap_or_else(|| DEFAULT_CONFIG_PATH.to_string());
    let config = load_config(&config_path)?;

    let socket = UdpSocket::bind("127.0.0.1:0")?;
    socket.set_nonblocking(true)?;
    println!(
        "Имитатор термометра отправляет температуру на {} каждые {} мс",
        config.address, config.period_ms
    );

    let mut tick = 0u32;
    loop {
        // "Arbitrary" temperature: oscillation around the base value.
        let temperature = BASE_TEMPERATURE + TEMPERATURE_AMPLITUDE * (tick as f32 * 0.3).sin();
        let payload = format!("{temperature:.1}");

        match socket.send_to(payload.as_bytes(), &config.address) {
            Ok(_) => println!("Отправлено: {payload}"),
            Err(error) if error.kind() == ErrorKind::WouldBlock => {}
            Err(error) => eprintln!("Ошибка отправки: {error}"),
        }

        tick = tick.wrapping_add(1);
        thread::sleep(Duration::from_millis(config.period_ms));
    }
}
