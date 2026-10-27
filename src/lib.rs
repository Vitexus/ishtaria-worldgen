//! # ishtaria-worldgen
//!
//! The planet is never stored in full: terrain is a pure function of
//! `(seed, position)`. Servers persist only *deltas* made by players and the
//! simulation; clients regenerate the base terrain locally.
//!
//! Geometry: a cube projected onto a sphere (cube-sphere). Each of the six
//! faces is a quadtree, which gives natural level-of-detail tiles.

pub mod cubesphere;
pub mod earth;
pub mod noise;
pub mod relief;
pub mod strip;

pub use cubesphere::{Face, Vec3};
pub use noise::Terrain;

/// Mean radius of the planet in metres (Earth-like).
pub const PLANET_RADIUS_M: f64 = 6_371_000.0;
