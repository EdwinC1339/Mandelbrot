use std::collections::{HashMap, BTreeSet};

use num_complex::{Complex64, ComplexFloat};
use lerp::{Lerp, LerpIter};
use image::{ImageBuffer, Rgb};
use num_traits::{Float, ToPrimitive};
use ordered_float::NotNan;

type C64 = Complex64;
const ITERMAX: i32 = 100;

#[derive(Debug, Copy, Clone)]
struct MathyColor<F> {
    r: F,
    g: F,
    b: F
}

impl<F> MathyColor<F> 
where F: Float + ToPrimitive
{
    fn new(r: F, g: F, b: F) -> Self {
        Self {r, g, b}
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
            b: F::from(other.0[2]).unwrap()
        }
    }

    fn unwrap(&self) -> Rgb<u8> {
        Rgb([self.r.round().to_u8().unwrap(), self.g.round().to_u8().unwrap(), self.b.round().to_u8().unwrap()])
    }

}

impl<F> std::ops::Add for MathyColor<F> 
where F: Float
{
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        MathyColor::new(
            self.r + rhs.r,
            self.g + rhs.g,
            self.b + rhs.b
        )
    }
}

impl<F> std::ops::Mul<F> for MathyColor<F>
where F: Float
{
    type Output = Self;
    fn mul(self, rhs: F) -> Self::Output {
        MathyColor::new(
            self.r * rhs,
            self.g * rhs,
            self.b * rhs
        )
    }
}

#[derive(Debug, Clone)]
struct Palette {
    /* Collection of x, color pairs with 0 <= x <= 1 */
    _keys: BTreeSet<NotNan<f64>>,
    _key_map: HashMap<NotNan<f64>, Rgb<u8>>
}

impl Palette {
    fn new() -> Palette{
        Palette {
            _keys: BTreeSet::new(),
            _key_map: HashMap::new()
        }
    }

    fn add_col(&mut self, key: NotNan<f64>, new_color: &Rgb<u8>) {
        self._keys.insert(key);
        self._key_map.insert(key, new_color.to_owned());
    }

    fn get_color(&self, k: NotNan<f64>) -> Rgb<u8> {
        let mut prev_key: &NotNan<f64> = self._keys.first().unwrap();

        for cur_key in &self._keys {
            if cur_key >= &k {
                let prev_color: &Rgb<u8> = self._key_map.get(prev_key).unwrap();
                let cur_color: &Rgb<u8> = self._key_map.get(cur_key).unwrap();
                let t: NotNan<f64> = (k - prev_key) / (cur_key - prev_key);
                let prev_color_mathy: MathyColor<f64> = MathyColor::from_ref(prev_color);
                let cur_color_mathy: MathyColor<f64> = MathyColor::from_ref(cur_color);
                return prev_color_mathy.lerp(cur_color_mathy, *t).unwrap();
            }
            prev_key = cur_key;
        }
        panic!()
    }

}
fn main() {
    let width: i32 = 1920;
    let height: i32 = 1080;
    let threshold: f64 = 300.0;
    let velocities: Vec<Vec<i32>> = get_divergence_vel(width, height, threshold);
    let mut palette: Palette = Palette::new();

    let cols: Vec<Rgb<u8>> = vec![
        Rgb([72, 67, 73]),
        Rgb([72, 67, 73]),
        Rgb([255, 89, 100]),
        Rgb([72, 67, 73])
        ];
    
    let col_keys: Vec<NotNan<f64>> = vec![
        NotNan::try_from(0.0).unwrap(),
        NotNan::try_from(0.1).unwrap(),
        NotNan::try_from(0.5).unwrap(),
        NotNan::try_from(1.0).unwrap()
    ];

    for (k, v) in std::iter::zip(col_keys.into_iter(), cols.into_iter()) {
        palette.add_col(k, &v);
    }

    let mut imgbuf: ImageBuffer<_, Vec<_>> = ImageBuffer::new(width.try_into().unwrap(), height.try_into().unwrap());
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let velocity: i32 = velocities[y as usize][x as usize];
        *pixel = gradient(velocity, &palette);
    }

    imgbuf.save(format!("mandelbrot{width}x{height}.png")).unwrap();
}

fn get_divergence_vel(width: i32, height: i32, threshold: f64) -> Vec<Vec<i32>> {
    let mut rows: Vec<Vec<i32>> = Vec::new();
    let aspect_ratio: f64 = width as f64 / height as f64;
    let bottom: f64 = -3.0;
    let left: f64 = bottom / aspect_ratio;
    
    for major_axis in left.lerp_iter(-left, height as usize) {
        let mut row: Vec<i32> = Vec::new();
        for minor_axis in bottom.lerp_iter(-bottom, width as usize) {
            let c: C64 = C64::new(minor_axis, major_axis);
            let divergence_vel: i32 = diverges_in(c, threshold);
            row.push(divergence_vel);
        }
        rows.push(row);
    }

    rows
}

fn diverges_in(c: C64, threshold: f64) -> i32 {
    let mut count: i32 = 0;
    let mut accumulator: C64 = c;

    while accumulator.abs() < threshold && count < ITERMAX {
        accumulator = next_mandelbrot(accumulator, c);
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