// A worker owns a scratch buffer and RNG, reads the shared target/current images,
// and searches for a good shape via random sampling + hill climbing.

use crate::bitmap::Bitmap;
use crate::core_ops::{compute_color, copy_lines, difference_partial, draw_lines};
use crate::shape::{Shape, ShapeType};
use crate::shapes::random_shape;
use crate::state::State;
use crate::util::Rnd;
use rand::{Rng, SeedableRng};

pub struct Worker<'a> {
    pub w: i32,
    pub h: i32,
    pub target: &'a Bitmap,
    pub current: &'a Bitmap,
    pub buffer: Bitmap,
    pub rng: Rnd,
    pub score: f64,
    pub counter: i64,
}

impl<'a> Worker<'a> {
    pub fn new(target: &'a Bitmap, current: &'a Bitmap, score: f64, seed: u64) -> Worker<'a> {
        Worker {
            w: target.w,
            h: target.h,
            target,
            current,
            buffer: Bitmap::new(target.w, target.h),
            rng: Rnd::seed_from_u64(seed),
            score,
            counter: 0,
        }
    }

    pub fn energy(&mut self, shape: &dyn Shape, alpha: i32) -> f64 {
        self.counter += 1;
        let lines = shape.rasterize(self.w, self.h);
        if lines.is_empty() {
            return self.score;
        }
        let color = compute_color(self.target, self.current, &lines, alpha);
        copy_lines(&mut self.buffer, self.current, &lines);
        draw_lines(&mut self.buffer, color, &lines);
        difference_partial(self.target, self.current, &self.buffer, self.score, &lines)
    }

    pub fn energy_of(&mut self, state: &mut State) -> f64 {
        if state.score < 0.0 {
            state.score = self.energy(state.shape.as_ref(), state.alpha);
        }
        state.score
    }

    fn do_move(&mut self, state: &mut State) -> State {
        let old = state.clone_state();
        state.shape.mutate(self.w, self.h, &mut self.rng);
        if state.mutate_alpha {
            state.alpha = (state.alpha + self.rng.gen_range(0..21) - 10).clamp(1, 255);
        }
        state.score = -1.0;
        old
    }

    pub fn hill_climb(&mut self, state: State, max_age: i32) -> State {
        let mut state = state.clone_state();
        let mut best = state.clone_state();
        let mut best_energy = self.energy_of(&mut state);
        let mut age = 0i32;
        while age < max_age {
            let undo = self.do_move(&mut state);
            let energy = self.energy_of(&mut state);
            if energy >= best_energy {
                state = undo;
            } else {
                best_energy = energy;
                best = state.clone_state();
                age = -1;
            }
            age += 1;
        }
        best
    }

    pub fn best_random_state(&mut self, t: ShapeType, a: i32, n: i32) -> State {
        let mut best: Option<State> = None;
        let mut best_energy = 0.0f64;
        for i in 0..n {
            let shape = random_shape(t, self.w, self.h, &mut self.rng);
            let mut st = State::new(shape, a);
            let energy = self.energy_of(&mut st);
            if i == 0 || energy < best_energy {
                best_energy = energy;
                best = Some(st);
            }
        }
        best.unwrap()
    }

    pub fn best_hill_climb_state(&mut self, t: ShapeType, a: i32, n: i32, age: i32, m: i32) -> State {
        let mut best: Option<State> = None;
        let mut best_energy = 0.0f64;
        for i in 0..m {
            let st = self.best_random_state(t, a, n);
            let mut st = self.hill_climb(st, age);
            let energy = self.energy_of(&mut st);
            if i == 0 || energy < best_energy {
                best_energy = energy;
                best = Some(st);
            }
        }
        best.unwrap()
    }
}
