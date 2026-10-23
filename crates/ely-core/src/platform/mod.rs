//! OS calls that need `unsafe`: their one home in the core.

#[cfg(target_os = "macos")]
pub(crate) mod mac;
