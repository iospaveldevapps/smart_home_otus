27.04.26 - Added HW_1  
25.05.26 - Added HW_2  
28.07.26 - Added HW_3  
30.07.26 - Added HW_5

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
