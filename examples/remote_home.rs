//! Smart home example with devices talking to the emulators.
//!
//! Start the emulators in separate terminals first:
//!
//! ```text
//! cargo run --bin socket_emulator -- 127.0.0.1:55331
//! cargo run --bin thermometer_emulator -- thermometer_emulator.conf
//! ```
//!
//! Then run: `cargo run --example remote_home`.
//!
//! With the emulators running, a full home state report is printed.
//! If a device fails to fetch its data, the report contains an error message.

use std::thread;
use std::time::Duration;

use smart_home::{Report, Room, SmartHome, SmartSocket, SmartThermometer};

/// Address of the TCP socket emulator.
const SOCKET_ADDR: &str = "127.0.0.1:55331";
/// Address to receive UDP packets on — matches `address` in `thermometer_emulator.conf`.
const THERMOMETER_ADDR: &str = "127.0.0.1:55441";

fn main() {
    let mut living_room = Room::new();

    match SmartSocket::connect(SOCKET_ADDR) {
        Ok(mut socket) => {
            if let Err(error) = socket.turn_on() {
                println!("Не удалось включить розетку: {error}");
            }
            living_room.add_device("socket", socket);
        }
        Err(error) => {
            println!("Не удалось подключиться к розетке ({SOCKET_ADDR}): {error}");
        }
    }

    match SmartThermometer::listen(THERMOMETER_ADDR) {
        Ok(thermometer) => {
            living_room.add_device("thermometer", thermometer);
        }
        Err(error) => {
            println!("Не удалось запустить приём температуры ({THERMOMETER_ADDR}): {error}");
        }
    }

    println!("Ожидание данных от имитаторов...");
    thread::sleep(Duration::from_secs(2));

    let mut home = SmartHome::new();
    home.add_room("living room", living_room);

    home.print_report();
}
