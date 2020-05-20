use std::error::Error;
use std::fmt::Write;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use palette::{Lab, Srgb};

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
