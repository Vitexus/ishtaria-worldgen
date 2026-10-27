//! Render the six cube faces of a planet as a PGM heightmap strip.
//!
//! Usage:
//!
//! ```sh
//! ishtaria-worldgen [seed] [face_size] > planet.pgm                    # procedural
//! ishtaria-worldgen --seed N --earth HEIGHT.pgm --water MASK.pgm [face_size] > planet.pgm
//! ```
//!
//! `--earth` renders a real planet from a datadisk's equirectangular relief
//! maps instead of noise (see `ishtaria_worldgen::earth`). The seed only sets
//! the local relief added to every land sample (see `ishtaria_worldgen::strip`).
//!
//! Progress is reported on stderr as `PROGRESS <done>/<total>` lines (one per
//! completed face row, `done` from 0 to `total`). stdout remains a clean P5 PGM
//! with maxval 65535 (16-bit, big-endian)
//! so callers that only capture stdout keep working; the admin tool reads both.

use ishtaria_worldgen::{earth::Equirect, strip, Face, Terrain};
use std::io::{self, BufWriter, Write};

fn fail(message: &str) -> ! {
    eprintln!("ishtaria-worldgen: {message}");
    std::process::exit(2);
}

/// Tell interactive callers how far the strip has gone (row `done` of `total`).
fn report_progress(done: usize, total: usize) {
    let mut err = io::stderr().lock();
    let _ = writeln!(err, "PROGRESS {done}/{total}");
    let _ = err.flush();
}

fn main() -> io::Result<()> {
    let mut arguments = std::env::args().skip(1);
    let mut heightmap = None;
    let mut water = None;
    let mut seed_flag = None;
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
            "--seed" => {
                seed_flag = Some(
                    arguments
                        .next()
                        .unwrap_or_else(|| fail("--seed needs a number")),
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
            fail("expected: ishtaria-worldgen --seed N --earth HEIGHT.pgm --water MASK.pgm [face_size]");
        }
        let seed = seed_flag.unwrap_or_else(|| fail("--earth needs --seed, the world's seed"));
        let seed = canonical_seed(&seed);
        let size: usize = match positional.first() {
            None => 256,
            Some(text) => text.parse().unwrap_or_else(|_| fail("invalid face size")),
        };
        let map = Equirect::load(height.as_ref(), mask.as_ref())?;
        write_strip(&mut out, size, |face, u, v| {
            map.face_sample(face, u, v, &seed)
        })?;
        return out.flush();
    }
    if positional.len() > 2 {
        fail("expected: ishtaria-worldgen [seed] [face_size]");
    }
    let seed_text = seed_flag.or_else(|| positional.first().cloned());
    let seed: u64 = match &seed_text {
        None => Terrain::default().seed,
        Some(text) => text
            .parse()
            .unwrap_or_else(|_| fail("the seed must be a number")),
    };
    let relief_seed = seed.to_string();
    let size: usize = positional
        .get(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(256);
    let terrain = Terrain {
        seed,
        ..Terrain::default()
    };
    write_strip(&mut out, size, |face, u, v| {
        let point = face.sphere_point(u, v);
        let metres = terrain.elevation(point);
        let water = metres <= 0.0;
        let metres = if water {
            metres
        } else {
            strip::height_with_relief([point.x, point.y, point.z], metres, &relief_seed)
        };
        strip::encode(metres, water)
    })?;
    out.flush()
}

/// The seed as the server writes it (decimal digits, no sign or leading zeros), so the
/// relief is the same whichever side computes it.
fn canonical_seed(text: &str) -> String {
    text.parse::<u64>()
        .map(|number| number.to_string())
        .unwrap_or_else(|_| fail("the seed must be a number"))
}

/// Writes the six faces as a 16-bit P5 PGM, one row of samples at a time.
fn write_strip(
    out: &mut impl Write,
    size: usize,
    mut sample: impl FnMut(Face, f64, f64) -> u16,
) -> io::Result<()> {
    writeln!(out, "P5\n{} {}\n65535", size * 6, size)?;
    report_progress(0, size);
    let mut row = vec![0u8; size * 6 * 2];
    for j in 0..size {
        for (f, face) in Face::ALL.iter().enumerate() {
            for i in 0..size {
                let u = (i as f64 + 0.5) / size as f64 * 2.0 - 1.0;
                let v = 1.0 - (j as f64 + 0.5) / size as f64 * 2.0;
                let value = sample(*face, u, v).to_be_bytes();
                let at = (f * size + i) * 2;
                row[at..at + 2].copy_from_slice(&value);
            }
        }
        out.write_all(&row)?;
        report_progress(j + 1, size);
    }
    Ok(())
}
