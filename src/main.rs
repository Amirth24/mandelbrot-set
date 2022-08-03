extern crate num;
use std::str::FromStr;

use num::Complex;

fn escape_time(c: Complex<f64>, limit: u32) -> Option<u32> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z * z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }
    None
}


// Commandline Parsing utils
fn parse_pair<T: FromStr>(s: &str, seperator: char) -> Option<(T, T)> {
    match s.find(seperator) {
        None => None,
        Some(i) => match (T::from_str(&s[..i]), T::from_str(&s[i + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

// Mapping Pixel to point

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, height) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * height / bounds.1 as f64,
    }
}

// Image
extern crate image;
use image::codecs::png::PngEncoder;
use image::{ColorType, ImageEncoder};
use std::fs::File;

fn render(
    pixel: &mut Vec<Vec<u8>>,
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixel.len() == bounds.0 * bounds.1);
    for r in 0..bounds.0 {
        for c in 0..bounds.1 {
            let point = pixel_to_point(bounds, (c, r), upper_left, lower_right);
            
            pixel[r][c] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
            
        }
    }
}

use std::path::Path;
fn write_image(
    filename: &str,
    pixel: &mut [u8],
    bounds: (usize, usize),
) -> Result<(), std::io::Error> {
    let path = Path::new("assets");
    let output = match File::create(path.join(filename)) {
        Ok(f) => f,
        Err(e) => {
            return Err(e);
        }
    };
    let encoder = PngEncoder::new(output);
    encoder
        .write_image(&pixel, bounds.0 as u32, bounds.1 as u32, ColorType::L8)
        .unwrap();

    Ok(())
}

use std::io::Write;
use rayon::prelude::*;
fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 5 {
        writeln!(
            std::io::stderr(),
            "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT"
        )
        .unwrap();
        writeln!(
            std::io::stderr(),
            "Example {} mandelbrot.png 1000x1000 -2.0,3.0 2.0,-3.0",
            args[0]
        )
        .unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x').expect("Error parsing image dimensions");

    let upper_left = parse_complex(&args[3]).expect("Error parsing upper left point of image");

    let lower_right =
        parse_complex(&args[4]).expect("Error parsing lower right point of the image");

    let mut pixels = vec![vec![0u8; bounds.0]; bounds.1];

    let thread = 8;
    let rows_per_band = bounds.1 / thread + 1;
    
    let mut bands: Vec<Vec<Vec<u8>>> = pixels.chunks_mut(rows_per_band ).map(|x| x.to_vec()).collect();
    assert!(bands.len() != 1); 
    println!("No. of bands: {}, rows per bands {} length of last band: {}", bands.len(), rows_per_band, &bands.last().unwrap().len());
    bands.par_iter_mut().enumerate().for_each(|(i, band)|{
        let top = rows_per_band * i;
        let height = band.len() ;
        let band_bounds = (band[0].len(), height);
        let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
        let band_lower_right = pixel_to_point(bounds, (band[0].len(), height), upper_left, lower_right);
        render(band, band_bounds, band_upper_left, band_lower_right);
    });


    let mut pixels = vec_2d_to_1d(pixels);
    write_image(&args[1], pixels.as_mut_slice(), bounds).expect("Error while writing image");
}

fn vec_2d_to_1d(v: Vec<Vec<u8>>) -> Vec<u8>{
    let mut r = vec![0u8;v.len()*v[0].len()];
    for i in 0..v.len(){
        for j in 0..v[i].len(){
            r[i*v[i].len() + j] = v[i][j];
        }
    }
    r
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_parse_pair() {
        assert_eq!(parse_pair::<i32>("12x-34", 'x'), Some((12, -34)));
        assert_eq!(parse_pair::<u32>("12", ' '), None);
        assert_eq!(parse_pair::<f64>("12.2 6.46", ' '), Some((12.2, 6.46)));
        assert_eq!(parse_pair::<f64>("1.3.22.2", ','), None);
        assert_eq!(parse_pair::<f64>("3.14,6.28", ','), Some((3.14, 6.28)))
    }

    #[test]
    fn test_pixel_to_point() {
        assert_eq!(
            pixel_to_point(
                (100, 100),
                (50, 50),
                Complex { re: -1.0, im: 1.0 },
                Complex { re: 1.0, im: -1.0 }
            ),
            Complex { re: 0.0, im: 0.0 }
        );
        assert_ne!(
            pixel_to_point(
                (100, 100),
                (0, 50),
                Complex { re: -1.0, im: 1.0 },
                Complex { re: 1.0, im: -1.0 }
            ),
            Complex { re: 1.0, im: 0.0 }
        );
        assert_eq!(
            pixel_to_point(
                (100, 100),
                (0, 50),
                Complex { re: -1.0, im: 1.0 },
                Complex { re: 1.0, im: -1.0 }
            ),
            Complex { re: -1.0, im: 0.0 }
        );
    }

    #[test]
    fn test_parse_complex() {
        assert_eq!(parse_complex("0.0,0.0"), Some(Complex { re: 0.0, im: 0.0 }));
        assert_eq!(parse_complex("9.2,4.0"), Some(Complex { re: 9.2, im: 4.0 }));
        assert_eq!(parse_complex("343.3"), None)
    }

    #[test]
    fn test_vec_2d_to_1d(){
        assert_eq!(vec_2d_to_1d(vec![vec![3,51,35,45,11,3],vec![2,12,34,66,74,45]]), vec![3,51,35,45,11,3,2,12,34,66,74,45]);
        assert_eq!(vec_2d_to_1d(vec![vec![1,2,3,4],vec![2,12]]), vec![1,2,3,4,2,12]);
        assert_ne!(vec_2d_to_1d(vec![vec![1,2,3,4],vec![2,3,12]]), vec![1,2,3,4,2,12]);
    }
}
