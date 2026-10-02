use crate::raster::{fill_polygon, flatten_quadratic, stroke_outline};
use crate::scanline::Scanline;
use crate::shape::Shape;
use crate::util::{norm, Rnd};
use rand::Rng;

#[derive(Clone, Copy)]
pub struct Quadratic {
    pub x1: f64, pub y1: f64,
    pub x2: f64, pub y2: f64,
    pub x3: f64, pub y3: f64,
    pub width: f64,
}

impl Quadratic {
    pub fn new_random(w: i32, h: i32, rng: &mut Rnd) -> Quadratic {
        let x1 = rng.gen::<f64>() * w as f64;
        let y1 = rng.gen::<f64>() * h as f64;
        let x2 = x1 + rng.gen::<f64>() * 40.0 - 20.0;
        let y2 = y1 + rng.gen::<f64>() * 40.0 - 20.0;
        let x3 = x2 + rng.gen::<f64>() * 40.0 - 20.0;
        let y3 = y2 + rng.gen::<f64>() * 40.0 - 20.0;
        let mut q = Quadratic { x1, y1, x2, y2, x3, y3, width: 1.0 / 2.0 };
        q.mutate(w, h, rng);
        q
    }

    fn valid(&self) -> bool {
        let dx12 = (self.x1 - self.x2) as i32;
        let dy12 = (self.y1 - self.y2) as i32;
        let dx23 = (self.x2 - self.x3) as i32;
        let dy23 = (self.y2 - self.y3) as i32;
        let dx13 = (self.x1 - self.x3) as i32;
        let dy13 = (self.y1 - self.y3) as i32;
        let d12 = dx12 * dx12 + dy12 * dy12;
        let d23 = dx23 * dx23 + dy23 * dy23;
        let d13 = dx13 * dx13 + dy13 * dy13;
        d13 > d12 && d13 > d23
    }
}

impl Shape for Quadratic {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        // Sample count based on control-polygon length, like a flattening tolerance.
        let span = ((self.x3 - self.x1).hypot(self.y3 - self.y1)
            + (self.x2 - self.x1).hypot(self.y2 - self.y1)) as usize;
        let n = span.clamp(16, 256);
        let pts = flatten_quadratic(self.x1, self.y1, self.x2, self.y2, self.x3, self.y3, n);
        // Original strokes with width ~0.5 at target resolution; keep it visible.
        let outline = stroke_outline(&pts, self.width.max(1.0));
        fill_polygon(&outline, w, h)
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        const M: f64 = 16.0;
        loop {
            match rng.gen_range(0..3) {
                0 => {
                    self.x1 = (self.x1 + norm(rng) * 16.0).clamp(-M, (w - 1) as f64 + M);
                    self.y1 = (self.y1 + norm(rng) * 16.0).clamp(-M, (h - 1) as f64 + M);
                }
                1 => {
                    self.x2 = (self.x2 + norm(rng) * 16.0).clamp(-M, (w - 1) as f64 + M);
                    self.y2 = (self.y2 + norm(rng) * 16.0).clamp(-M, (h - 1) as f64 + M);
                }
                _ => {
                    self.x3 = (self.x3 + norm(rng) * 16.0).clamp(-M, (w - 1) as f64 + M);
                    self.y3 = (self.y3 + norm(rng) * 16.0).clamp(-M, (h - 1) as f64 + M);
                }
            }
            if self.valid() {
                break;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(*self) }

    fn svg(&self, attrs: &str) -> String {
        let attrs = attrs.replace("fill", "stroke");
        format!(
            "<path {} fill=\"none\" d=\"M {} {} Q {} {}, {} {}\" stroke-width=\"{}\" />",
            attrs, self.x1, self.y1, self.x2, self.y2, self.x3, self.y3, self.width
        )
    }
}
