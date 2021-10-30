extern crate clap;
extern crate image;

mod error;
use crate::error::{AppResult, AppError};

use clap::{App, Arg};
use image::io::Reader;
use image::{GrayImage, RgbaImage, ImageBuffer, GenericImage};
use std::cmp::min;

fn main() -> AppResult<()> {
    let matches =
        App::new("rustic_eye")
            .version("1.0.0")
            .author("Andrew Brooks")
            .about("Add height map-based stereoscopy to drawings")
            .arg(Arg::with_name("size")
                 .short("s")
                 .long("size")
                 .value_name("size")
                 .help("maximum displacement (px, or use % for pct width)")
                 .default_value("2%")
                 .takes_value(true))
            .arg(Arg::with_name("base")
                 .long("base")
                 .value_name("base")
                 .help("base image")
                 .required(true)
                 .index(1))
            .arg(Arg::with_name("map")
                 .long("map")
                 .value_name("height_map")
                 .help("height map image")
                 .required(true)
                 .index(2))
            .arg(Arg::with_name("output")
                 .short("o")
                 .long("output")
                 .value_name("output")
                 .help("output image (inferred left channel)")
                 .required(true))
            .arg(Arg::with_name("type")
                 .short("t")
                 .long("type")
                 .value_name("type")
                 .help("output image channel layout")
                 .required(true)
                 .takes_value(true)
                 .possible_values(&["RL", "LR", "L"])
                 .default_value("RL"))
            .get_matches();

    let base = Reader::open(matches.value_of("base").unwrap())?
        .with_guessed_format()?
        .decode()?
        .to_rgba8();

    let map = Reader::open(matches.value_of("map").unwrap())?
        .with_guessed_format()?
        .decode()?
        .grayscale()
        .to_luma8();
   
    let base_dims = base.dimensions();
    let map_dims = map.dimensions();

    if base_dims != map_dims {
        return Err(AppError::BadImage("base/map images have different dimensions".to_string()));
    }

    let effect_size = parse_effect_size(matches.value_of("size").unwrap(), base.width())?;
    let left_channel = infer_left_channel(&base, &map, effect_size);

    let output_img = match matches.value_of("type").unwrap() {
        "LR" => horiz_stack(&left_channel, &base)?,
        "RL" => horiz_stack(&base, &left_channel)?,
        "L"  => left_channel,
        _    => return Err(AppError::ArgError("type".to_string()))
    };
    output_img.save(matches.value_of("output").unwrap())?;
    return Ok(());
}

fn parse_effect_size(specified: &str, width: u32) -> AppResult<u32> {
    // Could be a number of pixels, number of pixels + 'px', or a percent of width.
    let (mult, numeric_str) = match specified {
        s if s.ends_with("%")  => (width as f32 / 100.0, s.strip_suffix("%").unwrap()),
        s if s.ends_with("px") => (1.0, s.strip_suffix("px").unwrap()),
        s                      => (1.0, s)
    };
    match numeric_str.parse::<f32>() {
        Ok(num) if num >  0.0 =>
            Ok((mult * num) as u32),
        Ok(num) if num <= 0.0 =>
            Err(AppError::ArgError("size must be strictly positive".to_string())),
        Ok(num) if !num.is_normal() =>
            Err(AppError::ArgError("size must be a normal float".to_string())),
        _ =>
            Err(AppError::ArgError("size must be a number of pixels or x%".to_string()))
    }
}

fn horiz_stack(left: &RgbaImage, right: &RgbaImage) -> AppResult<RgbaImage> {
    if left.height() != right.height() {
        // We shouldn't get here, but, y'know, just in case
        return Err(AppError::BadImage("image heights mismatched".to_string()));
    }
    let mut output: RgbaImage = ImageBuffer::new(left.width() + right.width(), left.height());
    output.copy_from(left, 0, 0)?;//left.height() - 1)?;
    output.copy_from(right, left.width(), 0)?;
    return Ok(output);
}

fn infer_left_channel(base: &RgbaImage, map: &GrayImage, effect_size: u32) -> RgbaImage {
    let (width, height) = base.dimensions();
    let mut output: RgbaImage = ImageBuffer::new(width, height);
    // Need to track which pixels are accounted for in the output.
    // We'll need to fill in any not accounted for later on.
    let mut present = vec![false; width as usize];
    for i in 0..base.height() {
        present.fill(false);
        // Place each input pixel at the correct output location by shifting it
        // according to the height map.
        //
        // Reverse order assures that we avoid clobbering a heavily-shifted
        // pixel with a barely-shifted one to its right, which can really fuck
        // up the adjusted image.
        for j in (0..base.width()).rev() {
            let shift: u32 = ((effect_size as f32) * (map.get_pixel(j,i).0[0] as f32) / 255.0) as u32;
            let j2 = min(j + shift, width-1);
            output.put_pixel(j2, i, *base.get_pixel(j,i));
            present[j2 as usize] = true;
        }
        // Some pixels in the output image may never have been written to.
        // Make a decent guess as to what they should be by 'pulling over'
        // their neighbor from the left.
        let mut last_pix = *base.get_pixel(0, i);
        for (j, &has_pix) in present.iter().enumerate() {
            if has_pix {
                last_pix = *output.get_pixel(j as u32, i);
            } else {
                output.put_pixel(j as u32, i, last_pix);
            }
        }
    }
    return output;
}
