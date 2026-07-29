//! Composite pattern demo: `cargo run --example reporter`.
//!
//! `Reporter` collects heterogeneous objects via static polymorphism
//! (each `add` produces a new type) and prints a combined report.

use smart_home::{Reporter, SmartSocket, SmartThermometer, smart_room};

fn main() {
    let room = smart_room!(
        "socket" => SmartSocket::new(true, 100.0),
        "thermometer" => SmartThermometer::new(22.5),
    );
    let socket = SmartSocket::new(false, 200.0);
    let thermometer = SmartThermometer::new(21.0);

    Reporter::new()
        .add(&room)
        .add(&socket)
        .add(&thermometer)
        .report();
}
