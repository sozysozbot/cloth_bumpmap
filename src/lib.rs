#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}

extern crate image;
use rand::prelude::*;
use std::convert::TryInto;

fn gaussian_blur_asymmetric(
    image: image::RgbImage,
    blur_radius_horizontal: f32,
    blur_radius_vertical: f32,
) -> Option<image::RgbImage> {
    let (width, height) = image.dimensions();
    let mut data = unflatten(&image.into_raw());
    fastblur::gaussian_blur_asymmetric(
        &mut data,
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        blur_radius_horizontal,
        blur_radius_vertical,
    );
    image::RgbImage::from_raw(
        width.try_into().unwrap(),
        height.try_into().unwrap(),
        flatten(&data),
    )
}

fn flatten(data: &Vec<[u8; 3]>) -> Vec<u8> {
    let mut a = vec![];
    for rgb in data.into_iter() {
        a.push(rgb[0]);
        a.push(rgb[1]);
        a.push(rgb[2]);
    }
    a
}

fn unflatten(data: &Vec<u8>) -> Vec<[u8; 3]> {
    let iter = data.chunks(3);
    let mut a = vec![];
    for rgb in iter {
        // unwrap unwrap unwrap
        a.push([rgb[0], rgb[1], rgb[2]]);
    }
    a
}

fn multiply_channel(a: u8, b: u8) -> u8 {
    ((a as f32) * (b as f32) / 255.0) as u8
}
fn multiply_pixel(a: image::Rgb<u8>, b: image::Rgb<u8>) -> image::Rgb<u8> {
    let image::Rgb(a) = a;
    let image::Rgb(b) = b;
    image::Rgb([
        multiply_channel(a[0], b[0]),
        multiply_channel(a[1], b[1]),
        multiply_channel(a[2], b[2]),
    ])
}

pub fn cloth_bumpmap(
    width: u32,
    height: u32,
) -> Result<image::RgbImage, rand_distr::NormalError> {
    /* references :
     * https://fossies.org/linux/gimp/plug-ins/script-fu/scripts/clothify.scm
     * http://oldhome.schmorp.de/marc/pdb/plug_in_noisify.html
     * https://docs.gimp.org/2.10/en/gimp-filter-noise-rgb.html
     */
    let mut rng = thread_rng();
    let mut layer_one = image::RgbImage::new(width, height);
    let distr = rand_distr::Normal::new(0., 0.35)?;
    for (_, _, pixel) in layer_one.enumerate_pixels_mut() {
        let v = rng.sample(distr);
        let a = num::clamp(255.0 * (1. + v), 0.0, 255.0) as u8;
        *pixel = image::Rgb([a, a, a]);
    }

    let blur_strength = 9.0;

    let layer_two = layer_one.clone();
    let mut horizontal = gaussian_blur_asymmetric(layer_one, blur_strength, 1.0).unwrap();
    let vertical = gaussian_blur_asymmetric(layer_two, 1.0, blur_strength).unwrap();

    for (x, y, pixel) in horizontal.enumerate_pixels_mut() {
        *pixel = multiply_pixel(*pixel, *vertical.get_pixel(x, y))
    }

    let mut merged = horizontal;

    /* stretch contrast; this algorithm only works for a grayscale image */
    let (min, max) = get_min_max(&merged.clone().into_raw());
    if min >= max {
        panic!("zero fluctuation: impossible!")
    }
    for (_, _, pixel) in merged.enumerate_pixels_mut() {
        let image::Rgb(data) = *pixel;
        *pixel = image::Rgb([
            (255.0 / ((max - min) as f32) * ((data[0] - min) as f32)) as u8,
            (255.0 / ((max - min) as f32) * ((data[1] - min) as f32)) as u8,
            (255.0 / ((max - min) as f32) * ((data[2] - min) as f32)) as u8,
        ]);
    }

    let distr = rand_distr::Normal::new(0., 0.1)?;
    for (_, _, pixel) in merged.enumerate_pixels_mut() {
        let v = rng.sample(distr);
        let image::Rgb(data) = *pixel;

        *pixel = image::Rgb([
            num::clamp(data[0] as f32 + 255.0 * v, 0.0, 255.0) as u8,
            num::clamp(data[1] as f32 + 255.0 * v, 0.0, 255.0) as u8,
            num::clamp(data[2] as f32 + 255.0 * v, 0.0, 255.0) as u8,
        ]);
    }

    Ok(merged)
}

fn get_min_max(vec: &Vec<u8>) -> (u8, u8) {
    let mut minmax = (255, 0);
    for i in vec {
        if i < &minmax.0 {
            minmax.0 = *i;
        }
        if i > &minmax.1 {
            minmax.1 = *i;
        }
    }
    minmax
}