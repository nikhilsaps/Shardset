use crate::raster::{fill_polygon};
use crate::scanline::Scanline;
use crate::shape::Shape;
use crate::util::{norm, radians, rotate, Rnd};
use rand::Rng;

#[derive(Clone, Copy)]
pub struct Ellipse {
    pub x: i32, pub y: i32,
    pub rx: i32, pub ry: i32,
    pub circle: bool,
}

impl Ellipse {
    pub fn new_random_ellipse(w: i32, h: i32, rng: &mut Rnd) -> Ellipse {
        Ellipse {
            x: rng.gen_range(0..w),
            y: rng.gen_range(0..h),
            rx: rng.gen_range(0..32) + 1,
            ry: rng.gen_range(0..32) + 1,
            circle: false,
        }
    }

    pub fn new_random_circle(w: i32, h: i32, rng: &mut Rnd) -> Ellipse {
        let r = rng.gen_range(0..32) + 1;
        Ellipse { x: rng.gen_range(0..w), y: rng.gen_range(0..h), rx: r, ry: r, circle: true }
    }
}

impl Shape for Ellipse {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        let mut lines = Vec::new();
        let aspect = self.rx as f64 / self.ry as f64;
        for dy in 0..self.ry {
            let y1 = self.y - dy;
            let y2 = self.y + dy;
            if (y1 < 0 || y1 >= h) && (y2 < 0 || y2 >= h) {
                continue;
            }
            let s = (((self.ry * self.ry - dy * dy) as f64).sqrt() * aspect) as i32;
            let mut x1 = self.x - s;
            let mut x2 = self.x + s;
            if x1 < 0 { x1 = 0; }
            if x2 >= w { x2 = w - 1; }
            if y1 >= 0 && y1 < h {
                lines.push(Scanline { y: y1, x1, x2, alpha: 0xffff });
            }
            if y2 >= 0 && y2 < h && dy > 0 {
                lines.push(Scanline { y: y2, x1, x2, alpha: 0xffff });
            }
        }
        lines
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        match rng.gen_range(0..3) {
            0 => {
                self.x = (self.x + (norm(rng) * 16.0) as i32).clamp(0, w - 1);
                self.y = (self.y + (norm(rng) * 16.0) as i32).clamp(0, h - 1);
            }
            1 => {
                self.rx = (self.rx + (norm(rng) * 16.0) as i32).clamp(1, w - 1);
                if self.circle { self.ry = self.rx; }
            }
            _ => {
                self.ry = (self.ry + (norm(rng) * 16.0) as i32).clamp(1, h - 1);
                if self.circle { self.rx = self.ry; }
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(*self) }

    fn svg(&self, attrs: &str) -> String {
        format!(
            "<ellipse {} cx=\"{}\" cy=\"{}\" rx=\"{}\" ry=\"{}\" />",
            attrs, self.x, self.y, self.rx, self.ry
        )
    }
}

#[derive(Clone, Copy)]
pub struct RotatedEllipse {
    pub x: f64, pub y: f64,
    pub rx: f64, pub ry: f64,
    pub angle: f64,
}

impl RotatedEllipse {
    pub fn new_random(w: i32, h: i32, rng: &mut Rnd) -> RotatedEllipse {
        RotatedEllipse {
            x: rng.gen::<f64>() * w as f64,
            y: rng.gen::<f64>() * h as f64,
            rx: rng.gen::<f64>() * 32.0 + 1.0,
            ry: rng.gen::<f64>() * 32.0 + 1.0,
            angle: rng.gen::<f64>() * 360.0,
        }
    }
}

impl Shape for RotatedEllipse {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        const N: usize = 32;
        let a = radians(self.angle);
        let mut pts = Vec::with_capacity(N);
        for i in 0..N {
            let t = (i as f64 / N as f64) * 2.0 * std::f64::consts::PI;
            let (ex, ey) = (self.rx * t.cos(), self.ry * t.sin());
            let (rxp, ryp) = rotate(ex, ey, a);
            pts.push((rxp + self.x, ryp + self.y));
        }
        fill_polygon(&pts, w, h)
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        match rng.gen_range(0..3) {
            0 => {
                self.x = (self.x + norm(rng) * 16.0).clamp(0.0, (w - 1) as f64);
                self.y = (self.y + norm(rng) * 16.0).clamp(0.0, (h - 1) as f64);
            }
            1 => {
                self.rx = (self.rx + norm(rng) * 16.0).clamp(1.0, (w - 1) as f64);
                self.ry = (self.ry + norm(rng) * 16.0).clamp(1.0, (w - 1) as f64);
            }
            _ => {
                self.angle += norm(rng) * 32.0;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(*self) }

    fn svg(&self, attrs: &str) -> String {
        format!(
            "<g transform=\"translate({} {}) rotate({}) scale({} {})\"><ellipse {} cx=\"0\" cy=\"0\" rx=\"1\" ry=\"1\" /></g>",
            self.x, self.y, self.angle, self.rx, self.ry, attrs
        )
    }
}
