//! Demonstrates the smart socket driven through its C ABI while the library
//! is loaded dynamically at runtime with `libloading`: the binary has no
//! link-time dependency on `smart_socket` and resolves the `smart_socket_*`
//! symbols from the `cdylib` artifact.

use std::env;
use std::ffi::c_void;
use std::path::PathBuf;
use std::process::ExitCode;

use libloading::Library;

/// Opaque handle to a socket living inside the loaded library.
type SocketHandle = *mut c_void;

fn main() -> ExitCode {
    let path = library_path();

    // SAFETY: we load our own `smart_socket` library, which has no
    // initialisation side effects.
    let library = match unsafe { Library::new(&path) } {
        Ok(library) => library,
        Err(error) => {
            eprintln!(
                "Не удалось загрузить библиотеку `{}`: {error}",
                path.display()
            );
            eprintln!(
                "Соберите workspace командой `cargo build --workspace` \
                 или передайте путь к библиотеке первым аргументом."
            );
            return ExitCode::FAILURE;
        }
    };

    match run(&library) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("Ошибка работы с библиотекой: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Path to the socket library: the first CLI argument, or the `cdylib`
/// lying next to this executable in `target/<profile>` after a workspace
/// build.
fn library_path() -> PathBuf {
    if let Some(path) = env::args_os().nth(1) {
        return PathBuf::from(path);
    }

    let name = format!(
        "{}smart_socket{}",
        env::consts::DLL_PREFIX,
        env::consts::DLL_SUFFIX
    );
    env::current_exe()
        .ok()
        .and_then(|exe| Some(exe.parent()?.join(&name)))
        .unwrap_or_else(|| PathBuf::from(name))
}

fn run(library: &Library) -> Result<(), libloading::Error> {
    // SAFETY: the signatures match the C ABI declared in `smart_socket`.
    unsafe {
        let socket_new =
            *library.get::<unsafe extern "C" fn(f32) -> SocketHandle>(b"smart_socket_new")?;
        let socket_free =
            *library.get::<unsafe extern "C" fn(SocketHandle)>(b"smart_socket_free")?;
        let turn_on =
            *library.get::<unsafe extern "C" fn(SocketHandle)>(b"smart_socket_turn_on")?;
        let turn_off =
            *library.get::<unsafe extern "C" fn(SocketHandle)>(b"smart_socket_turn_off")?;
        let is_on =
            *library.get::<unsafe extern "C" fn(SocketHandle) -> bool>(b"smart_socket_is_on")?;
        let power =
            *library.get::<unsafe extern "C" fn(SocketHandle) -> f32>(b"smart_socket_power")?;

        // SAFETY: `socket` is a live handle created by `socket_new` below.
        let report = |socket: SocketHandle| {
            let (on, watts) = (is_on(socket), power(socket));
            println!(
                "  включена: {}, текущая мощность: {watts} Вт",
                if on { "да" } else { "нет" }
            );
        };

        println!("Умная розетка (динамическая загрузка библиотеки)");

        let socket = socket_new(3500.0);
        report(socket);

        println!("Включаем розетку...");
        turn_on(socket);
        report(socket);

        println!("Выключаем розетку...");
        turn_off(socket);
        report(socket);

        socket_free(socket);
    }

    Ok(())
}
