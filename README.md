27.04.26 - Added HW_1  
25.05.26 - Added HW_2  
28.07.26 - Added HW_3  
30.07.26 - Added HW_5  
30.07.26 - Added HW_6

## HW_6: Smart home web service

Workspace packages:

- `smart_home_web` — backend: REST API (axum) over the smart home library;
- `smart_home_client` — browser frontend written in Rust with Yew and compiled to WebAssembly.

Run the backend (address configurable via `SMART_HOME_ADDR`, default `127.0.0.1:8080`):

```sh
cargo run -p smart_home_web
```

Install the WebAssembly target and Trunk once:

```sh
rustup target add wasm32-unknown-unknown
cargo install trunk
```

Run the frontend in another terminal:

```sh
cd smart_home_client
trunk serve
```

Open `http://127.0.0.1:8081`. The frontend connects to the REST backend at
`http://127.0.0.1:8080` and supports rooms, devices, and the home report.

REST API:

| Method   | Path                              | Description                        |
| -------- | --------------------------------- | ---------------------------------- |
| `GET`    | `/rooms`                          | list rooms                         |
| `POST`   | `/rooms`                          | add a room `{"name": ...}`         |
| `GET`    | `/rooms/{room}`                   | room info                          |
| `DELETE` | `/rooms/{room}`                   | remove a room                      |
| `GET`    | `/rooms/{room}/devices`           | list devices in a room             |
| `POST`   | `/rooms/{room}/devices`           | add a device                       |
| `GET`    | `/rooms/{room}/devices/{device}`  | device info                        |
| `DELETE` | `/rooms/{room}/devices/{device}`  | remove a device                    |
| `GET`    | `/report`                         | full home report                   |

A device is described as `{"name": ..., "type": "socket", "is_on": bool, "power": f32}`
or `{"name": ..., "type": "thermometer", "temperature": f32}`.

Functional tests (`smart_home_web/tests/api.rs`) start the backend on a random
port and talk to it over HTTP: `cargo test -p smart_home_web`.

## HW_5: C-style smart socket

Workspace packages:

- `smart_socket` — socket library (on/off + power query), builds three artifacts: `rlib`, `staticlib` and `cdylib` with C ABI;
- `socket_static_demo` — application that links the library statically and calls its C ABI functions;
- `socket_dynamic_demo` — application that loads the `cdylib` at runtime via `libloading`.

```sh
cargo build --workspace
cargo run -p socket_static_demo
cargo run -p socket_dynamic_demo
```

`socket_dynamic_demo` looks for the library next to its own executable (`target/<profile>/`); a custom path can be passed as the first argument.
