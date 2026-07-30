//! Functional tests: talk to a running backend over HTTP and check responses.

use std::sync::{Arc, Mutex};

use reqwest::StatusCode;
use serde_json::json;

use smart_home::SmartHome;
use smart_home_web::app;

/// Starts the backend on a random port and returns its base URL.
async fn spawn_server() -> String {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("не удалось занять порт");
    let addr = listener.local_addr().expect("нет локального адреса");

    let home = Arc::new(Mutex::new(SmartHome::new()));
    tokio::spawn(async move {
        axum::serve(listener, app(home))
            .await
            .expect("ошибка работы тестового сервера");
    });

    format!("http://{addr}")
}

#[tokio::test]
async fn manages_rooms() {
    let base = spawn_server().await;
    let client = reqwest::Client::new();

    // Initially there are no rooms.
    let rooms: serde_json::Value = client
        .get(format!("{base}/rooms"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rooms["rooms"], json!([]));

    // Add two rooms.
    for name in ["Кухня", "Гостиная"] {
        let response = client
            .post(format!("{base}/rooms"))
            .json(&json!({ "name": name }))
            .send()
            .await
            .unwrap();
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    // Duplicate room is rejected.
    let response = client
        .post(format!("{base}/rooms"))
        .json(&json!({ "name": "Кухня" }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // The list is sorted and contains both rooms.
    let rooms: serde_json::Value = client
        .get(format!("{base}/rooms"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(rooms["rooms"], json!(["Гостиная", "Кухня"]));

    // Info about a specific room.
    let room: serde_json::Value = client
        .get(format!("{base}/rooms/Кухня"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(room["name"], "Кухня");
    assert_eq!(room["devices"], json!([]));

    // Remove a room; afterwards it is not found.
    let response = client
        .delete(format!("{base}/rooms/Кухня"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = client
        .get(format!("{base}/rooms/Кухня"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    let response = client
        .delete(format!("{base}/rooms/Кухня"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn manages_devices_in_room() {
    let base = spawn_server().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/rooms"))
        .json(&json!({ "name": "Кухня" }))
        .send()
        .await
        .unwrap();

    // Add a socket and a thermometer.
    let response = client
        .post(format!("{base}/rooms/Кухня/devices"))
        .json(&json!({
            "name": "Розетка",
            "type": "socket",
            "is_on": true,
            "power": 1500.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    let response = client
        .post(format!("{base}/rooms/Кухня/devices"))
        .json(&json!({
            "name": "Термометр",
            "type": "thermometer",
            "temperature": 24.5
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Duplicate device is rejected.
    let response = client
        .post(format!("{base}/rooms/Кухня/devices"))
        .json(&json!({
            "name": "Розетка",
            "type": "socket",
            "is_on": false,
            "power": 100.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CONFLICT);

    // Adding to a missing room fails with 404.
    let response = client
        .post(format!("{base}/rooms/Чулан/devices"))
        .json(&json!({
            "name": "Розетка",
            "type": "socket",
            "is_on": false,
            "power": 100.0
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);

    // Device list is sorted.
    let devices: serde_json::Value = client
        .get(format!("{base}/rooms/Кухня/devices"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(devices["devices"], json!(["Розетка", "Термометр"]));

    // Info about a specific device includes its report.
    let device: serde_json::Value = client
        .get(format!("{base}/rooms/Кухня/devices/Розетка"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(device["name"], "Розетка");
    assert_eq!(device["room"], "Кухня");
    assert_eq!(device["kind"], "socket");
    let report = device["report"].as_str().unwrap();
    assert!(report.contains("Умная розетка"), "report: {report}");
    assert!(report.contains("1500"), "report: {report}");

    let device: serde_json::Value = client
        .get(format!("{base}/rooms/Кухня/devices/Термометр"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(device["kind"], "thermometer");
    assert!(device["report"].as_str().unwrap().contains("24.5"));

    // Missing device yields 404 with an error body.
    let response = client
        .get(format!("{base}/rooms/Кухня/devices/Утюг"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("Утюг"));

    // Remove a device; afterwards it is not found.
    let response = client
        .delete(format!("{base}/rooms/Кухня/devices/Розетка"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NO_CONTENT);

    let response = client
        .get(format!("{base}/rooms/Кухня/devices/Розетка"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn returns_home_report() {
    let base = spawn_server().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{base}/rooms"))
        .json(&json!({ "name": "Спальня" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{base}/rooms/Спальня/devices"))
        .json(&json!({
            "name": "Ночник",
            "type": "socket",
            "is_on": true,
            "power": 40.0
        }))
        .send()
        .await
        .unwrap();

    let report: serde_json::Value = client
        .get(format!("{base}/report"))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let text = report["report"].as_str().unwrap();
    assert!(text.contains("Отчёт по умному дому"), "report: {text}");
    assert!(text.contains("Комната `Спальня`"), "report: {text}");
    assert!(text.contains("Устройство `Ночник`"), "report: {text}");
    assert!(text.contains("Умная розетка"), "report: {text}");
}
