use crate::raster::fill_polygon;
use crate::scanline::Scanline;
use crate::shape::Shape;
use crate::util::{norm, radians, rotate, Rnd};
use rand::Rng;

#[derive(Clone, Copy)]
pub struct Rectangle {
    pub x1: i32, pub y1: i32,
    pub x2: i32, pub y2: i32,
}

impl Rectangle {
    pub fn new_random(w: i32, h: i32, rng: &mut Rnd) -> Rectangle {
        let x1 = rng.gen_range(0..w);
        let y1 = rng.gen_range(0..h);
        let x2 = (x1 + rng.gen_range(0..32) + 1).clamp(0, w - 1);
        let y2 = (y1 + rng.gen_range(0..32) + 1).clamp(0, h - 1);
        Rectangle { x1, y1, x2, y2 }
    }

    fn bounds(&self) -> (i32, i32, i32, i32) {
        let (mut x1, mut y1, mut x2, mut y2) = (self.x1, self.y1, self.x2, self.y2);
        if x1 > x2 { std::mem::swap(&mut x1, &mut x2); }
        if y1 > y2 { std::mem::swap(&mut y1, &mut y2); }
        (x1, y1, x2, y2)
    }
}

impl Shape for Rectangle {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        let (x1, y1, x2, y2) = self.bounds();
        let mut lines = Vec::new();
        for y in y1..=y2 {
            if y < 0 || y >= h { continue; }
            let a = x1.max(0);
            let b = x2.min(w - 1);
            if b >= a {
                lines.push(Scanline { y, x1: a, x2: b, alpha: 0xffff });
            }
        }
        lines
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        match rng.gen_range(0..2) {
            0 => {
                self.x1 = (self.x1 + (norm(rng) * 16.0) as i32).clamp(0, w - 1);
                self.y1 = (self.y1 + (norm(rng) * 16.0) as i32).clamp(0, h - 1);
            }
            _ => {
                self.x2 = (self.x2 + (norm(rng) * 16.0) as i32).clamp(0, w - 1);
                self.y2 = (self.y2 + (norm(rng) * 16.0) as i32).clamp(0, h - 1);
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(*self) }

    fn svg(&self, attrs: &str) -> String {
        let (x1, y1, x2, y2) = self.bounds();
        format!(
            "<rect {} x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" />",
            attrs, x1, y1, x2 - x1 + 1, y2 - y1 + 1
        )
    }
}

#[derive(Clone, Copy)]
pub struct RotatedRectangle {
    pub x: i32, pub y: i32,
    pub sx: i32, pub sy: i32,
    pub angle: i32,
}

impl RotatedRectangle {
    pub fn new_random(w: i32, h: i32, rng: &mut Rnd) -> RotatedRectangle {
        let x = rng.gen_range(0..w);
        let y = rng.gen_range(0..h);
        let sx = rng.gen_range(0..32) + 1;
        let sy = rng.gen_range(0..32) + 1;
        let angle = rng.gen_range(0..360);
        let mut r = RotatedRectangle { x, y, sx, sy, angle };
        r.mutate(w, h, rng);
        r
    }

    fn corners(&self) -> [(f64, f64); 4] {
        let (sx, sy) = (self.sx as f64, self.sy as f64);
        let a = radians(self.angle as f64);
        let (x, y) = (self.x as f64, self.y as f64);
        let p = |px: f64, py: f64| {
            let (rx, ry) = rotate(px, py, a);
            (rx + x, ry + y)
        };
        [
            p(-sx / 2.0, -sy / 2.0),
            p(sx / 2.0, -sy / 2.0),
            p(sx / 2.0, sy / 2.0),
            p(-sx / 2.0, sy / 2.0),
        ]
    }
}

impl Shape for RotatedRectangle {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        fill_polygon(&self.corners(), w, h)
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        match rng.gen_range(0..3) {
            0 => {
                self.x = (self.x + (norm(rng) * 16.0) as i32).clamp(0, w - 1);
                self.y = (self.y + (norm(rng) * 16.0) as i32).clamp(0, h - 1);
            }
            1 => {
                self.sx = (self.sx + (norm(rng) * 16.0) as i32).clamp(1, w - 1);
                self.sy = (self.sy + (norm(rng) * 16.0) as i32).clamp(1, h - 1);
            }
            _ => {
                self.angle += (norm(rng) * 32.0) as i32;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(*self) }

    fn svg(&self, attrs: &str) -> String {
        format!(
            "<g transform=\"translate({} {}) rotate({}) scale({} {})\"><rect {} x=\"-0.5\" y=\"-0.5\" width=\"1\" height=\"1\" /></g>",
            self.x, self.y, self.angle, self.sx, self.sy, attrs
        )
    }
}
