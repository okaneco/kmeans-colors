use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use palette::{Lab, Pixel, Srgb};

use crate::err::CliError;

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

/// Prints the Lab colors and percentage of their appearance in an image buffer.
pub fn print_colors_lab(
    show_percentage: bool,
    colors: &Vec<(Lab, f32, u8)>,
) -> Result<(), Box<dyn Error>> {
    let mut col = String::new();
    let mut freq = String::new();
    if let Some((last, elements)) = colors.split_last() {
        for elem in elements {
            write!(&mut col, "{:x},", Srgb::from(elem.0).into_format::<u8>())?;
            write!(&mut freq, "{:0.4},", elem.1)?;
        }
        write!(&mut col, "{:x}\n", Srgb::from(last.0).into_format::<u8>())?;
        write!(&mut freq, "{:0.4}\n", last.1)?;
    }
    print!("{}", col);
    if show_percentage {
        print!("{}", freq);
    }

    Ok(())
}

/// Prints the Rgb colors and percentage of their appearance in an image buffer.
pub fn print_colors_rgb(
    show_percentage: bool,
    colors: &Vec<(Srgb, f32, u8)>,
) -> Result<(), Box<dyn Error>> {
    let mut col = String::new();
    let mut freq = String::new();
    if let Some((last, elements)) = colors.split_last() {
        for elem in elements {
            write!(&mut col, "{:x},", Srgb::from(elem.0).into_format::<u8>())?;
            write!(&mut freq, "{:0.4},", elem.1)?;
        }
        write!(&mut col, "{:x}\n", Srgb::from(last.0).into_format::<u8>())?;
        write!(&mut freq, "{:0.4}\n", last.1)?;
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
        let mut enc = png::Encoder::new(w, imgx, imgy);
        enc.set_color(png::ColorType::RGB);
        enc.set_compression(png::Compression::Best);
        enc.set_filter(png::FilterType::NoFilter);

        // Clean up if file is created but there's a problem writing to it
        match enc.write_header()?.write_image_data(imgbuf) {
            Ok(_) => {}
            Err(err) => {
                eprintln!("Error: {}.", err);
                std::fs::remove_file(title)?;
            }
        }
    } else {
        let mut encoder = image::jpeg::JPEGEncoder::new_with_quality(&mut w, 90);

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

/// Save palette image file from Lab colors.
pub fn save_palette_lab(
    res: &[(Lab, f32, u8)],
    proportional: bool,
    height: u32,
    width: Option<u32>,
    title: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let len = res.len() as u32;
    let mut imgbuf: image::RgbImage;

    match width {
        Some(mut w) => {
            // Width must be at least `k` pixels wide
            if w < len {
                w = len;
            }

            imgbuf = image::ImageBuffer::new(w, height);

            if !proportional {
                for (x, _, pixel) in imgbuf.enumerate_pixels_mut() {
                    let color = Srgb::from(
                        res.get(
                            (((x as f32 / w as f32) * len as f32 - 0.5)
                                .max(0.0)
                                .min(len as f32))
                            .round() as usize,
                        )
                        .unwrap()
                        .0,
                    )
                    .into_format()
                    .into_raw();
                    *pixel = image::Rgb(color);
                }
            } else {
                let mut curr_pos = 0;
                if let Some((last, elements)) = res.split_last() {
                    for r in elements.iter() {
                        let pix: [u8; 3] = Srgb::from(r.0).into_format().into_raw();
                        // Clamp boundary to image width
                        let boundary = ((curr_pos as f32 + (r.1 * w as f32)).round() as u32).min(w);
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
                    let pix: [u8; 3] = Srgb::from(last.0).into_format().into_raw();
                    for y in 0..height {
                        for x in curr_pos..w {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            }

            Ok(save_image(&imgbuf.to_vec(), w, height, title)?)
        }
        None => {
            let w = height * len;
            imgbuf = image::ImageBuffer::new(w, height);
            if !proportional {
                for (i, r) in res.iter().enumerate() {
                    let pix: [u8; 3] = Srgb::from(r.0).into_format().into_raw();
                    for y in 0..height {
                        for x in (i as u32 * height)..((i as u32 + 1) * height) {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            } else {
                let mut curr_pos = 0;
                if let Some((last, elements)) = res.split_last() {
                    for r in elements.iter() {
                        let pix: [u8; 3] = Srgb::from(r.0).into_format().into_raw();
                        let boundary = ((curr_pos as f32 + (r.1 * w as f32)).round() as u32).min(w);
                        for y in 0..height {
                            for x in curr_pos..boundary {
                                imgbuf.put_pixel(x, y, image::Rgb(pix));
                            }
                        }
                        if boundary == w {
                            return Ok(save_image(&imgbuf.to_vec(), w, height, title)?);
                        }
                        curr_pos = boundary;
                    }
                    let pix: [u8; 3] = Srgb::from(last.0).into_format().into_raw();
                    for y in 0..height {
                        for x in curr_pos..w {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            }

            Ok(save_image(&imgbuf.to_vec(), w, height, title)?)
        }
    }
}

/// Save palette image file from RGB colors.
pub fn save_palette_rgb(
    res: &[(Srgb, f32, u8)],
    proportional: bool,
    height: u32,
    width: Option<u32>,
    title: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let len = res.len() as u32;
    let mut imgbuf: image::RgbImage;

    match width {
        Some(mut w) => {
            // Width must be at least `k` pixels wide
            if w < len {
                w = len;
            }

            imgbuf = image::ImageBuffer::new(w, height);

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
                        .0
                        .into_format()
                        .into_raw();
                    *pixel = image::Rgb(color);
                }
            } else {
                let mut curr_pos = 0;
                if let Some((last, elements)) = res.split_last() {
                    for r in elements.iter() {
                        let pix: [u8; 3] = (r.0).into_format().into_raw();
                        let boundary = ((curr_pos as f32 + (r.1 * w as f32)).round() as u32).min(w);
                        for y in 0..height {
                            for x in curr_pos..boundary {
                                imgbuf.put_pixel(x, y, image::Rgb(pix));
                            }
                        }
                        if boundary == w {
                            return Ok(save_image(&imgbuf.to_vec(), w, height, title)?);
                        }
                        curr_pos = boundary;
                    }
                    let pix: [u8; 3] = (last.0).into_format().into_raw();
                    for y in 0..height {
                        for x in curr_pos..w {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            }

            Ok(save_image(&imgbuf.to_vec(), w, height, title)?)
        }
        None => {
            let w = height * len;
            imgbuf = image::ImageBuffer::new(w, height);
            if !proportional {
                for (i, r) in res.iter().enumerate() {
                    let pix: [u8; 3] = (r.0).into_format().into_raw();
                    for y in 0..height {
                        for x in (i as u32 * height)..((i as u32 + 1) * height) {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            } else {
                let mut curr_pos = 0;
                if let Some((last, elements)) = res.split_last() {
                    for r in elements.iter() {
                        let pix: [u8; 3] = (r.0).into_format().into_raw();
                        let boundary = ((curr_pos as f32 + (r.1 * w as f32)).round() as u32).min(w);
                        for y in 0..height {
                            for x in curr_pos..boundary {
                                imgbuf.put_pixel(x, y, image::Rgb(pix));
                            }
                        }
                        if boundary == w {
                            return Ok(save_image(&imgbuf.to_vec(), w, height, title)?);
                        }
                        curr_pos = boundary;
                    }
                    let pix: [u8; 3] = (last.0).into_format().into_raw();
                    for y in 0..height {
                        for x in curr_pos..w {
                            imgbuf.put_pixel(x, y, image::Rgb(pix));
                        }
                    }
                }
            }

            Ok(save_image(&imgbuf.to_vec(), w, height, title)?)
        }
    }
}
