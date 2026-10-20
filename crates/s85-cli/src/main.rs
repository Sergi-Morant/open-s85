use std::io::Write;
use std::path::PathBuf;

use anyhow::{bail, Result};
use chrono::{Datelike, Local, Timelike};
use clap::{Parser, Subcommand, ValueEnum};
use s85_protocol::clock::DateTime;
use s85_protocol::lighting::{self, Config, Palettes};
use s85_protocol::screen::{Animation, Fit};
use s85_protocol::Device;

#[derive(Parser)]
#[command(name = "s85", version, about = "Unofficial control tool for the Royal Kludge RK S85 keyboard")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmd,
}

#[derive(Subcommand)]
enum Cmd {
    /// Show the connected keyboard.
    Info,
    /// Upload an image or GIF to the screen.
    Screen {
        image: PathBuf,
        #[arg(long, value_enum, default_value_t = FitArg::Cover)]
        fit: FitArg,
        /// Frame delay in ms (default: from the GIF).
        #[arg(long)]
        delay: Option<u16>,
    },
    /// Set the screen clock to this computer's local time.
    Clock,
    /// Show or change the lighting effect. Without options, prints the current settings.
    Light {
        /// Effect number (0 = off), see `s85 modes`.
        #[arg(long)]
        mode: Option<u8>,
        /// 0..=20
        #[arg(long)]
        brightness: Option<u8>,
        #[arg(long)]
        speed: Option<u8>,
        /// Palette slot 0..=6, or 7 for colour mixing.
        #[arg(long)]
        color: Option<u8>,
        /// Custom colour as RRGGBB; stored in palette slot 0 and selected. Note: this rewrites all
        /// palettes (the keyboard cannot report them), resetting other modes' palettes to default.
        #[arg(long)]
        rgb: Option<String>,
    },
    /// List lighting effects.
    Modes,
}

#[derive(Clone, Copy, ValueEnum)]
enum FitArg {
    Cover,
    Contain,
    Stretch,
}

impl From<FitArg> for Fit {
    fn from(f: FitArg) -> Fit {
        match f {
            FitArg::Cover => Fit::Cover,
            FitArg::Contain => Fit::Contain,
            FitArg::Stretch => Fit::Stretch,
        }
    }
}

fn mode_name(m: u8) -> &'static str {
    if m == lighting::MODE_OFF {
        return "Off";
    }
    lighting::MODES.iter().find(|(n, _)| *n == m).map(|(_, s)| *s).unwrap_or("?")
}

fn parse_rgb(s: &str) -> Result<[u8; 3]> {
    let s = s.trim_start_matches('#');
    if s.len() != 6 {
        bail!("colour must be RRGGBB");
    }
    let v = u32::from_str_radix(s, 16)?;
    Ok([(v >> 16) as u8, (v >> 8) as u8, v as u8])
}

fn main() -> Result<()> {
    match Cli::parse().cmd {
        Cmd::Info => {
            let dev = Device::open()?;
            println!("{} ({:04x}:{:04x})", dev.info.name, s85_protocol::VID, dev.info.pid);
        }
        Cmd::Screen { image, fit, delay } => {
            let anim = Animation::load(&image, fit.into(), delay)?;
            println!("{} frame(s), {} ms/frame", anim.frames.len(), anim.delay_ms);
            let dev = Device::open()?;
            anim.upload(&dev, |done, total| {
                print!("\r{done}/{total} ({}%)", done * 100 / total);
                let _ = std::io::stdout().flush();
            })?;
            println!("\ndone");
        }
        Cmd::Clock => {
            let now = Local::now();
            let t = DateTime {
                year: now.year() as u16,
                month: now.month() as u8,
                day: now.day() as u8,
                hour: now.hour() as u8,
                minute: now.minute() as u8,
                second: now.second() as u8,
                weekday: now.weekday().num_days_from_sunday() as u8,
            };
            t.write(&Device::open()?)?;
            println!("clock set to {}", now.format("%Y-%m-%d %H:%M:%S"));
        }
        Cmd::Modes => {
            println!("  0  Off");
            for (n, name) in lighting::MODES {
                println!("{n:>3}  {name}");
            }
        }
        Cmd::Light { mode, brightness, speed, color, rgb } => {
            let dev = Device::open()?;
            let mut cfg = Config::read(&dev)?;
            let changed = mode.is_some() || brightness.is_some() || speed.is_some() || color.is_some() || rgb.is_some();
            if changed {
                if let Some(m) = mode {
                    cfg.set_mode(m);
                }
                let m = cfg.mode();
                if let Some(b) = brightness {
                    cfg.set_brightness(m, b);
                }
                if let Some(s) = speed {
                    cfg.set_speed(m, s);
                }
                if let Some(c) = color {
                    cfg.set_color(m, c);
                }
                if let Some(hex) = &rgb {
                    let mut pal = Palettes::default();
                    pal.0[m as usize][0] = parse_rgb(hex)?;
                    cfg.set_color(m, 0);
                    pal.write(&dev)?;
                }
                cfg.write(&dev)?;
            }
            let m = cfg.mode();
            println!("mode:       {m} ({})", mode_name(m));
            println!("brightness: {}/{}", cfg.brightness(m), lighting::MAX_BRIGHTNESS);
            println!("speed:      {}", cfg.speed(m));
            let c = cfg.color(m);
            println!("color:      {}", if c == lighting::COLOR_MIX { "mixed".to_string() } else { format!("palette {c}") });
            println!("sleep:      {} min", cfg.sleep_minutes());
        }
    }
    Ok(())
}
