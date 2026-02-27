//! Shared statistical utility functions

use rand::Rng;

/// Standard normal cumulative distribution function (CDF)
/// Φ(z) = probability that a standard normal random variable is ≤ z
/// Uses Hastings approximation (error < 7.5e-8)
pub fn normal_cdf(z: f64) -> f64 {
    if z.is_nan() {
        return 0.5;
    }
    if z >= 8.0 {
        return 1.0;
    }
    if z <= -8.0 {
        return 0.0;
    }

    // Handle negative z by symmetry: Φ(-z) = 1 - Φ(z)
    let (z_abs, negate) = if z < 0.0 { (-z, true) } else { (z, false) };

    // Hastings approximation constants (A&S 26.2.17)
    const B0: f64 = 0.2316419;
    const B1: f64 = 0.319381530;
    const B2: f64 = -0.356563782;
    const B3: f64 = 1.781477937;
    const B4: f64 = -1.821255978;
    const B5: f64 = 1.330274429;

    let t = 1.0 / (1.0 + B0 * z_abs);
    let t2 = t * t;
    let t3 = t2 * t;
    let t4 = t3 * t;
    let t5 = t4 * t;

    let pdf = (-0.5 * z_abs * z_abs).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let cdf = 1.0 - pdf * (B1 * t + B2 * t2 + B3 * t3 + B4 * t4 + B5 * t5);

    if negate {
        1.0 - cdf
    } else {
        cdf
    }
}

/// Generate a standard normal random sample using Box-Muller transform
pub fn box_muller(rng: &mut impl Rng) -> f64 {
    let u1: f64 = rng.random::<f64>().max(f64::EPSILON);
    let u2: f64 = rng.random();
    (-2.0_f64 * u1.ln()).sqrt() * (2.0_f64 * std::f64::consts::PI * u2).cos()
}
