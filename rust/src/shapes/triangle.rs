use crate::raster::fill_polygon;
use crate::scanline::Scanline;
use crate::shape::Shape;
use crate::util::{norm, Rnd};
use rand::Rng;

#[derive(Clone, Copy)]
pub struct Triangle {
    pub x1: i32, pub y1: i32,
    pub x2: i32, pub y2: i32,
    pub x3: i32, pub y3: i32,
}

impl Triangle {
    pub fn new_random(w: i32, h: i32, rng: &mut Rnd) -> Triangle {
        let x1 = rng.gen_range(0..w);
        let y1 = rng.gen_range(0..h);
        let x2 = x1 + rng.gen_range(0..31) - 15;
        let y2 = y1 + rng.gen_range(0..31) - 15;
        let x3 = x1 + rng.gen_range(0..31) - 15;
        let y3 = y1 + rng.gen_range(0..31) - 15;
        let mut t = Triangle { x1, y1, x2, y2, x3, y3 };
        t.mutate(w, h, rng);
        t
    }

    fn valid(&self) -> bool {
        const MIN_DEGREES: f64 = 15.0;
        let deg = |rad: f64| rad * 180.0 / std::f64::consts::PI;
        let a1;
        {
            let (mut x1, mut y1) = ((self.x2 - self.x1) as f64, (self.y2 - self.y1) as f64);
            let (mut x2, mut y2) = ((self.x3 - self.x1) as f64, (self.y3 - self.y1) as f64);
            let d1 = (x1 * x1 + y1 * y1).sqrt();
            let d2 = (x2 * x2 + y2 * y2).sqrt();
            x1 /= d1; y1 /= d1; x2 /= d2; y2 /= d2;
            a1 = deg((x1 * x2 + y1 * y2).clamp(-1.0, 1.0).acos());
        }
        let a2;
        {
            let (mut x1, mut y1) = ((self.x1 - self.x2) as f64, (self.y1 - self.y2) as f64);
            let (mut x2, mut y2) = ((self.x3 - self.x2) as f64, (self.y3 - self.y2) as f64);
            let d1 = (x1 * x1 + y1 * y1).sqrt();
            let d2 = (x2 * x2 + y2 * y2).sqrt();
            x1 /= d1; y1 /= d1; x2 /= d2; y2 /= d2;
            a2 = deg((x1 * x2 + y1 * y2).clamp(-1.0, 1.0).acos());
        }
        let a3 = 180.0 - a1 - a2;
        a1 > MIN_DEGREES && a2 > MIN_DEGREES && a3 > MIN_DEGREES
    }
}

impl Shape for Triangle {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        let pts = [
            (self.x1 as f64, self.y1 as f64),
            (self.x2 as f64, self.y2 as f64),
            (self.x3 as f64, self.y3 as f64),
        ];
        fill_polygon(&pts, w, h)
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        const M: i32 = 16;
        loop {
            match rng.gen_range(0..3) {
                0 => {
                    self.x1 = (self.x1 + (norm(rng) * 16.0) as i32).clamp(-M, w - 1 + M);
                    self.y1 = (self.y1 + (norm(rng) * 16.0) as i32).clamp(-M, h - 1 + M);
                }
                1 => {
                    self.x2 = (self.x2 + (norm(rng) * 16.0) as i32).clamp(-M, w - 1 + M);
                    self.y2 = (self.y2 + (norm(rng) * 16.0) as i32).clamp(-M, h - 1 + M);
                }
                _ => {
                    self.x3 = (self.x3 + (norm(rng) * 16.0) as i32).clamp(-M, w - 1 + M);
                    self.y3 = (self.y3 + (norm(rng) * 16.0) as i32).clamp(-M, h - 1 + M);
                }
            }
            if self.valid() {
                break;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> {
        Box::new(*self)
    }

    fn svg(&self, attrs: &str) -> String {
        format!(
            "<polygon {} points=\"{},{} {},{} {},{}\" />",
            attrs, self.x1, self.y1, self.x2, self.y2, self.x3, self.y3
        )
    }
}
