use num::Complex;
use palette::{convert::TryFromColor, Hsv, RgbHue, Srgb};
use rayon::prelude::*;
use std::{borrow::Cow, fs::File, io::Write};

const X_RES: usize = 2000;
const Y_RES: usize = 2000;
const N: usize = 2000;
const HEADER: &'static str = const_format::formatcp!("P3\n{X_RES} {Y_RES}\n255\n");

fn main() -> std::io::Result<()> {
    let mut final_buf = HEADER.to_string();
    let mut fname_buf = std::path::PathBuf::from("out");

    for max_iterations in 1..N + 1 {
        let mut file = {
            fname_buf.push(format!("out{max_iterations:04}.ppm"));
            let f = File::create(&fname_buf)?;
            fname_buf.pop();

            f
        };

        let s = (-(max_iterations as f64) / 50.0).exp();
        let scale_factors = scale_factors(
            X_RES,
            Y_RES,
            -1.75 * s + 0.1000001009999,
            1.75 * s + 0.1000001009999,
            -1.75 * s + 0.0999989899,
            1.75 * s + 0.0999989899,
        );

        let pixels: Vec<_> = (0..Y_RES)
            .into_par_iter()
            .flat_map(|y| (0..X_RES).into_par_iter().map(move |x| (x, y)))
            .map(|(x, y)| {
                // me when array_zip is unstable
                let scaled =
                    Complex::new((x as f64) / scale_factors[0], (y as f64) / scale_factors[1]);

                julia(
                    scaled,
                    |p: Complex<f64>| {
                        let c = Complex::new(-0.8, 0.156);

                        p.powu(2) + c
                    },
                    max_iterations,
                    3.0,
                )
                .map_or(Cow::Borrowed("0 0 0\n"), |iterations| {
                    let coef = (iterations as f64) / (max_iterations as f64);

                    let hsv = Hsv::new(RgbHue::from_degrees(360.0 * coef - 180.0), 1.0, 1.0);

                    let Srgb {
                        red, green, blue, ..
                    } = Srgb::try_from_color(hsv).unwrap();
                    let [red, green, blue] = [red, green, blue].map(|v| (v * 255.0) as u8);

                    Cow::Owned(format!("{red} {green} {blue}\n"))
                })
            })
            .collect();

        {
            for pixel in pixels {
                final_buf.push_str(pixel.as_ref())
            }

            file.write_all(final_buf.as_bytes())?;

            final_buf.drain(HEADER.len()..);
        }

        eprint!("\rWrote image #{:03}", max_iterations);
    }
    Ok(())
}

fn scale_factors(
    max_x: usize,
    max_y: usize,
    rmin: f64,
    rmax: f64,
    imin: f64,
    imax: f64,
) -> [f64; 2] {
    [
        ((max_x as f64) / (rmax - rmin)) + rmin,
        ((max_y as f64) / (imax - imin)) + imin,
    ]
}

fn julia(
    mut p: Complex<f64>,
    mut f: impl FnMut(Complex<f64>) -> Complex<f64>,
    max_iterations: usize,
    radius: f64,
) -> Option<usize> {
    let radius_sq = radius * radius;

    for i in 0..max_iterations {
        if p.norm_sqr() > radius_sq {
            return Some(i);
        }

        p = f(p);
    }

    None
}
