// A simple RGBA8 image buffer, laid out exactly like Go's image.RGBA (row-major, 4 bytes/pixel).

use crate::color::Color;

#[derive(Clone)]
pub struct Bitmap {
    pub w: i32,
    pub h: i32,
    pub pix: Vec<u8>,
}

impl Bitmap {
    pub fn new(w: i32, h: i32) -> Self {
        Bitmap { w, h, pix: vec![0u8; (w * h * 4) as usize] }
    }

    pub fn filled(w: i32, h: i32, c: Color) -> Self {
        let mut b = Bitmap::new(w, h);
        let (r, g, bl, a) = (c.r as u8, c.g as u8, c.b as u8, c.a as u8);
        let n = (w * h) as usize;
        for i in 0..n {
            let o = i * 4;
            b.pix[o] = r; b.pix[o + 1] = g; b.pix[o + 2] = bl; b.pix[o + 3] = a;
        }
        b
    }

    /// Build from externally supplied RGBA bytes (e.g. decoded by Pillow in Python).
    pub fn from_rgba(w: i32, h: i32, data: Vec<u8>) -> Self {
        debug_assert_eq!(data.len(), (w * h * 4) as usize);
        Bitmap { w, h, pix: data }
    }

    #[inline]
    pub fn offset(&self, x: i32, y: i32) -> usize {
        ((y * self.w + x) * 4) as usize
    }

    /// Average color of the whole image (opaque).
    pub fn average_color(&self) -> Color {
        let n = (self.w * self.h) as i64;
        if n == 0 { return Color::new(0, 0, 0, 255); }
        let (mut r, mut g, mut b) = (0i64, 0i64, 0i64);
        let mut i = 0usize;
        for _ in 0..n {
            r += self.pix[i] as i64;
            g += self.pix[i + 1] as i64;
            b += self.pix[i + 2] as i64;
            i += 4;
        }
        Color::new((r / n) as i32, (g / n) as i32, (b / n) as i32, 255)
    }
}
