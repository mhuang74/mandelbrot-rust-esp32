use num::Complex;
use anyhow::Result;

fn pixel_to_point(bounds: (usize, usize), 
                  pixel: (usize, usize), 
                  upper_left: Complex<f32>, 
                  lower_right: Complex<f32>) -> Result<Complex<f32>> {

    let (width, height) = (lower_right.re - upper_left.re,
                                   upper_left.im - lower_right.im);

    let result = Complex { 
        re: upper_left.re + ((pixel.0 as f32 / bounds.0 as f32) * width), 
        im: upper_left.im - ((pixel.1 as f32 / bounds.1 as f32) * height) 
    };

    Ok(result)
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,200), 
                              (25,175), 
                              Complex { re: -1.0, im: 1.0}, 
                              Complex { re: 1.0, im: -1.0})?,
                            Complex { re: -0.5, im: -0.75});

}

/// Try to determine if complex point `c` is in the Mandelbrot set, using at most `limit` iterations to decide.
fn escape_time(c: Complex<f32>, limit: usize) -> Option<usize> {

    let mut z = Complex { re: 0.0, im: 0.0 };

    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }

        z = z * z + c;
    }

    None
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
/// 
/// The `bounds` argument gives the width and height of the buffer `pixels`
/// which holds one grayschale pixel per byte. The `upper_left` and `lower_left`
/// arguments specify points on the complex plane corresponding to the upper-left 
/// and lower-right corners of the pixel buffer.
pub fn render(pixels: &mut Vec<u8>, 
    bounds: (usize, usize), 
    upper_left: Complex<f32>, 
    lower_right: Complex<f32>) -> Result<()> {

    assert!(pixels.capacity() >= bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right)?;
            let val = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8
            };
            pixels.push(val);
        };
    };

    Ok(())
}

