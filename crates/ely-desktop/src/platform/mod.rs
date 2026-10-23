//! OS calls that need `unsafe`: the tray's and the Dock's.

#[cfg(target_os = "macos")]
pub(crate) mod mac;
#[cfg(target_os = "windows")]
pub(crate) mod windows;

/// Points the process at a private session bus.
#[cfg(all(test, target_os = "linux"))]
pub(crate) fn use_session_bus(address: &str) {
    // SAFETY: only the one test that owns the bus reads or writes this variable.
    unsafe { std::env::set_var("DBUS_SESSION_BUS_ADDRESS", address) };
}
