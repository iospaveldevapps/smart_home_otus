//! Smart socket library.
//!
//! Builds three artifacts at once (see `crate-type` in `Cargo.toml`):
//! an `rlib` for Rust consumers, a `staticlib` and a `cdylib` exposing
//! the C ABI declared in [`ffi`].

mod ffi;
mod socket;

pub use ffi::*;
pub use socket::SmartSocket;
