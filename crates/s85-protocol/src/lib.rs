//! Unofficial USB HID protocol for the Royal Kludge RK S85 keyboard.
//!
//! All host-to-keyboard messages are 520-byte HID feature reports with report ID `0x09`, sent to the
//! vendor collection on usage page `0xFF02` / usage `0x02` (USB interface 1). The keyboard answers
//! with 8-byte input reports on the same collection.
//!
//! Packet layout: `09 <cmd> 00 <frame> <idx lo> <idx hi> <len lo> <len hi> <body...>` padded to 520.

pub mod clock;
pub mod device;
pub mod keycode;
pub mod keymap;
pub mod layout;
mod layout_data;
pub mod lighting;
pub mod packet;
pub mod screen;

pub use device::{Device, DeviceInfo};

/// USB vendor ID used by RK (SinoWealth).
pub const VID: u16 = 0x258A;
/// Product IDs known to be RK S85 variants.
pub const PIDS: &[(u16, &str)] = &[(0x01FA, "RK S85 ISO"), (0x0193, "RK S85")];

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("RK S85 not found; connect it with a USB cable (wired mode)")]
    NotFound,
    #[error("HID error: {0}")]
    Hid(#[from] hidapi::HidError),
    #[error("keyboard did not acknowledge {0}")]
    NoAck(String),
    #[error("image error: {0}")]
    Image(#[from] image::ImageError),
    #[error("{0}")]
    Invalid(String),
}

pub type Result<T> = std::result::Result<T, Error>;
