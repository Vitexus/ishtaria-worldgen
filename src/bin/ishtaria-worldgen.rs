//! Render the six cube faces of a planet as a PGM heightmap strip.
//!
//! Usage:
//!
//! ```sh
//! ishtaria-worldgen [seed] [face_size] > planet.pgm                    # procedural
//! ishtaria-worldgen --earth HEIGHT.pgm --water MASK.pgm [face_size] > planet.pgm
//! ```
//!
//! `--earth` renders a real planet from a datadisk's equirectangular relief
//! maps instead of noise (see `ishtaria_worldgen::earth`); the seed plays no
//! part in it.

use ishtaria_worldgen::{earth::Equirect, Face, Terrain};
use std::io::{self, BufWriter, Write};

fn fail(message: &str) -> ! {
    eprintln!("ishtaria-worldgen: {message}");
    std::process::exit(2);
}

fn main() -> io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let mut heightmap = None;
    let mut water = None;
    let mut positional = Vec::new();
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--earth" => {
                heightmap = Some(
                    arguments
                        .next()
                        .unwrap_or_else(|| fail("--earth needs a file")),
                );
            }
            "--water" => {
                water = Some(
                    arguments
                        .next()
                        .unwrap_or_else(|| fail("--water needs a file")),
                );
            }
            _ => positional.push(argument),
        }
    }
    match (&heightmap, &water) {
        (Some(_), None) => fail("--earth also needs --water"),
        (None, Some(_)) => fail("--water also needs --earth"),
        _ => {}
    }
    let mut out = BufWriter::new(io::stdout().lock());
    if let (Some(height), Some(mask)) = (&heightmap, &water) {
        if positional.len() > 1 {
            fail("expected: ishtaria-worldgen --earth HEIGHT.pgm --water MASK.pgm [face_size]");
        }
        let size: usize = match positional.first() {
            None => 256,
            Some(text) => text.parse().unwrap_or_else(|_| fail("invalid face size")),
        };
        let map = Equirect::load(height.as_ref(), mask.as_ref())?;
        writeln!(out, "P5\n{} {}\n255", size * 6, size)?;
        let mut row = vec![0u8; size * 6];
        for j in 0..size {
            for (f, face) in Face::ALL.iter().enumerate() {
                for i in 0..size {
                    let u = (i as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                    let v = 1.0 - (j as f64 + 0.5) / size as f64 * 2.0;
                    row[f * size + i] = map.face_byte(*face, u, v);
                }
            }
            out.write_all(&row)?;
        }
        return out.flush();
    }
    if positional.len() > 2 {
        fail("expected: ishtaria-worldgen [seed] [face_size]");
    }
    let seed = positional
        .first()
        .and_then(|s| s.parse().ok())
        .unwrap_or(Terrain::default().seed);
    let size: usize = positional
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);
    let terrain = Terrain {
        seed,
        ..Terrain::default()
    };

    writeln!(out, "P5\n{} {}\n255", size * 6, size)?;
    let mut row = vec![0u8; size * 6];
    for j in 0..size {
        for (f, face) in Face::ALL.iter().enumerate() {
            for i in 0..size {
                let u = (i as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                let v = 1.0 - (j as f64 + 0.5) / size as f64 * 2.0;
                let e = terrain.elevation(face.sphere_point(u, v)) / terrain.amplitude_m;
                row[f * size + i] = ((e * 0.5 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
            }
        }
        out.write_all(&row)?;
    }
    out.flush()
}
