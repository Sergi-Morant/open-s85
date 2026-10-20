//! Physical layout of the RK S85 (ISO).

pub use crate::layout_data::{FACTORY_DEFAULT, FACTORY_FN1, KEYS};

/// One physical key.
#[derive(Debug, Clone, Copy)]
pub struct KeyDef {
    /// Printed legend (US names; the ISO board's extra key is `><`).
    pub label: &'static str,
    /// Position in a 780x300 drawing canvas: (x1, y1, x2, y2).
    pub rect: (u16, u16, u16, u16),
    /// Index into the keyboard's key matrix, used by key mapping and per-key lighting.
    pub slot: u8,
}

/// Look up a key by its matrix slot.
pub fn key_at_slot(slot: u8) -> Option<&'static KeyDef> {
    KEYS.iter().find(|k| k.slot == slot)
}
