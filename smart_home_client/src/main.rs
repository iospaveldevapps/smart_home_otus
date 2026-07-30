//! Console frontend for the smart home web service.
//!
//! Usage: `smart_home_client [base_url]`, the address can also be set
//! via the `SMART_HOME_URL` environment variable
//! (default: `http://127.0.0.1:8080`).

use std::io::{self, BufRead, Write};
use std::process::ExitCode;

use serde::Deserialize;
use serde_json::json;

const DEFAULT_URL: &str = "http://127.0.0.1:8080";

#[derive(Deserialize)]
struct RoomListResponse {
    rooms: Vec<String>,
}

#[derive(Deserialize)]
struct DeviceListResponse {
    devices: Vec<String>,
}

#[derive(Deserialize)]
struct DeviceResponse {
    name: String,
    room: String,
    kind: String,
    report: String,
}

#[derive(Deserialize)]
struct ReportResponse {
    report: String,
}

#[derive(Deserialize)]
struct ErrorBody {
    error: String,
}

/// Client-side error: transport failure or an API error response.
#[derive(Debug)]
enum ClientError {
    Http(reqwest::Error),
    Api(String),
}

impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClientError::Http(error) => write!(f, "ошибка соединения: {error}"),
            ClientError::Api(message) => write!(f, "{message}"),
        }
    }
}

impl From<reqwest::Error> for ClientError {
    fn from(error: reqwest::Error) -> Self {
        Self::Http(error)
    }
}

/// Thin wrapper over the REST API of the backend.
struct Api {
    base_url: String,
    client: reqwest::blocking::Client,
}

impl Api {
    fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base_url)
    }

    /// Turns an error response `{"error": "..."}` into [`ClientError::Api`].
    fn check(
        response: reqwest::blocking::Response,
    ) -> Result<reqwest::blocking::Response, ClientError> {
        if response.status().is_success() {
            return Ok(response);
        }

        let status = response.status();
        let message = response
            .json::<ErrorBody>()
            .map(|body| body.error)
            .unwrap_or_else(|_| format!("сервер вернул статус {status}"));
        Err(ClientError::Api(message))
    }

    fn list_rooms(&self) -> Result<Vec<String>, ClientError> {
        let response = Self::check(self.client.get(self.url("/rooms")).send()?)?;
        Ok(response.json::<RoomListResponse>()?.rooms)
    }

    fn add_room(&self, name: &str) -> Result<(), ClientError> {
        Self::check(
            self.client
                .post(self.url("/rooms"))
                .json(&json!({ "name": name }))
                .send()?,
        )?;
        Ok(())
    }

    fn remove_room(&self, name: &str) -> Result<(), ClientError> {
        Self::check(
            self.client
                .delete(self.url(&format!("/rooms/{name}")))
                .send()?,
        )?;
        Ok(())
    }

    fn list_devices(&self, room: &str) -> Result<Vec<String>, ClientError> {
        let response = Self::check(
            self.client
                .get(self.url(&format!("/rooms/{room}/devices")))
                .send()?,
        )?;
        Ok(response.json::<DeviceListResponse>()?.devices)
    }

    fn add_device(&self, room: &str, request: serde_json::Value) -> Result<(), ClientError> {
        Self::check(
            self.client
                .post(self.url(&format!("/rooms/{room}/devices")))
                .json(&request)
                .send()?,
        )?;
        Ok(())
    }

    fn remove_device(&self, room: &str, device: &str) -> Result<(), ClientError> {
        Self::check(
            self.client
                .delete(self.url(&format!("/rooms/{room}/devices/{device}")))
                .send()?,
        )?;
        Ok(())
    }

    fn get_device(&self, room: &str, device: &str) -> Result<DeviceResponse, ClientError> {
        let response = Self::check(
            self.client
                .get(self.url(&format!("/rooms/{room}/devices/{device}")))
                .send()?,
        )?;
        Ok(response.json()?)
    }

    fn home_report(&self) -> Result<String, ClientError> {
        let response = Self::check(self.client.get(self.url("/report")).send()?)?;
        Ok(response.json::<ReportResponse>()?.report)
    }
}

/// Reads a line from stdin with a prompt; `None` means end of input.
fn prompt(text: &str) -> Option<String> {
    print!("{text}");
    io::stdout().flush().ok();

    let mut line = String::new();
    match io::stdin().lock().read_line(&mut line) {
        Ok(0) => None,
        Ok(_) => Some(line.trim().to_string()),
        Err(_) => None,
    }
}

fn print_result(result: Result<(), ClientError>, success: &str) {
    match result {
        Ok(()) => println!("{success}"),
        Err(error) => println!("Ошибка: {error}"),
    }
}

fn show_rooms(api: &Api) {
    match api.list_rooms() {
        Ok(rooms) if rooms.is_empty() => println!("В доме пока нет комнат."),
        Ok(rooms) => {
            println!("Комнаты:");
            for room in rooms {
                println!("  - {room}");
            }
        }
        Err(error) => println!("Ошибка: {error}"),
    }
}

fn add_device_dialog(api: &Api, room: &str) {
    let Some(name) = prompt("Название устройства: ") else {
        return;
    };
    let Some(kind) = prompt("Тип устройства (1 — розетка, 2 — термометр): ")
    else {
        return;
    };

    let request = match kind.as_str() {
        "1" => {
            let is_on = matches!(prompt("Розетка включена? (y/n): ").as_deref(), Some("y"));
            let power = prompt("Мощность, Вт: ")
                .and_then(|text| text.parse::<f32>().ok())
                .unwrap_or(0.0);
            json!({ "name": name, "type": "socket", "is_on": is_on, "power": power })
        }
        "2" => {
            let temperature = prompt("Температура, °C: ")
                .and_then(|text| text.parse::<f32>().ok())
                .unwrap_or(0.0);
            json!({ "name": name, "type": "thermometer", "temperature": temperature })
        }
        _ => {
            println!("Неизвестный тип устройства.");
            return;
        }
    };

    print_result(api.add_device(room, request), "Устройство добавлено.");
}

fn show_device(api: &Api, room: &str, device: &str) {
    match api.get_device(room, device) {
        Ok(info) => {
            let kind = match info.kind.as_str() {
                "socket" => "розетка",
                "thermometer" => "термометр",
                other => other,
            };
            println!(
                "Устройство `{}` в комнате `{}` ({kind}):",
                info.name, info.room
            );
            println!("  {}", info.report);
        }
        Err(error) => println!("Ошибка: {error}"),
    }
}

/// Menu for a specific room; returns when the user goes back.
fn room_menu(api: &Api, room: &str) {
    loop {
        println!();
        println!("=== Комната `{room}` ===");
        println!("1 — список устройств");
        println!("2 — информация об устройстве");
        println!("3 — добавить устройство");
        println!("4 — удалить устройство");
        println!("0 — назад");

        let Some(choice) = prompt("> ") else { return };

        match choice.as_str() {
            "1" => match api.list_devices(room) {
                Ok(devices) if devices.is_empty() => println!("В комнате нет устройств."),
                Ok(devices) => {
                    println!("Устройства:");
                    for device in devices {
                        println!("  - {device}");
                    }
                }
                Err(error) => println!("Ошибка: {error}"),
            },
            "2" => {
                if let Some(device) = prompt("Название устройства: ") {
                    show_device(api, room, &device);
                }
            }
            "3" => add_device_dialog(api, room),
            "4" => {
                if let Some(device) = prompt("Название устройства: ") {
                    print_result(api.remove_device(room, &device), "Устройство удалено.");
                }
            }
            "0" => return,
            _ => println!("Неизвестная команда."),
        }
    }
}

fn main_menu(api: &Api) {
    loop {
        println!();
        println!("=== Умный дом ===");
        println!("1 — список комнат");
        println!("2 — перейти в комнату");
        println!("3 — добавить комнату");
        println!("4 — удалить комнату");
        println!("5 — отчёт о доме");
        println!("0 — выход");

        let Some(choice) = prompt("> ") else { return };

        match choice.as_str() {
            "1" => show_rooms(api),
            "2" => {
                if let Some(room) = prompt("Название комнаты: ") {
                    // Validate the room exists before entering its menu.
                    match api.list_devices(&room) {
                        Ok(_) => room_menu(api, &room),
                        Err(error) => println!("Ошибка: {error}"),
                    }
                }
            }
            "3" => {
                if let Some(name) = prompt("Название комнаты: ") {
                    print_result(api.add_room(&name), "Комната добавлена.");
                }
            }
            "4" => {
                if let Some(name) = prompt("Название комнаты: ") {
                    print_result(api.remove_room(&name), "Комната удалена.");
                }
            }
            "5" => match api.home_report() {
                Ok(report) => println!("{report}"),
                Err(error) => println!("Ошибка: {error}"),
            },
            "0" => return,
            _ => println!("Неизвестная команда."),
        }
    }
}

fn main() -> ExitCode {
    let base_url = std::env::args()
        .nth(1)
        .or_else(|| std::env::var("SMART_HOME_URL").ok())
        .unwrap_or_else(|| DEFAULT_URL.to_string());

    let api = Api::new(base_url.trim_end_matches('/').to_string());

    // Check the backend is reachable before showing the menu.
    if let Err(error) = api.list_rooms() {
        eprintln!(
            "Не удалось подключиться к серверу {}: {error}",
            api.base_url
        );
        eprintln!("Запустите backend: cargo run -p smart_home_web");
        return ExitCode::FAILURE;
    }

    println!("Подключено к серверу {}", api.base_url);
    main_menu(&api);
    ExitCode::SUCCESS
}
