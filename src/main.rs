use smart_home::{Room, SmartDevice, SmartHome, SmartSocket, SmartThermometer};

fn main() {
    let living_room = Room::new(vec![
        SmartDevice::Thermometer(SmartThermometer::new(23.5)),
        SmartDevice::Socket(SmartSocket::new(true, 150.0)),
    ]);

    let kitchen = Room::new(vec![
        SmartDevice::Thermometer(SmartThermometer::new(21.0)),
        SmartDevice::Socket(SmartSocket::new(false, 200.0)),
    ]);

    let mut home = SmartHome::new(vec![living_room, kitchen]);

    if let SmartDevice::Socket(socket) = home.get_room_mut(0).get_device_mut(1) {
        socket.turn_off();
    }

    home.print_report();

    // Это вызовет panic!
    // home.get_room(100);
}
