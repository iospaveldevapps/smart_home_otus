use smart_home::{Report, SmartDevice, SmartHome, SmartSocket, SmartThermometer, smart_room};

fn main() {
    let living_room = smart_room!(
        "thermometer" => SmartThermometer::new(23.5),
        "socket" => SmartSocket::new(true, 150.0),
    );

    let mut kitchen = smart_room!(
        "thermometer" => SmartThermometer::new(21.0),
        "socket" => SmartSocket::new(false, 200.0),
    );
    kitchen.add_device("kettle socket", SmartSocket::new(true, 750.0));
    kitchen.remove_device("socket");

    let mut home = SmartHome::new();
    home.add_room("living room", living_room);
    home.add_room("kitchen", kitchen);

    if let Ok(SmartDevice::Socket(socket)) = home.get_smart_device_mut("living room", "socket") {
        socket.turn_off();
    }

    home.print_report();

    home.remove_room("kitchen");
    home.add_room(
        "bedroom",
        smart_room!(("thermometer", SmartThermometer::new(19.5))),
    );

    println!();
    home.print_report();

    match home.get_smart_device("kitchen", "socket") {
        Ok(device) => println!("Найдено устройство: {}", device.report()),
        Err(error) => println!("Ошибка поиска: {error}"),
    }
}
