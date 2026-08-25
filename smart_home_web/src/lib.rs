//! REST backend for the smart home library.
//!
//! Endpoints:
//! - `GET    /rooms` — list rooms
//! - `POST   /rooms` — add a room
//! - `GET    /rooms/{room}` — room info
//! - `DELETE /rooms/{room}` — remove a room
//! - `GET    /rooms/{room}/devices` — list devices in a room
//! - `POST   /rooms/{room}/devices` — add a device to a room
//! - `GET    /rooms/{room}/devices/{device}` — device info
//! - `DELETE /rooms/{room}/devices/{device}` — remove a device
//! - `GET    /report` — full home report

use std::sync::{Arc, Mutex};

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};

use smart_home::{Report, Room, SmartDevice, SmartHome, SmartSocket, SmartThermometer};

/// Home shared between request handlers.
pub type SharedHome = Arc<Mutex<SmartHome>>;

/// Builds the application router around a shared home.
pub fn app(home: SharedHome) -> Router {
    Router::new()
        .route("/rooms", get(list_rooms).post(add_room))
        .route("/rooms/{room}", get(get_room).delete(remove_room))
        .route("/rooms/{room}/devices", get(list_devices).post(add_device))
        .route(
            "/rooms/{room}/devices/{device}",
            get(get_device).delete(remove_device),
        )
        .route("/report", get(home_report))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(home)
}

/// API error with an HTTP status and a JSON body `{"error": "..."}`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

impl ApiError {
    fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    fn conflict(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            message: message.into(),
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = Json(ErrorBody {
            error: self.message,
        });
        (self.status, body).into_response()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ErrorBody {
    pub error: String,
}

/// Device description in API requests and responses.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum DeviceSpec {
    Socket { is_on: bool, power: f32 },
    Thermometer { temperature: f32 },
}

impl From<DeviceSpec> for SmartDevice {
    fn from(spec: DeviceSpec) -> Self {
        match spec {
            DeviceSpec::Socket { is_on, power } => SmartSocket::new(is_on, power).into(),
            DeviceSpec::Thermometer { temperature } => SmartThermometer::new(temperature).into(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct RoomListResponse {
    pub rooms: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct RoomRequest {
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct RoomResponse {
    pub name: String,
    pub devices: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceListResponse {
    pub devices: Vec<String>,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceRequest {
    pub name: String,
    #[serde(flatten)]
    pub spec: DeviceSpec,
}

#[derive(Serialize, Deserialize)]
pub struct DeviceResponse {
    pub name: String,
    pub room: String,
    pub kind: String,
    pub report: String,
}

#[derive(Serialize, Deserialize)]
pub struct ReportResponse {
    pub report: String,
}

fn room_not_found(room: &str) -> ApiError {
    ApiError::not_found(format!("Комната `{room}` не найдена"))
}

fn device_not_found(room: &str, device: &str) -> ApiError {
    ApiError::not_found(format!(
        "Устройство `{device}` в комнате `{room}` не найдено"
    ))
}

async fn list_rooms(State(home): State<SharedHome>) -> Json<RoomListResponse> {
    let home = home.lock().unwrap();
    Json(RoomListResponse {
        rooms: home.room_names(),
    })
}

async fn add_room(
    State(home): State<SharedHome>,
    Json(request): Json<RoomRequest>,
) -> Result<StatusCode, ApiError> {
    let mut home = home.lock().unwrap();

    if home.get_room(&request.name).is_some() {
        return Err(ApiError::conflict(format!(
            "Комната `{}` уже существует",
            request.name
        )));
    }

    home.add_room(request.name, Room::new());
    Ok(StatusCode::CREATED)
}

async fn get_room(
    State(home): State<SharedHome>,
    Path(room_name): Path<String>,
) -> Result<Json<RoomResponse>, ApiError> {
    let home = home.lock().unwrap();
    let room = home
        .get_room(&room_name)
        .ok_or_else(|| room_not_found(&room_name))?;

    Ok(Json(RoomResponse {
        name: room_name,
        devices: room.device_names(),
    }))
}

async fn remove_room(
    State(home): State<SharedHome>,
    Path(room_name): Path<String>,
) -> Result<StatusCode, ApiError> {
    let mut home = home.lock().unwrap();

    home.remove_room(&room_name)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or_else(|| room_not_found(&room_name))
}

async fn list_devices(
    State(home): State<SharedHome>,
    Path(room_name): Path<String>,
) -> Result<Json<DeviceListResponse>, ApiError> {
    let home = home.lock().unwrap();
    let room = home
        .get_room(&room_name)
        .ok_or_else(|| room_not_found(&room_name))?;

    Ok(Json(DeviceListResponse {
        devices: room.device_names(),
    }))
}

async fn add_device(
    State(home): State<SharedHome>,
    Path(room_name): Path<String>,
    Json(request): Json<DeviceRequest>,
) -> Result<StatusCode, ApiError> {
    let mut home = home.lock().unwrap();
    let room = home
        .get_room_mut(&room_name)
        .ok_or_else(|| room_not_found(&room_name))?;

    if room.get_device(&request.name).is_some() {
        return Err(ApiError::conflict(format!(
            "Устройство `{}` в комнате `{room_name}` уже существует",
            request.name
        )));
    }

    room.add_device(request.name, SmartDevice::from(request.spec));
    Ok(StatusCode::CREATED)
}

async fn get_device(
    State(home): State<SharedHome>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<Json<DeviceResponse>, ApiError> {
    let home = home.lock().unwrap();
    let device = home
        .get_smart_device(&room_name, &device_name)
        .map_err(|_| match home.get_room(&room_name) {
            Some(_) => device_not_found(&room_name, &device_name),
            None => room_not_found(&room_name),
        })?;

    let kind = match device {
        SmartDevice::Socket(_) => "socket",
        SmartDevice::Thermometer(_) => "thermometer",
    };

    Ok(Json(DeviceResponse {
        name: device_name,
        room: room_name,
        kind: kind.to_string(),
        report: device.report(),
    }))
}

async fn remove_device(
    State(home): State<SharedHome>,
    Path((room_name, device_name)): Path<(String, String)>,
) -> Result<StatusCode, ApiError> {
    let mut home = home.lock().unwrap();
    let room = home
        .get_room_mut(&room_name)
        .ok_or_else(|| room_not_found(&room_name))?;

    room.remove_device(&device_name)
        .map(|_| StatusCode::NO_CONTENT)
        .ok_or_else(|| device_not_found(&room_name, &device_name))
}

async fn home_report(State(home): State<SharedHome>) -> Json<ReportResponse> {
    let home = home.lock().unwrap();
    Json(ReportResponse {
        report: home.report(),
    })
}
