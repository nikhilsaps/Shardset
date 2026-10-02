// The model: holds the target image and the growing reconstruction. Each Step
// runs the workers in parallel across OS threads, commits the best shape, then
// optionally refines with extra "repeat" shapes.

use crate::bitmap::Bitmap;
use crate::color::Color;
use crate::core_ops::{compute_color, difference_full, difference_partial, draw_lines};
use crate::shape::{Shape, ShapeType};
use crate::state::State;
use crate::worker::Worker;

pub struct Model {
    pub w: i32,
    pub h: i32,
    pub background: Color,
    pub target: Bitmap,
    pub current: Bitmap,
    pub score: f64,
    pub shapes: Vec<Box<dyn Shape>>,
    pub colors: Vec<Color>,
    pub scores: Vec<f64>,
    pub num_workers: usize,
    rng_base: u64,
    seed_counter: u64,
}

impl Model {
    pub fn new(target: Bitmap, background: Color, num_workers: usize) -> Model {
        let current = Bitmap::filled(target.w, target.h, background);
        let score = difference_full(&target, &current);
        let rng_base = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678);
        Model {
            w: target.w,
            h: target.h,
            background,
            target,
            current,
            score,
            shapes: Vec::new(),
            colors: Vec::new(),
            scores: Vec::new(),
            num_workers: num_workers.max(1),
            rng_base,
            seed_counter: 0,
        }
    }

    fn next_seed(&mut self) -> u64 {
        self.seed_counter = self.seed_counter.wrapping_add(1);
        self.rng_base
            .wrapping_add(self.seed_counter.wrapping_mul(0x9E37_79B9_7F4A_7C15))
    }

    fn add(&mut self, shape: Box<dyn Shape>, alpha: i32) {
        let before = self.current.clone();
        let lines = shape.rasterize(self.w, self.h);
        if lines.is_empty() {
            return;
        }
        let color = compute_color(&self.target, &self.current, &lines, alpha);
        draw_lines(&mut self.current, color, &lines);
        let score = difference_partial(&self.target, &before, &self.current, self.score, &lines);
        self.score = score;
        self.shapes.push(shape);
        self.colors.push(color);
        self.scores.push(score);
    }

    pub fn step(&mut self, t: ShapeType, alpha: i32, repeat: i32) -> i64 {
        let (mut best_state, mut total) = self.run_workers(t, alpha, 1000, 100, 16);
        self.add(best_state.shape.clone_box(), best_state.alpha);

        for _ in 0..repeat {
            let seed = self.next_seed();
            best_state.score = -1.0;
            let (a, b, cnt, refined) = {
                let mut worker = Worker::new(&self.target, &self.current, self.score, seed);
                let a = worker.energy_of(&mut best_state);
                let refined = worker.hill_climb(best_state.clone_state(), 100);
                let mut refined = refined;
                let b = worker.energy_of(&mut refined);
                (a, b, worker.counter, refined)
            };
            total += cnt;
            best_state = refined;
            if a == b {
                break;
            }
            self.add(best_state.shape.clone_box(), best_state.alpha);
        }
        total
    }

    fn run_workers(&mut self, t: ShapeType, a: i32, n: i32, age: i32, m: i32) -> (State, i64) {
        let wn = self.num_workers.max(1);
        let wm = (((m as usize) + wn - 1) / wn) as i32;
        let seeds: Vec<u64> = (0..wn).map(|_| self.next_seed()).collect();

        let target = &self.target;
        let current = &self.current;
        let score = self.score;

        let results: Vec<(State, i64)> = std::thread::scope(|scope| {
            let handles: Vec<_> = seeds
                .into_iter()
                .map(|seed| {
                    scope.spawn(move || {
                        let mut worker = Worker::new(target, current, score, seed);
                        let st = worker.best_hill_climb_state(t, a, n, age, wm);
                        (st, worker.counter)
                    })
                })
                .collect();
            handles.into_iter().map(|h| h.join().unwrap()).collect()
        });

        let mut best: Option<State> = None;
        let mut best_energy = 0.0f64;
        let mut counter = 0i64;
        for (st, cnt) in results {
            counter += cnt;
            let e = st.score;
            if best.is_none() || e < best_energy {
                best_energy = e;
                best = Some(st);
            }
        }
        (best.unwrap(), counter)
    }

    pub fn svg(&self) -> String {
        let bg = self.background;
        let mut lines: Vec<String> = Vec::new();
        lines.push(format!(
            "<svg xmlns=\"http://www.w3.org/2000/svg\" version=\"1.1\" width=\"{}\" height=\"{}\">",
            self.w, self.h
        ));
        lines.push(format!(
            "<rect x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" fill=\"#{:02x}{:02x}{:02x}\" />",
            self.w, self.h, bg.r, bg.g, bg.b
        ));
        lines.push("<g transform=\"translate(0.5 0.5)\">".to_string());
        for (i, shape) in self.shapes.iter().enumerate() {
            let c = self.colors[i];
            let attrs = format!(
                "fill=\"#{:02x}{:02x}{:02x}\" fill-opacity=\"{:.4}\"",
                c.r, c.g, c.b, c.a as f64 / 255.0
            );
            lines.push(shape.svg(&attrs));
        }
        lines.push("</g>".to_string());
        lines.push("</svg>".to_string());
        lines.join("\n")
    }
}
