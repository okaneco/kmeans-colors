use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use palette::{Pixel, Srgb};

use crate::err::CliError;
use kmeans_colors::{Calculate, CentroidData};

/// Parse hex string to Rgb color.
pub fn parse_color(c: &str) -> Result<Srgb<u8>, CliError> {
    let red = u8::from_str_radix(
        match &c.get(0..2) {
            Some(x) => x,
            None => {
                eprintln!("Invalid color: {}", c);
                return Err(CliError::InvalidHex);
            }
        },
        16,
    )?;
    let green = u8::from_str_radix(
        match &c.get(2..4) {
            Some(x) => x,
            None => {
                eprintln!("Invalid color: {}", c);
                return Err(CliError::InvalidHex);
            }
        },
        16,
    )?;
    let blue = u8::from_str_radix(
        match &c.get(4..6) {
            Some(x) => x,
            None => {
                eprintln!("Invalid color: {}", c);
                return Err(CliError::InvalidHex);
            }
        },
        16,
    )?;
    Ok(Srgb::new(red, green, blue))
}

/// Prints colors and percentage of their appearance in an image buffer.
pub fn print_colors<C: Calculate + Copy + Into<Srgb>>(
    show_percentage: bool,
    colors: &[CentroidData<C>],
) -> Result<(), Box<dyn Error>> {
    let mut col = String::new();
    let mut freq = String::new();
    if let Some((last, elements)) = colors.split_last() {
        for elem in elements {
            write!(&mut col, "{:x},", elem.centroid.into().into_format::<u8>())?;
            write!(&mut freq, "{:0.4},", elem.percentage)?;
        }
        writeln!(&mut col, "{:x}", last.centroid.into().into_format::<u8>())?;
        writeln!(&mut freq, "{:0.4}", last.percentage)?;
    }
    print!("{}", col);
    if show_percentage {
        print!("{}", freq);
    }

    Ok(())
}

/// Saves image buffer to file.
pub fn save_image(
    imgbuf: &[u8],
    imgx: u32,
    imgy: u32,
    title: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut w = BufWriter::new(File::create(title)?);
    if title.extension().unwrap() == "png" {
        let encoder = image::png::PngEncoder::new_with_quality(
            w,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::NoFilter,
        );

        // Clean up if file is created but there's a problem writing to it
        match encoder.encode(imgbuf, imgx, imgy, image::ColorType::Rgb8) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}.", err);
                std::fs::remove_file(title)?;
            }
        }
    } else {
        let mut encoder = image::jpeg::JpegEncoder::new_with_quality(&mut w, 90);

        match encoder.encode(imgbuf, imgx, imgy, image::ColorType::Rgb8) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}.", err);
                std::fs::remove_file(title)?;
            }
        }
    };

    Ok(())
}

/// Saves transparent image buffer to file.
pub fn save_image_alpha(
    imgbuf: &[u8],
    imgx: u32,
    imgy: u32,
    title: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut w = BufWriter::new(File::create(title)?);
    if title.extension().unwrap() == "png" {
        let encoder = image::png::PngEncoder::new_with_quality(
            w,
            image::codecs::png::CompressionType::Best,
            image::codecs::png::FilterType::NoFilter,
        );

        // Clean up if file is created but there's a problem writing to it
        match encoder.encode(imgbuf, imgx, imgy, image::ColorType::Rgba8) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}.", err);
                std::fs::remove_file(title)?;
            }
        }
    } else {
        let mut encoder = image::jpeg::JpegEncoder::new_with_quality(&mut w, 90);

        match encoder.encode(imgbuf, imgx, imgy, image::ColorType::Rgba8) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}.", err);
                std::fs::remove_file(title)?;
            }
        }
    };

    Ok(())
}

/// Save palette image file.
pub fn save_palette<C: Calculate + Copy + Into<Srgb>>(
    res: &[CentroidData<C>],
    proportional: bool,
    height: u32,
    width: Option<u32>,
    title: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let len = res.len() as u32;
    let w = match width {
        Some(x) => {
            // Width must be at least `k` pixels wide
            if x < len {
                len
            } else {
                x
            }
        }
        None => height * len,
    };

    let mut imgbuf: image::RgbImage = image::ImageBuffer::new(w, height);

    if !proportional {
        for (x, _, pixel) in imgbuf.enumerate_pixels_mut() {
            let color = res
                .get(
                    (((x as f32 / w as f32) * len as f32 - 0.5)
                        .max(0.0)
                        .min(len as f32))
                    .round() as usize,
                )
                .unwrap()
                .centroid
                .into()
                .into_format()
                .into_raw();
            *pixel = image::Rgb(color);
        }
    } else {
        let mut curr_pos = 0;
        if let Some((last, elements)) = res.split_last() {
            for r in elements.iter() {
                let pix: [u8; 3] = r.centroid.into().into_format().into_raw();
                // Clamp boundary to image width
                let boundary =
                    ((curr_pos as f32 + (r.percentage * w as f32)).round() as u32).min(w);
                for y in 0..height {
                    for x in curr_pos..boundary {
                        imgbuf.put_pixel(x, y, image::Rgb(pix));
                    }
                }
                // If boundary has been clamped, return early
                if boundary == w {
                    return Ok(save_image(&imgbuf.to_vec(), w, height, title)?);
                }
                curr_pos = boundary;
            }
            let pix: [u8; 3] = last.centroid.into().into_format().into_raw();
            for y in 0..height {
                for x in curr_pos..w {
                    imgbuf.put_pixel(x, y, image::Rgb(pix));
                }
            }
        }
    }

    Ok(save_image(&imgbuf.to_vec(), w, height, title)?)
}
