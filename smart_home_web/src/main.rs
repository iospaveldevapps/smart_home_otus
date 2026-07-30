use std::sync::{Arc, Mutex};

use smart_home::{HomeBuilder, SmartSocket, SmartThermometer};
use smart_home_web::app;

const DEFAULT_ADDR: &str = "127.0.0.1:8080";

#[tokio::main]
async fn main() {
    let addr = std::env::var("SMART_HOME_ADDR").unwrap_or_else(|_| DEFAULT_ADDR.to_string());

    // Pre-populated home so the frontend has something to show right away.
    let home = HomeBuilder::new()
        .add_room("Гостиная")
        .add_device("Розетка у дивана", SmartSocket::new(true, 120.0))
        .add_device("Термометр", SmartThermometer::new(22.5))
        .add_room("Кухня")
        .add_device("Розетка чайника", SmartSocket::new(false, 2000.0))
        .build();

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .unwrap_or_else(|error| panic!("не удалось занять адрес {addr}: {error}"));

    println!("Сервер умного дома запущен на http://{addr}");

    axum::serve(listener, app(Arc::new(Mutex::new(home))))
        .await
        .expect("ошибка работы сервера");
}
