//! The height strip, as the server stores and clients read it.
//!
//! A P5 PGM with maxval 65535, big-endian, width = 6 x face size. Each sample is
//! the height in metres above sea level quantised in steps of [`STEP_M`] from
//! [`FLOOR_M`]. Sea level is exactly [`SEA_SAMPLE`]: water samples are never
//! above it and land samples are never at or below it, so the coast survives the
//! quantisation. The local relief ([`crate::relief`]) is part of the strip, so the
//! server and every client read the same surface.

use crate::relief::relief_metres;

/// Height of the lowest sample, in metres.
pub const FLOOR_M: f64 = -8000.0;
/// Metres per sample step.
pub const STEP_M: f64 = 0.25;
/// Sample of sea level (0 m).
pub const SEA_SAMPLE: u16 = 32000;
/// Lowest sample a land point may take.
pub const LAND_SAMPLE: u16 = SEA_SAMPLE + 1;

/// Encodes a height in metres. `water` says whether the point is sea, lake or river.
pub fn encode(metres: f64, water: bool) -> u16 {
    let sample = ((metres - FLOOR_M) / STEP_M)
        .round()
        .clamp(0.0, f64::from(u16::MAX)) as u16;
    if water {
        sample.min(SEA_SAMPLE)
    } else {
        sample.max(LAND_SAMPLE)
    }
}

/// Height of a unit direction in metres, with the local relief of the seed added on land.
pub fn height_with_relief(direction: [f64; 3], base_metres: f64, seed: &str) -> f64 {
    base_metres + relief_metres(direction, base_metres, seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sea_level_is_the_sea_sample() {
        assert_eq!(encode(0.0, true), SEA_SAMPLE);
        assert_eq!(encode(0.0, false), LAND_SAMPLE);
    }

    #[test]
    fn steps_are_a_quarter_metre_from_the_floor() {
        assert_eq!(encode(FLOOR_M, true), 0);
        assert_eq!(encode(-7999.75, true), 1);
        assert_eq!(encode(1.0, false), 32004);
    }

    #[test]
    fn water_never_rises_above_sea_and_land_never_sinks_below_it() {
        assert_eq!(encode(12.0, true), SEA_SAMPLE);
        assert_eq!(encode(-12.0, false), LAND_SAMPLE);
    }

    #[test]
    fn samples_clamp_at_both_ends() {
        assert_eq!(encode(-100_000.0, true), 0);
        assert_eq!(encode(100_000.0, false), u16::MAX);
    }
}
