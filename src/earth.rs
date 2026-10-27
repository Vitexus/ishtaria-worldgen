//! Real-planet mode: sample an equirectangular relief map onto the cube-sphere.
//!
//! A datadisk may ship the outline and heights of a real planet instead of a
//! procedural one (see the datadisk manifest's `world.terrain`). The source is
//! two plain P5 PGMs of identical size, `width == 2 * height`, row 0 at +90
//! degrees and column 0 at -180 degrees:
//!
//! - `heightmap`: 16-bit (maxval 65535, big-endian), pixel = elevation in metres
//!   as a signed 16-bit two's complement value, so 0 is sea level, land is
//!   positive and bathymetry is negative;
//! - `water`: 8-bit (maxval 255), 0 is land and 255 is water (sea, lakes,
//!   rivers). It wins over the heights: a water pixel never rises above sea
//!   level, a land pixel never sinks below it, so dry depressions (Qattara)
//!   stay land and shallow seas (the Persian Gulf) stay sea however coarse
//!   the 8-bit strip quantises them.
//!
//! Sphere coordinates map to Earth geography as `latitude = asin(y)` and
//! `longitude = atan2(-z, x)`: the +Y face is the north pole, the +X face the
//! Greenwich meridian and east is towards -Z, so the right-handed Y-up frame is
//! not a mirror image of the planet (seen from outside, east is on the right). With this convention the server's compass (`north` in
//! placement) is the real north of the source planet.

use crate::cubesphere::Face;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::Path;

/// Metres below which the strip encoder clamps to 0, matching the server.
/// Strip bytes: 127 and less decode at or below sea level, 128 and more above it.

#[derive(Debug)]
pub struct Equirect {
    pub width: usize,
    pub height: usize,
    metres: Vec<f64>,
    water: Vec<f64>,
}

impl Equirect {
    /// Reads and validates the datadisk's equirectangular height and water PGMs.
    pub fn load(heightmap: &Path, water: &Path) -> io::Result<Self> {
        let (width, height, metres) = read_pgm(heightmap, 65535)?;
        let (water_width, water_height, mask) = read_pgm(water, 255)?;
        if width != 2 * height {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{}: equirectangular map must be twice as wide as tall, got {width}x{height}",
                    heightmap.display()
                ),
            ));
        }
        if water_width != width || water_height != height {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!(
                    "{}: water mask is {water_width}x{water_height}, expected {width}x{height}",
                    water.display()
                ),
            ));
        }
        Ok(Self {
            width,
            height,
            metres: metres.iter().map(|&raw| raw as i16 as f64).collect(),
            water: mask.iter().map(|&raw| f64::from(raw) / 255.0).collect(),
        })
    }

    /// Bilinear sample at `latitude_degrees`, `lon_degrees`: metres above sea
    /// level and the water fraction (1.0 is surely water).
    pub fn sample(&self, latitude_degrees: f64, lon_degrees: f64) -> (f64, f64) {
        let x =
            ((lon_degrees + 180.0) / 360.0 * self.width as f64 - 0.5).rem_euclid(self.width as f64);
        let y = ((90.0 - latitude_degrees) / 180.0 * self.height as f64 - 0.5)
            .clamp(0.0, self.height as f64 - 1.0);
        let left = x.floor() as usize;
        let right = (left + 1) % self.width;
        let north = (y.floor() as usize).min(self.height - 1);
        let south = (north + 1).min(self.height - 1);
        let blend = x - x.floor();
        let step = y - y.floor();
        let at = |column: usize, row: usize, grid: &Vec<f64>| grid[row * self.width + column];
        let metres = at(left, north, &self.metres) * (1.0 - blend) * (1.0 - step)
            + at(right, north, &self.metres) * blend * (1.0 - step)
            + at(left, south, &self.metres) * (1.0 - blend) * step
            + at(right, south, &self.metres) * blend * step;
        let water = at(left, north, &self.water) * (1.0 - blend) * (1.0 - step)
            + at(right, north, &self.water) * blend * (1.0 - step)
            + at(left, south, &self.water) * (1.0 - blend) * step
            + at(right, south, &self.water) * blend * step;
        (metres, water)
    }

    /// Strip sample of a cube-sphere point: the height with its local relief for `seed`,
    /// quantised by [`crate::strip::encode`] with the water mask deciding the side of sea level.
    pub fn face_sample(&self, face: Face, u: f64, v: f64, seed: &str) -> u16 {
        let point = face.sphere_point(u, v);
        let latitude = point.y.asin().to_degrees();
        let lon = (-point.z).atan2(point.x).to_degrees();
        let (base, water) = self.sample(latitude, lon);
        let metres = if water >= 0.5 {
            base
        } else {
            crate::strip::height_with_relief([point.x, point.y, point.z], base, seed)
        };
        crate::strip::encode(metres, water >= 0.5)
    }
}

/// A binary P5 PGM of one sample per pixel, `maxval` at most `maximum`.
fn read_pgm(path: &Path, maximum: u16) -> io::Result<(usize, usize, Vec<u16>)> {
    let name = path.display();
    let bytes = fs::read(path)
        .map_err(|error| io::Error::new(error.kind(), format!("cannot read {name}: {error}")))?;
    let invalid =
        |message: &str| io::Error::new(ErrorKind::InvalidData, format!("{name}: {message}"));
    let mut cursor = 0usize;
    let mut token = || -> Option<String> {
        while cursor < bytes.len() {
            if bytes[cursor] == b'#' {
                while cursor < bytes.len() && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
            } else if bytes[cursor].is_ascii_whitespace() {
                cursor += 1;
            } else {
                break;
            }
        }
        let start = cursor;
        while cursor < bytes.len() && !bytes[cursor].is_ascii_whitespace() && bytes[cursor] != b'#'
        {
            cursor += 1;
        }
        (start < cursor).then(|| String::from_utf8_lossy(&bytes[start..cursor]).into_owned())
    };
    if token().as_deref() != Some("P5") {
        return Err(invalid("expected a binary P5 PGM"));
    }
    let width: usize = token()
        .and_then(|t| t.parse().ok())
        .ok_or_else(|| invalid("invalid width"))?;
    let height: usize = token()
        .and_then(|t| t.parse().ok())
        .ok_or_else(|| invalid("invalid height"))?;
    let maxval: u16 = token()
        .and_then(|t| t.parse().ok())
        .ok_or_else(|| invalid("invalid maximum value"))?;
    if width == 0 || height == 0 || maxval == 0 || maxval > maximum {
        return Err(invalid(&format!(
            "invalid dimensions or maximum value (max {maximum})"
        )));
    }
    cursor += 1; // the single whitespace after the maximum
    let sample = if maxval > 255 { 2 } else { 1 };
    if bytes.len() < cursor + width * height * sample {
        return Err(invalid("raster is truncated"));
    }
    let mut values = Vec::with_capacity(width * height);
    for index in 0..width * height {
        let start = cursor + index * sample;
        values.push(if sample == 2 {
            u16::from(bytes[start]) << 8 | u16::from(bytes[start + 1])
        } else {
            u16::from(bytes[start])
        });
    }
    Ok((width, height, values))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::strip;

    fn map_of(heights: &[u16], water: &[u8]) -> Equirect {
        Equirect {
            width: 4,
            height: 2,
            metres: heights.iter().map(|&raw| raw as i16 as f64).collect(),
            water: water.iter().map(|&raw| f64::from(raw) / 255.0).collect(),
        }
    }

    #[test]
    fn parses_sixteen_bit_big_endian() {
        let pgm = b"P5\n# comment\n4 2\n65535\n\x80\x00\x80\x01\x7f\xff\x00\x00\x00\x01\x00\x02\xff\xff\xff\xfe\x7f\xff\x00\x00";
        let path = std::env::temp_dir().join("ishtaria_worldgen_parse16.pgm");
        std::fs::write(&path, pgm).unwrap();
        let (width, height, values) = read_pgm(&path, 65535).unwrap();
        assert_eq!((width, height), (4, 2));
        assert_eq!(values, vec![32768, 32769, 32767, 0, 1, 2, 65535, 65534]);
        assert!(read_pgm(Path::new("no-such-map.pgm"), 255).is_err());
    }

    #[test]
    fn parses_eight_bit() {
        let pgm = b"P5\n4 2\n255\n\x00\xff\x80\x7f\x00\x01\xfe\xff";
        let path = std::env::temp_dir().join("ishtaria_worldgen_parse8.pgm");
        std::fs::write(&path, pgm).unwrap();
        let (width, height, values) = read_pgm(&path, 255).unwrap();
        assert_eq!((width, height), (4, 2));
        assert_eq!(values, vec![0, 255, 128, 127, 0, 1, 254, 255]);
    }

    #[test]
    fn rejects_wrong_shapes() {
        let pgm = b"P5\n4 3\n255\n\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00";
        let path = std::env::temp_dir().join("ishtaria_worldgen_shape.pgm");
        std::fs::write(&path, pgm).unwrap();
        let mask_path = std::env::temp_dir().join("ishtaria_worldgen_shape_mask.pgm");
        std::fs::write(
            &mask_path,
            b"P5\n4 3\n255\n\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00\x00",
        )
        .unwrap();
        let error = Equirect::load(&path, &mask_path).unwrap_err();
        assert!(error.to_string().contains("twice as wide"));
    }

    #[test]
    fn land_never_sinks_and_water_never_rises() {
        // Raw 32768 decodes to -32768 m: a land abyss is clamped up to just above sea level.
        let map = map_of(&[32768; 8], &[0, 0, 0, 255, 0, 0, 0, 0]);
        assert_eq!(
            map.face_sample(Face::PosX, -0.5, 0.0, "42"),
            strip::LAND_SAMPLE
        );
        // Raw 8192 decodes to +8192 m, but water is clamped down to sea level.
        let map = map_of(&[8192; 8], &[255; 8]);
        assert_eq!(
            map.face_sample(Face::PosZ, 0.0, 0.0, "42"),
            strip::SEA_SAMPLE
        );
    }

    #[test]
    fn round_trips_metres_through_the_strip_with_its_relief() {
        // 4000 m with the seed's relief must decode, within one step, to the height the
        // relief function gives for that point: the server and clients see the same surface.
        let map = map_of(&[4000; 8], &[0; 8]);
        let sample = map.face_sample(Face::PosX, 0.0, 0.0, "42");
        let decoded = f64::from(sample) * strip::STEP_M + strip::FLOOR_M;
        let point = Face::PosX.sphere_point(0.0, 0.0);
        let expected = strip::height_with_relief([point.x, point.y, point.z], 4000.0, "42");
        assert!(
            (decoded - expected).abs() <= strip::STEP_M,
            "{decoded} vs {expected}"
        );
    }

    #[test]
    fn the_pole_face_is_the_north() {
        // All heights 0, all water: latitude and longitude only. With the
        // documented convention the centre of the +Y face is 90 north, the
        // centre of -Y is 90 south, so sampling them must not touch row 1
        // (they cannot, both clamp to their own end of the map).
        let mut map = map_of(&[0; 8], &[255; 8]);
        map.metres = vec![100.0; 4].into_iter().chain(vec![-100.0; 4]).collect();
        let (north, _) = map.sample(89.9, 0.0);
        let (south, _) = map.sample(-89.9, 0.0);
        assert!(
            north > south,
            "north {north} must be higher than south {south}"
        );
    }

    #[test]
    fn east_is_towards_negative_z() {
        // Only the column centred on 45 E is high. On the +X face (Greenwich) east is -Z, which is
        // u > 0 there, so a mirrored frame would put the plateau at u < 0. The relief is at most
        // 12 m, so a 1000 m plateau stays within 13 m of its height.
        let map = map_of(&[0, 0, 1000, 0, 0, 0, 1000, 0], &[0; 8]);
        let decode = |sample: u16| f64::from(sample) * strip::STEP_M + strip::FLOOR_M;
        let east = decode(map.face_sample(Face::PosX, 1.0, 0.0, "42"));
        let west = decode(map.face_sample(Face::PosX, -1.0, 0.0, "42"));
        assert!((east - 1000.0).abs() < 13.0, "east {east}");
        assert!(west.abs() < 0.5, "west {west}");
    }
}
