//! Demonstrates the smart socket driven through its C ABI while the library
//! is linked statically: the `smart_socket` crate is compiled into this
//! binary, and the `smart_socket_*` symbols are resolved at link time.

use smart_socket::{
    SmartSocket, smart_socket_free, smart_socket_is_on, smart_socket_new, smart_socket_power,
    smart_socket_turn_off, smart_socket_turn_on,
};

fn main() {
    println!("Умная розетка (статическая линковка библиотеки)");

    // SAFETY: the pointer comes from `smart_socket_new`, is used only until
    // `smart_socket_free` and is freed exactly once.
    unsafe {
        let socket = smart_socket_new(1200.0);
        report(socket);

        println!("Включаем розетку...");
        smart_socket_turn_on(socket);
        report(socket);

        println!("Выключаем розетку...");
        smart_socket_turn_off(socket);
        report(socket);

        smart_socket_free(socket);
    }
}

/// # Safety
///
/// `socket` must be a valid pointer returned by [`smart_socket_new`].
unsafe fn report(socket: *const SmartSocket) {
    let is_on = unsafe { smart_socket_is_on(socket) };
    let power = unsafe { smart_socket_power(socket) };
    println!(
        "  включена: {}, текущая мощность: {power} Вт",
        if is_on { "да" } else { "нет" }
    );
}
