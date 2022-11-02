use num::Complex;
use palette::{convert::TryFromColor, Hsv, RgbHue, Srgb};
use rayon::prelude::*;
use std::fs::File;
use std::io::{BufWriter, Write};

const X_RES: i32 = 2000;
const Y_RES: i32 = 2000;
const N: i32 = 2000;

fn main() -> std::io::Result<()> {
    let mut final_buf = String::new();

    for its in 1..N + 1 {
        let f = File::create(format!("out/out{its:04}.ppm"))?;
        let mut w = BufWriter::new(&f);
        writeln!(w, "P3")?;
        writeln!(w, "{} {}", X_RES, Y_RES)?;
        writeln!(w, "255")?;

        let it = its;

        let pixels: Vec<_> = (0..Y_RES)
            .into_par_iter()
            .flat_map(|y| (0..X_RES).into_par_iter().map(move |x| (x, y)))
            .map(|(x, y)| -> String {
                let s = (-(it as f64) / 50.0).exp();
                let scaled = scale(
                    x,
                    y,
                    X_RES,
                    Y_RES,
                    -1.75 * s + 0.1000001009999,
                    1.75 * s + 0.1000001009999,
                    -1.75 * s + 0.0999989899,
                    1.75 * s + 0.0999989899,
                );
                let i = julia(
                    scaled,
                    |p: Complex<f64>| {
                        let c = Complex::new(-0.8, 0.156);
                        p.powu(2) + c
                    },
                    0,
                    it,
                    3.0,
                );
                let coef = (i as f64) / (it as f64);

                let hsv = Hsv::new(
                    RgbHue::from_degrees(360.0 * coef - 180.0),
                    1.0,
                    if i == -1 { 0.0 } else { 1.0 },
                );

                let Srgb {
                    red, green, blue, ..
                } = Srgb::try_from_color(hsv).unwrap();
                let [red, green, blue] = [red, green, blue].map(|v| (v * 255.0) as u8);

                format!("{red} {green} {blue}\n")
            })
            .collect();

        for pixel in pixels {
            final_buf.push_str(pixel.as_str())
        }

        w.write_all(final_buf.as_bytes())?;

        eprint!("\rWrote image #{:03}", its);
    }
    Ok(())
}

fn scale(
    x: i32,
    y: i32,
    xm: i32,
    ym: i32,
    rmin: f64,
    rmax: f64,
    imin: f64,
    imax: f64,
) -> Complex<f64> {
    let r = (x as f64) / ((xm as f64) / (rmax - rmin)) + rmin;
    let i = (y as f64) / ((ym as f64) / (imax - imin)) + imin;
    Complex::new(r, i)
}

fn julia(
    p: Complex<f64>,
    f: fn(Complex<f64>) -> Complex<f64>,
    i: i32,
    max: i32,
    radius: f64,
) -> i32 {
    if i >= max {
        return -1;
    }

    if p.norm_sqr() > radius * radius {
        return i;
    }

    julia(f(p), f, i + 1, max, radius)
}
