//! The 320x172 TFT screen: uploading still images and animations.
//!
//! Sequence: start (`0x0D` SetGIFParam, begin flag set), then per frame 215 chunks (`0x0C`, 512 bytes
//! of RGB565 big-endian each, acknowledged by index), then finish (`0x0D`, begin flag clear, real delay).

use std::path::Path;

use image::codecs::gif::GifDecoder;
use image::imageops::FilterType;
use image::{AnimationDecoder, DynamicImage, RgbImage, RgbaImage};

use crate::device::Device;
use crate::packet::{build, cmd, REPORT_LEN};
use crate::{Error, Result};

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 172;
pub const MAX_FRAMES: usize = 60;
pub const CHUNK: usize = 512;
pub const CHUNKS_PER_FRAME: usize = (WIDTH * HEIGHT * 2) as usize / CHUNK; // 215

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Fit {
    /// Fill the screen, cropping the overflow.
    #[default]
    Cover,
    /// Fit inside the screen with black bars.
    Contain,
    /// Distort to the screen size.
    Stretch,
}

/// Frames ready to send, already at screen resolution.
pub struct Animation {
    pub frames: Vec<RgbImage>,
    pub delay_ms: u16,
}

impl Animation {
    /// Load a still image or animated GIF, resize it, and cap it at [`MAX_FRAMES`].
    pub fn load(path: &Path, fit: Fit, delay_override: Option<u16>) -> Result<Animation> {
        let is_gif = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("gif"));
        let mut frames = Vec::new();
        let mut delays = Vec::new();
        if is_gif {
            let file = std::io::BufReader::new(std::fs::File::open(path).map_err(image::ImageError::IoError)?);
            for f in GifDecoder::new(file)?.into_frames() {
                let f = f?;
                let (n, d) = f.delay().numer_denom_ms();
                delays.push(n.checked_div(d).unwrap_or(0));
                frames.push(fit_frame(&DynamicImage::ImageRgba8(f.into_buffer()), fit));
            }
        } else {
            frames.push(fit_frame(&image::open(path)?, fit));
            delays.push(100);
        }
        if frames.is_empty() {
            return Err(Error::Invalid("image has no frames".into()));
        }
        let avg = (delays.iter().sum::<u32>() / delays.len() as u32).max(1);
        if frames.len() > MAX_FRAMES {
            let step = frames.len() as f64 / MAX_FRAMES as f64;
            let total: u32 = delays.iter().sum();
            frames = (0..MAX_FRAMES).map(|i| frames[(i as f64 * step) as usize].clone()).collect();
            delays = vec![(total / MAX_FRAMES as u32).max(1)];
        }
        let delay = delay_override.map(u32::from).unwrap_or(if delays.len() == 1 { delays[0] } else { avg });
        Ok(Animation { frames, delay_ms: delay.clamp(1, u16::MAX as u32) as u16 })
    }

    /// Every report needed to upload this animation, with the ack index to expect.
    pub fn reports(&self) -> Vec<([u8; REPORT_LEN], Option<u8>)> {
        let n = self.frames.len() as u16;
        // The official app sends delay 20 in the start packet and the real delay in the finish packet.
        let mut out = vec![(build(cmd::SCREEN_CONTROL, 0, 1, &gif_param(true, 0, n, 20)), Some(1))];
        for (f, img) in self.frames.iter().enumerate() {
            let data = to_rgb565(img);
            for (i, chunk) in data.chunks(CHUNK).enumerate() {
                out.push((build(cmd::SCREEN_DATA, f as u8, i as u16, chunk), Some(i as u8)));
            }
        }
        out.push((build(cmd::SCREEN_CONTROL, 0, 1, &gif_param(false, 0, n, self.delay_ms)), Some(1)));
        out
    }

    /// Upload to the keyboard, calling `progress(done, total)` after each report.
    pub fn upload(&self, dev: &Device, mut progress: impl FnMut(usize, usize)) -> Result<()> {
        let reports = self.reports();
        dev.drain()?;
        for (i, (r, idx)) in reports.iter().enumerate() {
            dev.send(r, *idx)?;
            progress(i + 1, reports.len());
        }
        Ok(())
    }
}

/// `SetGIFParam` body: `[begin<<6 | slot] [frames u16 LE] [delay ms u16 LE]`.
fn gif_param(begin: bool, slot: u8, frames: u16, delay_ms: u16) -> [u8; 5] {
    let [f0, f1] = frames.to_le_bytes();
    let [d0, d1] = delay_ms.to_le_bytes();
    [(begin as u8) << 6 | slot, f0, f1, d0, d1]
}

fn fit_frame(img: &DynamicImage, fit: Fit) -> RgbImage {
    let (w, h) = (img.width() as f64, img.height() as f64);
    let resized = match fit {
        Fit::Stretch => img.resize_exact(WIDTH, HEIGHT, FilterType::Lanczos3),
        Fit::Cover | Fit::Contain => {
            let s = if fit == Fit::Cover {
                (WIDTH as f64 / w).max(HEIGHT as f64 / h)
            } else {
                (WIDTH as f64 / w).min(HEIGHT as f64 / h)
            };
            let (nw, nh) = (((w * s).round() as u32).max(1), ((h * s).round() as u32).max(1));
            img.resize_exact(nw, nh, FilterType::Lanczos3)
        }
    };
    let mut canvas = RgbaImage::from_pixel(WIDTH, HEIGHT, image::Rgba([0, 0, 0, 255]));
    let x = (WIDTH as i64 - resized.width() as i64) / 2;
    let y = (HEIGHT as i64 - resized.height() as i64) / 2;
    image::imageops::overlay(&mut canvas, &resized.to_rgba8(), x, y);
    DynamicImage::ImageRgba8(canvas).to_rgb8()
}

/// RGB888 -> RGB565, big-endian, row-major from the top-left.
pub fn to_rgb565(img: &RgbImage) -> Vec<u8> {
    img.pixels()
        .flat_map(|p| {
            let [r, g, b] = p.0;
            let v = ((r as u16 & 0xF8) << 8) | ((g as u16 & 0xFC) << 3) | (b as u16 >> 3);
            v.to_be_bytes()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn solid(c: [u8; 3]) -> RgbImage {
        RgbImage::from_pixel(WIDTH, HEIGHT, image::Rgb(c))
    }

    /// Matches the official app's upload of a 2-frame red/blue GIF at 200 ms, as captured over USB.
    #[test]
    fn matches_captured_red_blue_upload() {
        let anim = Animation { frames: vec![solid([255, 0, 0]), solid([0, 0, 255])], delay_ms: 200 };
        let r = anim.reports();
        assert_eq!(r.len(), 1 + 2 * CHUNKS_PER_FRAME + 1);
        assert_eq!(&r[0].0[..13], &[0x09, 0x0D, 0, 0, 0x01, 0, 0x05, 0, 0x40, 0x02, 0x00, 0x14, 0x00]);
        assert_eq!(&r[1].0[..12], &[0x09, 0x0C, 0, 0, 0, 0, 0x00, 0x02, 0xF8, 0x00, 0xF8, 0x00]);
        assert_eq!(&r[430].0[..12], &[0x09, 0x0C, 0, 1, 0xD6, 0, 0x00, 0x02, 0x00, 0x1F, 0x00, 0x1F]);
        assert_eq!(&r[431].0[..13], &[0x09, 0x0D, 0, 0, 0x01, 0, 0x05, 0, 0x00, 0x02, 0x00, 0xC8, 0x00]);
        assert!(r[431].0[13..].iter().all(|&b| b == 0));
    }

    #[test]
    fn gif_param_is_little_endian() {
        assert_eq!(gif_param(false, 0, 60, 300), [0x00, 60, 0, 0x2C, 0x01]);
        assert_eq!(gif_param(true, 0, 2, 20), [0x40, 2, 0, 0x14, 0]);
    }

    #[test]
    fn frame_is_exactly_215_chunks() {
        assert_eq!(to_rgb565(&solid([1, 2, 3])).len(), CHUNKS_PER_FRAME * CHUNK);
    }
}
