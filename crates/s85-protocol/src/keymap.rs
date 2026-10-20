//! Key mapping (SetMatrix, command 0x03).
//!
//! Each layer is written whole: 126 matrix slots x 4 bytes. The firmware cannot report its current
//! mapping, so the host must keep the full keymap and resend a layer whenever it changes.

use crate::device::Device;
use crate::keycode::KeyCode;
use crate::layout::{FACTORY_DEFAULT, FACTORY_FN1};
use crate::packet::{build, cmd};
use crate::Result;

pub const SLOTS: usize = 126;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Layer {
    Default = 0,
    Fn1 = 1,
    Fn2 = 2,
    Tap = 3,
}

impl Layer {
    pub const ALL: [Layer; 4] = [Layer::Default, Layer::Fn1, Layer::Fn2, Layer::Tap];
    pub fn title(self) -> &'static str {
        match self {
            Layer::Default => "Default",
            Layer::Fn1 => "FN1",
            Layer::Fn2 => "FN2",
            Layer::Tap => "Tap",
        }
    }
}

/// All four layers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keymap {
    pub layers: [[KeyCode; SLOTS]; 4],
}

impl Default for Keymap {
    /// Factory mapping as shipped (captured from the official app). FN2 and Tap are empty.
    fn default() -> Keymap {
        let conv = |a: &[u32; SLOTS]| a.map(KeyCode);
        Keymap {
            layers: [conv(&FACTORY_DEFAULT), conv(&FACTORY_FN1), [KeyCode::NONE; SLOTS], [KeyCode::NONE; SLOTS]],
        }
    }
}

impl Keymap {
    pub fn get(&self, layer: Layer, slot: u8) -> KeyCode {
        self.layers[layer as usize][slot as usize]
    }

    pub fn set(&mut self, layer: Layer, slot: u8, code: KeyCode) {
        self.layers[layer as usize][slot as usize] = code;
    }

    /// The report that writes one layer.
    pub fn layer_report(&self, layer: Layer) -> [u8; crate::packet::REPORT_LEN] {
        let body: Vec<u8> = self.layers[layer as usize].iter().flat_map(|c| c.to_bytes()).collect();
        build_access(cmd::SET_MATRIX, layer as u8, &body)
    }

    /// Write one layer to the keyboard.
    pub fn write_layer(&self, dev: &Device, layer: Layer) -> Result<()> {
        dev.write(&self.layer_report(layer))
    }

    pub fn layer_is_empty(&self, layer: Layer) -> bool {
        self.layers[layer as usize].iter().all(|&c| c == KeyCode::NONE)
    }

    /// Whether the Default layer still has a way to reach FN1 (refuse to write a board without Fn).
    pub fn has_fn_key(&self) -> bool {
        self.layers[Layer::Default as usize].contains(&KeyCode::FN)
    }
}

/// Single-chunk `AccessData` report: `09 <cmd> <b2> 00 01 00 <len LE> <body>`.
pub(crate) fn build_access(cmd: u8, b2: u8, body: &[u8]) -> [u8; crate::packet::REPORT_LEN] {
    let mut p = build(cmd, 0, 0, body);
    p[2] = b2;
    p[4] = 1; // total chunks
    p[5] = 0; // chunk index
    p
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn factory_default_matches_capture_header() {
        let km = Keymap::default();
        let r = km.layer_report(Layer::Fn1);
        assert_eq!(&r[..8], &[0x09, 0x03, 0x01, 0x00, 0x01, 0x00, 0xF8, 0x01]);
        // FN1 F1 (slot 6) = My Computer consumer key in the factory layer
        assert_eq!(&r[8 + 24..8 + 28], &[0x02, 0x00, 0x01, 0x94]);
        assert!(km.has_fn_key());
        // Caps Lock (slot 3) on the Default layer
        assert_eq!(km.get(Layer::Default, 3), KeyCode::key(0x39));
    }
}
