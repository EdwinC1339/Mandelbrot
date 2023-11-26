use rayon::prelude::*;
use std::collections::{BTreeSet, HashMap};

use image::{ImageBuffer, Rgb};
use lerp::Lerp;
use num_complex::{Complex64, ComplexFloat};
use num_rational::{Ratio, Rational64};
use num_traits::{Float, ToPrimitive};
use ordered_float::NotNan;

type C64 = Complex64;
const ITERMAX: i32 = 100;

#[derive(Debug, Copy, Clone)]
struct MathyColor<F> {
    r: F,
    g: F,
    b: F,
}

impl<F> MathyColor<F>
where
    F: Float + ToPrimitive,
{
    fn new(r: F, g: F, b: F) -> Self {
        Self { r, g, b }
    }

    // fn from(other: Rgb<u8>) -> Self {
    //     Self {
    //         r: F::from(other.0[0]).unwrap(),
    //         g: F::from(other.0[1]).unwrap(),
    //         b: F::from(other.0[2]).unwrap()
    //     }
    // }

    fn from_ref(other: &Rgb<u8>) -> Self {
        Self {
            r: F::from(other.0[0]).unwrap(),
            g: F::from(other.0[1]).unwrap(),
            b: F::from(other.0[2]).unwrap(),
        }
    }

    fn unwrap(&self) -> Rgb<u8> {
        Rgb([
            self.r.round().to_u8().unwrap(),
            self.g.round().to_u8().unwrap(),
            self.b.round().to_u8().unwrap(),
        ])
    }
}

impl<F> std::ops::Add for MathyColor<F>
where
    F: Float,
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        MathyColor::new(self.r + rhs.r, self.g + rhs.g, self.b + rhs.b)
    }
}

impl<F> std::ops::Mul<F> for MathyColor<F>
where
    F: Float,
{
    type Output = Self;
    fn mul(self, rhs: F) -> Self::Output {
        MathyColor::new(self.r * rhs, self.g * rhs, self.b * rhs)
    }
}

#[derive(Debug, Clone)]
struct Palette {
    /* Collection of x, color pairs with 0 <= x <= 1 */
    _keys: BTreeSet<NotNan<f64>>,
    _key_map: HashMap<NotNan<f64>, Rgb<u8>>,
}

impl Palette {
    fn new() -> Palette {
        Palette {
            _keys: BTreeSet::new(),
            _key_map: HashMap::new(),
        }
    }

    fn add_col(&mut self, key: NotNan<f64>, new_color: &Rgb<u8>) {
        self._keys.insert(key);
        self._key_map.insert(key, new_color.to_owned());
    }

    fn get_color(&self, k: NotNan<f64>) -> Rgb<u8> {
        let mut prev_key: &NotNan<f64> = self._keys.first().unwrap();

        if k <= *prev_key {
            return *self._key_map.get(prev_key).unwrap();
        }

        for cur_key in &self._keys {
            if cur_key >= &k {
                let prev_color: &Rgb<u8> = self._key_map.get(prev_key).unwrap();
                let cur_color: &Rgb<u8> = self._key_map.get(cur_key).unwrap();
                let interpolation_factor: NotNan<f64> = (k - prev_key) / (cur_key - prev_key);
                let prev_color_mathy: MathyColor<f64> = MathyColor::from_ref(prev_color);
                let cur_color_mathy: MathyColor<f64> = MathyColor::from_ref(cur_color);
                return prev_color_mathy
                    .lerp(cur_color_mathy, *interpolation_factor)
                    .unwrap();
            }
            prev_key = cur_key;
        }
        panic!()
    }
}
fn main() {
    let width: i32 = 3840;
    let height: i32 = 2160;
    let threshold: f64 = 2.0;
    let velocities: Vec<Vec<i32>> = get_divergence_vel(width, height, threshold);
    let mut palette: Palette = Palette::new();

    let cols: Vec<Rgb<u8>> = vec![
        Rgb([229, 208, 204]),
        Rgb([229, 208, 204]),
        Rgb([23, 33, 33]),
    ];

    let col_keys: Vec<NotNan<f64>> = vec![
        NotNan::try_from(0.0).unwrap(),
        NotNan::try_from(0.15).unwrap(),
        NotNan::try_from(1.0).unwrap(),
    ];

    for (k, v) in std::iter::zip(col_keys.into_iter(), cols.into_iter()) {
        palette.add_col(k, &v);
    }

    let mut imgbuf: ImageBuffer<_, Vec<_>> =
        ImageBuffer::new(width.try_into().unwrap(), height.try_into().unwrap());
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let velocity: i32 = velocities[y as usize][x as usize];
        *pixel = gradient(velocity, &palette);
    }

    imgbuf
        .save(format!("mandelbrot{width}x{height}.png"))
        .unwrap();
}

fn transform(base: C64) -> C64 {
    let base = (base + C64::new(0.0, -0.53)) * C64::i();
    if base.abs() == 0.0 {
        C64::new(10.0, 10.0)
    } else {
        0.4 / base
    }
}

fn get_divergence_vel(width: i32, height: i32, threshold: f64) -> Vec<Vec<i32>> {
    let aspect_ratio: Rational64 = Rational64::new(width as i64, height as i64);
    let y_scale: Rational64 = Rational64::new(112, 100);
    let x_scale: Rational64 = y_scale * aspect_ratio;

    let grid: Vec<_> = (0..height)
        .map(|h: i32| -> Vec<_> {
            let y = Rational64::new(2 * h as i64, height as i64) * y_scale - y_scale;
            (0..width)
                .map(|w: i32| -> (Rational64, Rational64) {
                    let x = Rational64::new(2 * w as i64, width as i64) * x_scale - x_scale;
                    (x, y)
                })
                .collect()
        })
        .collect();

    grid.into_par_iter()
        .map(|row| -> Vec<i32> {
            row.into_par_iter()
                .map(|c: (Ratio<i64>, Ratio<i64>)| -> i32 {
                    let (re, im) = c;
                    let re: f64 = re.to_f64().expect("Couldn't cast to float.");
                    let im: f64 = im.to_f64().expect("Couldn't cast to float");
                    let c = transform(C64::new(re, im));
                    diverges_in(c, threshold)
                })
                .collect()
        })
        .collect()
}

fn diverges_in(c: C64, threshold: f64) -> i32 {
    let mut count: i32 = 0;
    let mut accumulator: C64 = c;
    let mut d1: C64 = C64::new(0.0, 0.0);
    let mut d2: C64;

    while accumulator.abs() < threshold && count < ITERMAX {
        let next_accumulator = next_mandelbrot(accumulator, c);
        let d = next_accumulator - accumulator;
        d2 = d1;
        d1 = d;
        let second_d = d2 - d1;
        if second_d.abs() < 0.05 {
            //return ITERMAX
        }
        accumulator = next_accumulator;
        count += 1;
    }

    count
}

fn next_mandelbrot(z: C64, c: C64) -> C64 {
    z * z + c
}

fn gradient(velocity: i32, palette: &Palette) -> Rgb<u8> {
    let norm: NotNan<f64> = NotNan::try_from(velocity as f64 / ITERMAX as f64).unwrap();
    palette.get_color(norm)
}
