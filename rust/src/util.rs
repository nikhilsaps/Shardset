// Shared helpers: RNG type, angle conversion, rotation, gaussian sampling.

use rand::rngs::SmallRng;
use rand::Rng;
use rand_distr::StandardNormal;

/// The per-worker random number generator.
pub type Rnd = SmallRng;

/// Standard-normal sample (mean 0, stddev 1), like Go's rand.NormFloat64().
#[inline]
pub fn norm(rng: &mut Rnd) -> f64 {
    rng.sample(StandardNormal)
}

#[inline]
pub fn radians(degrees: f64) -> f64 {
    degrees * std::f64::consts::PI / 180.0
}

#[inline]
pub fn rotate(x: f64, y: f64, theta: f64) -> (f64, f64) {
    let rx = x * theta.cos() - y * theta.sin();
    let ry = x * theta.sin() + y * theta.cos();
    (rx, ry)
}
