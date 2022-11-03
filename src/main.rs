use num::Complex;
use palette::{convert::TryFromColor, Hsv, RgbHue, Srgb};
use rayon::prelude::*;
use std::{
    borrow::Cow,
    fs::File,
    io::{self, Write},
};

const X_RES: usize = 2000;
const Y_RES: usize = 2000;
const N: usize = 2000;
const HEADER: &'static str = const_format::formatcp!("P3\n{X_RES} {Y_RES}\n255\n");
const OUT_PREFIX: &'static str = "out/out";

fn main() -> io::Result<()> {
    let thread_count = rayon::current_num_threads();
    let iters = 1..N + 1;
    let progress = {
        let bar = indicatif::ProgressBar::new(N as u64);

        let n_width = (N as f64).log10() as usize;

        bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .progress_chars("#=-")
            .template(format!("[{{elapsed_precise}}] [{{bar:40.cyan/blue}}] {{pos:>{n_width}}}/{{len:{n_width}}} {{msg}}").as_str())
            .expect("bad progress bar style")
        );

        bar
    };

    let result: Result<Vec<()>, _> = (0..thread_count)
        .into_par_iter()
        .map(|iter_offset| iters.clone().skip(iter_offset).step_by(thread_count))
        .map(|iters_thread| -> io::Result<()> {
            let mut buf_ppm = HEADER.to_string();
            let mut buf_fname = OUT_PREFIX.to_string();

            for max_iters in iters_thread {
                let mut file = {
                    buf_fname.push_str(format!("{max_iters:04}").as_str());
                    buf_fname.push_str(".ppm");
                    let f = File::create(&buf_fname)?;
                    buf_fname.drain(OUT_PREFIX.len()..);

                    f
                };

                let s = (-(max_iters as f64) / 50.0).exp();
                let scaler = scaler(
                    X_RES,
                    Y_RES,
                    -1.75 * s + 0.1000001009999,
                    1.75 * s + 0.1000001009999,
                    -1.75 * s + 0.0999989899,
                    1.75 * s + 0.0999989899,
                );

                let pixels: Vec<_> = (0..Y_RES)
                    .into_par_iter()
                    .flat_map(|y| (0..X_RES).into_par_iter().map(move |x| [x, y]))
                    .map(|point| {
                        let scaled = scaler(point);

                        julia(
                            scaled,
                            |p: Complex<f64>| {
                                let c = Complex::new(-0.8, 0.156);

                                p.powu(2) + c
                            },
                            max_iters,
                            3.0,
                        )
                        .map_or(Cow::Borrowed("0 0 0\n"), |iterations| {
                            let coef = (iterations as f64) / (max_iters as f64);

                            let hsv =
                                Hsv::new(RgbHue::from_degrees(360.0 * coef - 180.0), 1.0, 1.0);

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
                        buf_ppm.push_str(pixel.as_ref())
                    }

                    file.write_all(buf_ppm.as_bytes())?;

                    buf_ppm.drain(HEADER.len()..);
                }

                progress.inc(1);
            }

            Ok(())
        })
        .collect();

    match result {
        Ok(_) => {
            progress.finish();

            Ok(())
        }
        Err(e) => {
            progress.abandon();

            Err(e)
        }
    }
}

fn scaler(
    max_x: usize,
    max_y: usize,
    rmin: f64,
    rmax: f64,
    imin: f64,
    imax: f64,
) -> impl Fn([usize; 2]) -> Complex<f64> {
    let [factor_re, factor_im] = [
        ((max_x as f64) / (rmax - rmin)).recip(),
        ((max_y as f64) / (imax - imin)).recip(),
    ];

    move |point| {
        let [x, y] = point.map(|v| v as f64);
        let comps = [[x, factor_re, rmin], [y, factor_im, imin]];
        let [re, im] = comps.map(|[point, fac, off]| point * fac + off);

        Complex { re, im }
    }
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
