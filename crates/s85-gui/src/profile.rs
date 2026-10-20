//! Host-side profiles. The keyboard cannot report its key mapping or palettes, so the app keeps
//! the source of truth here and writes it to the keyboard.

use std::path::PathBuf;

use s85_protocol::keycode::KeyCode;
use s85_protocol::keymap::{Keymap, SLOTS};
use s85_protocol::lighting::{Palettes, PALETTE_COLORS, PALETTE_MODES};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Profile {
    pub name: String,
    /// 4 layers x 126 slots of raw key codes.
    pub layers: Vec<Vec<u32>>,
    /// 20 modes x 7 colours.
    pub palettes: Vec<[[u8; 3]; PALETTE_COLORS]>,
}

impl Profile {
    pub fn new(name: &str) -> Profile {
        let mut p = Profile { name: name.into(), layers: vec![], palettes: vec![] };
        p.set_keymap(&Keymap::default());
        p.set_palettes(&Palettes::default());
        p
    }

    pub fn keymap(&self) -> Keymap {
        let mut km = Keymap::default();
        for (l, layer) in self.layers.iter().enumerate().take(4) {
            for (s, &c) in layer.iter().enumerate().take(SLOTS) {
                km.layers[l][s] = KeyCode(c);
            }
        }
        km
    }

    pub fn set_keymap(&mut self, km: &Keymap) {
        self.layers = km.layers.iter().map(|l| l.iter().map(|c| c.0).collect()).collect();
    }

    pub fn palettes(&self) -> Palettes {
        let mut p = Palettes::default();
        for (i, m) in self.palettes.iter().enumerate().take(PALETTE_MODES) {
            p.0[i] = *m;
        }
        p
    }

    pub fn set_palettes(&mut self, p: &Palettes) {
        self.palettes = p.0.to_vec();
    }
}

#[derive(Serialize, Deserialize)]
pub struct Store {
    pub active: usize,
    pub profiles: Vec<Profile>,
}

impl Default for Store {
    fn default() -> Store {
        Store { active: 0, profiles: (1..=3).map(|i| Profile::new(&format!("Profile{i}"))).collect() }
    }
}

fn path() -> Option<PathBuf> {
    directories::ProjectDirs::from("", "", "open-s85").map(|d| d.config_dir().join("profiles.json"))
}

impl Store {
    pub fn load() -> Store {
        let s: Option<Store> = path()
            .and_then(|p| std::fs::read_to_string(p).ok())
            .and_then(|t| serde_json::from_str(&t).ok());
        match s {
            Some(s) if !s.profiles.is_empty() => Store { active: s.active.min(s.profiles.len() - 1), ..s },
            _ => Store::default(),
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let p = path().ok_or_else(|| anyhow::anyhow!("no config directory"))?;
        std::fs::create_dir_all(p.parent().unwrap())?;
        std::fs::write(p, serde_json::to_string_pretty(self)?)?;
        Ok(())
    }

    pub fn current(&mut self) -> &mut Profile {
        &mut self.profiles[self.active]
    }
}
