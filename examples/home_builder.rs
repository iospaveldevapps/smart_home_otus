//! Builder pattern demo: `cargo run --example home_builder`.
//!
//! The builder forbids adding devices before the first room —
//! this is enforced at compile time (typestate pattern).

use smart_home::{HomeBuilder, Report, SmartSocket, SmartThermometer};

fn main() {
    let home = HomeBuilder::new()
        .add_room("гостиная")
        .add_device("socket_1", SmartSocket::new(true, 150.0))
        .add_device("socket_2", SmartSocket::new(false, 200.0))
        .add_device("thermo_1", SmartThermometer::new(23.5))
        .add_room("спальня")
        .add_device("socket_3", SmartSocket::new(true, 60.0))
        .add_device("thermo_2", SmartThermometer::new(19.0))
        .build();

    home.print_report();
}
