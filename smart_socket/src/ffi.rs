//! C ABI for the smart socket.
//!
//! The socket is exposed to C callers as an opaque handle
//! (`struct SmartSocket *`). Every function is exported with an unmangled
//! name, so the `staticlib`/`cdylib` artifacts can be consumed from C or any
//! other FFI-capable language.

use crate::SmartSocket;

/// Creates a socket that draws `power` watts while on.
///
/// The returned pointer owns the socket and must be released with
/// [`smart_socket_free`]. Never returns null.
#[unsafe(no_mangle)]
pub extern "C" fn smart_socket_new(power: f32) -> *mut SmartSocket {
    Box::into_raw(Box::new(SmartSocket::new(power)))
}

/// Destroys a socket created by [`smart_socket_new`]. Null is ignored.
///
/// # Safety
///
/// `socket` must be null or a pointer returned by [`smart_socket_new`]
/// that has not been freed yet.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_free(socket: *mut SmartSocket) {
    if !socket.is_null() {
        drop(unsafe { Box::from_raw(socket) });
    }
}

/// Turns the socket on. Null is ignored.
///
/// # Safety
///
/// `socket` must be null or a valid pointer returned by [`smart_socket_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_turn_on(socket: *mut SmartSocket) {
    if let Some(socket) = unsafe { socket.as_mut() } {
        socket.turn_on();
    }
}

/// Turns the socket off. Null is ignored.
///
/// # Safety
///
/// `socket` must be null or a valid pointer returned by [`smart_socket_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_turn_off(socket: *mut SmartSocket) {
    if let Some(socket) = unsafe { socket.as_mut() } {
        socket.turn_off();
    }
}

/// Reports whether the socket is on. Returns `false` for null.
///
/// # Safety
///
/// `socket` must be null or a valid pointer returned by [`smart_socket_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_is_on(socket: *const SmartSocket) -> bool {
    unsafe { socket.as_ref() }.is_some_and(SmartSocket::is_on)
}

/// Returns the current power draw in watts. Returns `0.0` for null.
///
/// # Safety
///
/// `socket` must be null or a valid pointer returned by [`smart_socket_new`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn smart_socket_power(socket: *const SmartSocket) -> f32 {
    unsafe { socket.as_ref() }.map_or(0.0, SmartSocket::power)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ptr;

    #[test]
    fn controls_socket_through_c_abi() {
        unsafe {
            let socket = smart_socket_new(2000.0);
            assert!(!smart_socket_is_on(socket));
            assert_eq!(smart_socket_power(socket), 0.0);

            smart_socket_turn_on(socket);
            assert!(smart_socket_is_on(socket));
            assert_eq!(smart_socket_power(socket), 2000.0);

            smart_socket_turn_off(socket);
            assert!(!smart_socket_is_on(socket));
            assert_eq!(smart_socket_power(socket), 0.0);

            smart_socket_free(socket);
        }
    }

    #[test]
    fn tolerates_null_pointers() {
        unsafe {
            smart_socket_turn_on(ptr::null_mut());
            smart_socket_turn_off(ptr::null_mut());
            assert!(!smart_socket_is_on(ptr::null()));
            assert_eq!(smart_socket_power(ptr::null()), 0.0);
            smart_socket_free(ptr::null_mut());
        }
    }
}
