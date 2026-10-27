//! Local relief of environment version 2, baked into the world map at generation time.
//!
//! The formula is the one the clients used on their own (`surface_environment.gd`, `relief64`):
//! a sum of three sinusoids over the position on the Earth-sized sphere, phased by the world seed
//! and faded out at the coast, so the relief never lifts the sea floor.

use std::f64::consts::TAU;

/// Radius of the sphere the relief is sampled on, in metres.
const EARTH_RADIUS_M: f64 = 6_371_000.0;

/// Relief in metres at a unit direction, for a base elevation of `base_metres`.
///
/// `direction` need not be normalised; the point is taken on the sphere of [`EARTH_RADIUS_M`].
pub fn relief_metres(direction: [f64; 3], base_metres: f64, seed: &str) -> f64 {
    let radius =
        (direction[0] * direction[0] + direction[1] * direction[1] + direction[2] * direction[2])
            .sqrt();
    let x = direction[0] / radius * EARTH_RADIUS_M;
    let y = direction[1] / radius * EARTH_RADIUS_M;
    let z = direction[2] / radius * EARTH_RADIUS_M;

    let hash = seed
        .bytes()
        .fold(0u64, |hash, byte| (hash * 31 + u64::from(byte)) % 997);
    let phase = hash as f64 * TAU / 997.0;

    let coast = (base_metres / 150.0).clamp(0.0, 1.0);
    let fade = coast * coast * (3.0 - 2.0 * coast);

    fade * (9.0
        * ((x * 0.73 + y * 0.41 + z * 0.55) / 93.0 + phase).sin()
        * ((x * -0.38 + y * 0.87 + z * 0.31) / 127.0 - phase).sin()
        + 3.0 * ((x * 0.21 + y * -0.52 + z * 0.83) / 37.0 + phase).sin())
}

#[cfg(test)]
mod tests {
    use super::relief_metres;

    /// Reference values computed by the client formula in Godot 4.5 (headless).
    #[test]
    fn matches_the_client_formula() {
        assert_eq!(relief_metres([1.0, 0.0, 0.0], 0.0, "2577465639"), 0.0);
        assert!(
            (relief_metres([0.3, -0.7, 0.64], 500.0, "2577465639") - 5.890_928_834_337_3).abs()
                < 1e-9
        );
        assert!(
            (relief_metres([-0.5, 0.2, -0.84], 120.0, "gilgamesh") - 2.334_152_346_294_56).abs()
                < 1e-9
        );
        assert_eq!(relief_metres([0.0, 1.0, 0.0], -300.0, "x"), 0.0);
        assert!(
            (relief_metres([0.57, 0.57, 0.58], 75.0, "tutorial-seed") - 2.986_327_662_722_07).abs()
                < 1e-9
        );
    }
}
