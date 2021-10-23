use num::Complex;

fn pixel_to_point(bounds: (usize, usize), 
                  pixel: (usize, usize), 
                  upper_left: Complex<f32>, 
                  lower_right: Complex<f32>) -> Complex<f32> {

    let (width, height) = (lower_right.re - upper_left.re,
                                   upper_left.im - lower_right.im);

    Complex { 
        re: upper_left.re + ((pixel.0 as f32 / bounds.0 as f32) * width), 
        im: upper_left.im - ((pixel.1 as f32 / bounds.1 as f32) * height) 
    }
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100,200), 
                              (25,175), 
                              Complex { re: -1.0, im: 1.0}, 
                              Complex { re: 1.0, im: -1.0}),
                            Complex { re: -0.5, im: -0.75});

}