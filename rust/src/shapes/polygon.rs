use crate::raster::fill_polygon;
use crate::scanline::Scanline;
use crate::shape::Shape;
use crate::util::{norm, Rnd};
use rand::Rng;

#[derive(Clone)]
pub struct Polygon {
    pub order: usize,
    pub convex: bool,
    pub x: Vec<f64>,
    pub y: Vec<f64>,
}

impl Polygon {
    pub fn new_random(w: i32, h: i32, order: usize, convex: bool, rng: &mut Rnd) -> Polygon {
        let mut x = vec![0.0; order];
        let mut y = vec![0.0; order];
        x[0] = rng.gen::<f64>() * w as f64;
        y[0] = rng.gen::<f64>() * h as f64;
        for i in 1..order {
            x[i] = x[0] + rng.gen::<f64>() * 40.0 - 20.0;
            y[i] = y[0] + rng.gen::<f64>() * 40.0 - 20.0;
        }
        let mut p = Polygon { order, convex, x, y };
        p.mutate(w, h, rng);
        p
    }

    fn valid(&self) -> bool {
        if !self.convex {
            return true;
        }
        let cross3 = |x1: f64, y1: f64, x2: f64, y2: f64, x3: f64, y3: f64| {
            (x2 - x1) * (y3 - y2) - (y2 - y1) * (x3 - x2)
        };
        let mut sign = false;
        for a in 0..self.order {
            let i = a % self.order;
            let j = (a + 1) % self.order;
            let k = (a + 2) % self.order;
            let c = cross3(self.x[i], self.y[i], self.x[j], self.y[j], self.x[k], self.y[k]);
            if a == 0 {
                sign = c > 0.0;
            } else if (c > 0.0) != sign {
                return false;
            }
        }
        true
    }
}

impl Shape for Polygon {
    fn rasterize(&self, w: i32, h: i32) -> Vec<Scanline> {
        let pts: Vec<(f64, f64)> = (0..self.order).map(|i| (self.x[i], self.y[i])).collect();
        fill_polygon(&pts, w, h)
    }

    fn mutate(&mut self, w: i32, h: i32, rng: &mut Rnd) {
        const M: f64 = 16.0;
        loop {
            if rng.gen::<f64>() < 0.25 {
                let i = rng.gen_range(0..self.order);
                let j = rng.gen_range(0..self.order);
                self.x.swap(i, j);
                self.y.swap(i, j);
            } else {
                let i = rng.gen_range(0..self.order);
                self.x[i] = (self.x[i] + norm(rng) * 16.0).clamp(-M, (w - 1) as f64 + M);
                self.y[i] = (self.y[i] + norm(rng) * 16.0).clamp(-M, (h - 1) as f64 + M);
            }
            if self.valid() {
                break;
            }
        }
    }

    fn clone_box(&self) -> Box<dyn Shape> { Box::new(self.clone()) }

    fn svg(&self, attrs: &str) -> String {
        let pts: Vec<String> = (0..self.order)
            .map(|i| format!("{},{}", self.x[i], self.y[i]))
            .collect();
        format!("<polygon {} points=\"{}\" />", attrs, pts.join(","))
    }
}
