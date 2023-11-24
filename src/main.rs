use num_complex::{Complex64, ComplexFloat};
use lerp::LerpIter;
use image::{ImageBuffer, Rgb};

type C64 = Complex64;
const ITERMAX: i32 = 100;


fn main() {
    let width: i32 = 1920;
    let height: i32 = 1080;
    let threshold: f64 = 300.0;
    let velocities: Vec<Vec<i32>> = get_divergence_vel(width, height, threshold);

    let mut imgbuf: ImageBuffer<_, Vec<_>> = ImageBuffer::new(width.try_into().unwrap(), height.try_into().unwrap());
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let velocity: i32 = velocities[y as usize][x as usize];
        *pixel = gradient(velocity);
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

fn gradient(velocity: i32) -> Rgb<u8> {
    let norm: f32 = velocity as f32 / ITERMAX as f32 * 256.0;
    let r: f32 = norm;
    let g: f32 = norm;
    let b: f32 = norm;
    Rgb([r as u8, g as u8, b as u8])
}