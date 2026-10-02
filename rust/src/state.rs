// A candidate solution during optimization: one shape plus its alpha.

use crate::shape::Shape;

pub struct State {
    pub shape: Box<dyn Shape>,
    pub alpha: i32,
    pub mutate_alpha: bool,
    pub score: f64, // negative => dirty / needs recompute
}

impl State {
    pub fn new(shape: Box<dyn Shape>, alpha: i32) -> State {
        let (alpha, mutate_alpha) = if alpha == 0 { (128, true) } else { (alpha, false) };
        State { shape, alpha, mutate_alpha, score: -1.0 }
    }

    pub fn clone_state(&self) -> State {
        State {
            shape: self.shape.clone_box(),
            alpha: self.alpha,
            mutate_alpha: self.mutate_alpha,
            score: self.score,
        }
    }
}
