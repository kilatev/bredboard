use bevy::asset::RenderAssetUsages;
use bevy::image::{Image, ImageSampler};
use bevy::math::IVec2;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use super::palette::{self, Rgb};

/// A small RGB pixel grid with transparency; y grows downwards like the art.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PixelCanvas {
    width: i32,
    height: i32,
    pixels: Vec<Option<Rgb>>,
}

impl PixelCanvas {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            pixels: vec![None; (width * height) as usize],
        }
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    fn index(&self, x: i32, y: i32) -> Option<usize> {
        ((0..self.width).contains(&x) && (0..self.height).contains(&y))
            .then(|| (y * self.width + x) as usize)
    }

    /// Writes one pixel; coordinates outside the canvas are ignored.
    pub fn set(&mut self, x: i32, y: i32, color: Rgb) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = Some(color);
        }
    }

    pub fn clear(&mut self, x: i32, y: i32) {
        if let Some(i) = self.index(x, y) {
            self.pixels[i] = None;
        }
    }

    pub fn get(&self, x: i32, y: i32) -> Option<Rgb> {
        self.index(x, y).and_then(|i| self.pixels[i])
    }

    /// Draws `other` with its top-left corner at (`ox`, `oy`); transparent pixels are skipped.
    pub fn blit(&mut self, other: &PixelCanvas, ox: i32, oy: i32) {
        for y in 0..other.height {
            for x in 0..other.width {
                if let Some(color) = other.get(x, y) {
                    self.set(x + ox, y + oy, color);
                }
            }
        }
    }

    /// Rotates clockwise by `quarter_turns` * 90 degrees; the left edge becomes the top.
    pub fn rotated(&self, quarter_turns: u8) -> Self {
        let mut result = self.clone();
        for _ in 0..quarter_turns % 4 {
            let mut next = PixelCanvas::new(result.height, result.width);
            for y in 0..result.height {
                for x in 0..result.width {
                    if let Some(color) = result.get(x, y) {
                        next.set(result.height - 1 - y, x, color);
                    }
                }
            }
            result = next;
        }
        result
    }

    /// Silver 2 px lead from a pin's top-left pixel towards `to`, ending in the hole.
    pub fn lead(&mut self, from: IVec2, to: IVec2) {
        let delta = to - from;
        let steps = delta.x.abs().max(delta.y.abs()).max(1);
        let vertical = delta.y.abs() >= delta.x.abs();
        for i in 0..=steps {
            let t = i as f64 / steps as f64;
            let x = (from.x as f64 + delta.x as f64 * t).round() as i32;
            let y = (from.y as f64 + delta.y as f64 * t).round() as i32;
            self.set(x, y, palette::LEAD);
            if vertical {
                self.set(x + 1, y, palette::LEAD_SHADE);
            } else {
                self.set(x, y + 1, palette::LEAD_SHADE);
            }
        }
        self.set(from.x + 1, from.y + 1, palette::HOLE);
    }

    pub fn to_image(&self) -> Image {
        let data = self
            .pixels
            .iter()
            .flat_map(|pixel| match pixel {
                Some([r, g, b]) => [*r, *g, *b, 255],
                None => [0, 0, 0, 0],
            })
            .collect();
        let mut image = Image::new(
            Extent3d {
                width: self.width as u32,
                height: self.height as u32,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            data,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::nearest();
        image
    }

    /// One character per pixel using the palette legend; `.` is transparent.
    #[cfg(test)]
    pub fn to_ascii(&self) -> String {
        let mut out = String::new();
        for y in 0..self.height {
            for x in 0..self.width {
                out.push(self.get(x, y).map_or('.', palette::symbol));
            }
            out.push('\n');
        }
        out
    }
}
