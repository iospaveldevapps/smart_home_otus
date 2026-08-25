use gloo_net::http::{Request, Response};
use serde::{Deserialize, Serialize};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{HtmlFormElement, HtmlInputElement, HtmlSelectElement, SubmitEvent};
use yew::prelude::*;

const API: &str = "http://127.0.0.1:8080";

#[derive(Clone, PartialEq, Deserialize)]
struct RoomList {
    rooms: Vec<String>,
}

#[derive(Clone, PartialEq, Deserialize)]
struct RoomInfo {
    name: String,
    devices: Vec<String>,
}

#[derive(Clone, PartialEq, Deserialize)]
struct DeviceInfo {
    name: String,
    room: String,
    kind: String,
    report: String,
}

#[derive(Deserialize)]
struct HomeReport {
    report: String,
}

#[derive(Deserialize)]
struct ApiError {
    error: String,
}

#[derive(Serialize)]
struct NewRoom<'a> {
    name: &'a str,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum NewDevice {
    Socket {
        name: String,
        is_on: bool,
        power: f32,
    },
    Thermometer {
        name: String,
        temperature: f32,
    },
}

fn encode(value: &str) -> String {
    js_sys::encode_uri_component(value)
        .as_string()
        .unwrap_or_default()
}

async fn api_error(response: Response) -> String {
    let status = response.status();
    response
        .json::<ApiError>()
        .await
        .map(|body| body.error)
        .unwrap_or_else(|_| format!("Сервер вернул статус {status}"))
}

fn show_error(error: &UseStateHandle<Option<String>>, message: impl ToString) {
    error.set(Some(message.to_string()));
}

#[function_component(App)]
fn app() -> Html {
    let rooms = use_state(Vec::<String>::new);
    let room = use_state(|| None::<RoomInfo>);
    let device = use_state(|| None::<DeviceInfo>);
    let report = use_state(|| None::<String>);
    let error = use_state(|| None::<String>);
    let loading = use_state(|| true);

    let refresh_rooms = {
        let rooms = rooms.clone();
        let error = error.clone();
        let loading = loading.clone();
        Callback::from(move |_| {
            let (rooms, error, loading) = (rooms.clone(), error.clone(), loading.clone());
            spawn_local(async move {
                loading.set(true);
                match Request::get(&format!("{API}/rooms")).send().await {
                    Ok(response) if response.ok() => match response.json::<RoomList>().await {
                        Ok(data) => {
                            rooms.set(data.rooms);
                            error.set(None);
                        }
                        Err(err) => show_error(&error, format!("Некорректный ответ: {err}")),
                    },
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, format!("Backend недоступен: {err}")),
                }
                loading.set(false);
            });
        })
    };

    {
        let refresh_rooms = refresh_rooms.clone();
        use_effect_with((), move |_| refresh_rooms.emit(()));
    }

    let open_room = {
        let room = room.clone();
        let device = device.clone();
        let error = error.clone();
        Callback::from(move |name: String| {
            let (room, device, error) = (room.clone(), device.clone(), error.clone());
            spawn_local(async move {
                let url = format!("{API}/rooms/{}", encode(&name));
                match Request::get(&url).send().await {
                    Ok(response) if response.ok() => match response.json::<RoomInfo>().await {
                        Ok(info) => {
                            room.set(Some(info));
                            device.set(None);
                            error.set(None);
                        }
                        Err(err) => show_error(&error, format!("Некорректный ответ: {err}")),
                    },
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let add_room = {
        let refresh_rooms = refresh_rooms.clone();
        let error = error.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let form = event.target_unchecked_into::<HtmlFormElement>();
            let input = form
                .query_selector("input")
                .ok()
                .flatten()
                .expect("room input")
                .unchecked_into::<HtmlInputElement>();
            let name = input.value().trim().to_string();
            if name.is_empty() {
                return show_error(&error, "Введите название комнаты");
            }
            let (refresh_rooms, error) = (refresh_rooms.clone(), error.clone());
            spawn_local(async move {
                match Request::post(&format!("{API}/rooms")).json(&NewRoom { name: &name }) {
                    Ok(request) => match request.send().await {
                        Ok(response) if response.ok() => {
                            input.set_value("");
                            refresh_rooms.emit(());
                        }
                        Ok(response) => show_error(&error, api_error(response).await),
                        Err(err) => show_error(&error, err),
                    },
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let remove_room = {
        let refresh_rooms = refresh_rooms.clone();
        let room = room.clone();
        let error = error.clone();
        Callback::from(move |name: String| {
            let (refresh_rooms, room, error) = (refresh_rooms.clone(), room.clone(), error.clone());
            spawn_local(async move {
                match Request::delete(&format!("{API}/rooms/{}", encode(&name)))
                    .send()
                    .await
                {
                    Ok(response) if response.ok() => {
                        room.set(None);
                        refresh_rooms.emit(());
                    }
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let open_device = {
        let device = device.clone();
        let error = error.clone();
        Callback::from(move |(room_name, name): (String, String)| {
            let (device, error) = (device.clone(), error.clone());
            spawn_local(async move {
                let url = format!(
                    "{API}/rooms/{}/devices/{}",
                    encode(&room_name),
                    encode(&name)
                );
                match Request::get(&url).send().await {
                    Ok(response) if response.ok() => match response.json::<DeviceInfo>().await {
                        Ok(info) => device.set(Some(info)),
                        Err(err) => show_error(&error, err),
                    },
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let add_device = {
        let room_name = room.as_ref().map(|info| info.name.clone());
        let open_room = open_room.clone();
        let error = error.clone();
        Callback::from(move |event: SubmitEvent| {
            event.prevent_default();
            let Some(room_name) = room_name.clone() else {
                return;
            };
            let form = event.target_unchecked_into::<HtmlFormElement>();
            let element = |name| form.elements().named_item(name).expect("form element");
            let name = element("device-name")
                .unchecked_into::<HtmlInputElement>()
                .value();
            let kind = element("device-kind")
                .unchecked_into::<HtmlSelectElement>()
                .value();
            let value = element("device-value")
                .unchecked_into::<HtmlInputElement>()
                .value();
            if name.trim().is_empty() {
                return show_error(&error, "Введите название устройства");
            }
            let Ok(value) = value.parse::<f32>() else {
                return show_error(&error, "Введите мощность или температуру числом");
            };
            let body = if kind == "socket" {
                NewDevice::Socket {
                    name: name.trim().to_string(),
                    is_on: false,
                    power: value,
                }
            } else {
                NewDevice::Thermometer {
                    name: name.trim().to_string(),
                    temperature: value,
                }
            };
            let (open_room, error) = (open_room.clone(), error.clone());
            spawn_local(async move {
                let url = format!("{API}/rooms/{}/devices", encode(&room_name));
                match Request::post(&url).json(&body) {
                    Ok(request) => match request.send().await {
                        Ok(response) if response.ok() => {
                            form.reset();
                            open_room.emit(room_name);
                        }
                        Ok(response) => show_error(&error, api_error(response).await),
                        Err(err) => show_error(&error, err),
                    },
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let remove_device = {
        let open_room = open_room.clone();
        let device = device.clone();
        let error = error.clone();
        Callback::from(move |(room_name, name): (String, String)| {
            let (open_room, device, error) = (open_room.clone(), device.clone(), error.clone());
            spawn_local(async move {
                let url = format!(
                    "{API}/rooms/{}/devices/{}",
                    encode(&room_name),
                    encode(&name)
                );
                match Request::delete(&url).send().await {
                    Ok(response) if response.ok() => {
                        device.set(None);
                        open_room.emit(room_name);
                    }
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    let load_report = {
        let report = report.clone();
        let error = error.clone();
        Callback::from(move |_| {
            let (report, error) = (report.clone(), error.clone());
            spawn_local(async move {
                match Request::get(&format!("{API}/report")).send().await {
                    Ok(response) if response.ok() => match response.json::<HomeReport>().await {
                        Ok(data) => report.set(Some(data.report)),
                        Err(err) => show_error(&error, err),
                    },
                    Ok(response) => show_error(&error, api_error(response).await),
                    Err(err) => show_error(&error, err),
                }
            });
        })
    };

    html! {
        <main class="shell">
            <header>
                <div><span class="eyebrow">{"RUST · YEW · WEBASSEMBLY"}</span><h1>{"Умный дом"}</h1></div>
                <button class="primary" onclick={load_report}>{"Получить отчёт"}</button>
            </header>
            if let Some(message) = &*error {
                <div class="alert"><span>{message}</span><button onclick={{let error=error.clone(); Callback::from(move |_| error.set(None))}}>{"×"}</button></div>
            }
            <section class="layout">
                <aside class="panel rooms">
                    <div class="panel-title"><h2>{"Комнаты"}</h2><span>{rooms.len()}</span></div>
                    if *loading { <p class="muted">{"Подключение…"}</p> }
                    <nav>{for rooms.iter().map(|name| {
                        let open = {let cb=open_room.clone(); let name=name.clone(); Callback::from(move |_| cb.emit(name.clone()))};
                        let remove = {let cb=remove_room.clone(); let name=name.clone(); Callback::from(move |e: MouseEvent| {e.stop_propagation(); cb.emit(name.clone())})};
                        let active=room.as_ref().is_some_and(|item| item.name == *name);
                        html!{<button class={classes!("room-item",active.then_some("active"))} onclick={open}><span>{name}</span><i onclick={remove}>{"×"}</i></button>}
                    })}</nav>
                    <form class="inline-form" onsubmit={add_room}><input placeholder="Новая комната"/><button type="submit">{"+"}</button></form>
                </aside>
                <section class="panel content">
                if let Some(info)=&*room {
                    <div class="content-heading"><div><span class="eyebrow">{"КОМНАТА"}</span><h2>{&info.name}</h2></div><span class="count">{format!("{} устройств",info.devices.len())}</span></div>
                    <div class="device-grid">{for info.devices.iter().map(|name| {
                        let room_name=info.name.clone(); let device_name=name.clone();
                        let open={let cb=open_device.clone(); let r=room_name.clone(); let d=device_name.clone(); Callback::from(move |_| cb.emit((r.clone(),d.clone())))};
                        let remove={let cb=remove_device.clone(); Callback::from(move |e: MouseEvent| {e.stop_propagation();cb.emit((room_name.clone(),device_name.clone()))})};
                        html!{<article class="device-card" onclick={open}><div class="device-icon">{"⌁"}</div><h3>{name}</h3><p>{"Открыть информацию"}</p><button class="delete" onclick={remove}>{"Удалить"}</button></article>}
                    })}</div>
                    <form class="device-form" onsubmit={add_device}><h3>{"Добавить устройство"}</h3><input name="device-name" placeholder="Название"/><select name="device-kind"><option value="socket">{"Розетка"}</option><option value="thermometer">{"Термометр"}</option></select><input name="device-value" type="number" step="any" placeholder="Мощность / температура"/><button class="primary" type="submit">{"Добавить"}</button></form>
                } else {
                    <div class="empty"><div class="home-mark">{"⌂"}</div><h2>{"Выберите комнату"}</h2><p>{"Здесь появятся устройства и элементы управления."}</p></div>
                }
                </section>
            </section>
            if let Some(info)=&*device {
                <div class="modal-backdrop" onclick={{let device=device.clone();Callback::from(move |_| device.set(None))}}><article class="modal" onclick={Callback::from(|e: MouseEvent|e.stop_propagation())}><span class="eyebrow">{if info.kind=="socket"{"УМНАЯ РОЗЕТКА"}else{"ТЕРМОМЕТР"}}</span><h2>{&info.name}</h2><p class="muted">{format!("Комната: {}",info.room)}</p><div class="reading">{&info.report}</div><button class="primary" onclick={{let device=device.clone();Callback::from(move |_|device.set(None))}}>{"Закрыть"}</button></article></div>
            }
            if let Some(text)=&*report {
                <div class="modal-backdrop" onclick={{let report=report.clone();Callback::from(move |_|report.set(None))}}><article class="modal report" onclick={Callback::from(|e: MouseEvent|e.stop_propagation())}><span class="eyebrow">{"СОСТОЯНИЕ ДОМА"}</span><h2>{"Отчёт"}</h2><pre>{text}</pre><button class="primary" onclick={{let report=report.clone();Callback::from(move |_|report.set(None))}}>{"Закрыть"}</button></article></div>
            }
        </main>
    }
}

fn main() {
    yew::Renderer::<App>::new().render();
}
