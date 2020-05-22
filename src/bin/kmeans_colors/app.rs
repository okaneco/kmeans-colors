use std::error::Error;
use std::path::PathBuf;

use palette::{Lab, Pixel, Srgb};

use crate::args::Opt;
use crate::filename::{create_filename, create_filename_palette};
use crate::utils::{
    parse_color, print_colors_lab, print_colors_rgb, save_image, save_palette_lab, save_palette_rgb,
};
use kmeans_colors::{
    get_closest_centroid_lab, get_closest_centroid_rgb, get_kmeans_lab, get_kmeans_rgb,
    map_indices_to_colors_lab, map_indices_to_colors_rgb, sort_indexed_colors_lab,
    sort_indexed_colors_rgb, KmeansLab, KmeansRgb,
};

pub fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    if opt.input.len() == 0 {
        eprintln!("No input files specified.")
    }

    let seed = match opt.seed {
        Some(s) => s,
        None => 0,
    };

    for file in &opt.input {
        if opt.verbose {
            println!("{}", &file.to_string_lossy());
        }
        let img = image::open(&file)?.to_rgb();
        let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
        let img_vec = img.into_raw();
        let buffer;
        let converge;
        match opt.factor {
            Some(x) => converge = x,
            None => {
                converge = if !opt.rgb { 10.0 } else { 0.0025 };
            }
        }

        // Defaults to Lab, first case.
        if !opt.rgb {
            // Convert Srgb image buffer to Lab for kmeans
            let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
                .iter()
                .map(|x| x.into_format().into())
                .collect();

            // Iterate over amount of runs keeping best results
            let mut result = KmeansLab::new();
            (0..opt.runs).for_each(|i| {
                let run_result = get_kmeans_lab(
                    opt.k,
                    opt.max_iter,
                    converge,
                    opt.verbose,
                    &lab,
                    seed + i as u64,
                );
                if run_result.score < result.score {
                    result = run_result;
                }
            });

            // Print and/or sort results, output to palette
            if opt.print || opt.percentage || opt.palette {
                let mut res = sort_indexed_colors_lab(&result.centroids, &result.indices);
                if opt.sort {
                    res.sort_unstable_by(|a, b| (b.1).partial_cmp(&a.1).unwrap());
                }

                if opt.print || opt.percentage {
                    print_colors_lab(opt.percentage, &res)?;
                }

                if opt.palette {
                    save_palette_lab(
                        &res,
                        opt.proportional,
                        opt.height,
                        opt.width,
                        &create_filename_palette(
                            &opt.input,
                            &opt.palette_output,
                            opt.rgb,
                            Some(opt.k),
                            file,
                        )?,
                    )?;
                }
            }

            // Don't allocate image buffer if no-file
            if opt.no_file {
                continue;
            }

            // Convert indexed colors to Srgb colors to output as final result
            buffer = map_indices_to_colors_lab(&result.centroids, &result.indices);
        } else {
            // Read image buffer into Srgb format
            let rgb: Vec<Srgb> = Srgb::from_raw_slice(&img_vec)
                .iter()
                .map(|x| x.into_format().into())
                .collect();

            let mut result = KmeansRgb::new();

            // Iterate over amount of runs keeping best results
            (0..opt.runs).for_each(|i| {
                let run_result = get_kmeans_rgb(
                    opt.k,
                    opt.max_iter,
                    converge,
                    opt.verbose,
                    &rgb,
                    seed + i as u64,
                );
                if run_result.score < result.score {
                    result = run_result;
                }
            });

            // Print and/or sort results, output to palette
            if opt.print || opt.percentage || opt.palette {
                let mut res = sort_indexed_colors_rgb(&result.centroids, &result.indices);
                if opt.sort {
                    res.sort_unstable_by(|a, b| (b.1).partial_cmp(&a.1).unwrap());
                }

                if opt.print || opt.percentage {
                    print_colors_rgb(opt.percentage, &res)?;
                }

                if opt.palette {
                    save_palette_rgb(
                        &res,
                        opt.proportional,
                        opt.height,
                        opt.width,
                        &create_filename_palette(
                            &opt.input,
                            &opt.palette_output,
                            opt.rgb,
                            Some(opt.k),
                            file,
                        )?,
                    )?;
                }
            }

            // Don't allocate image buffer if no-file
            if opt.no_file {
                continue;
            }

            // Convert indexed colors to Srgb colors to output as final result
            buffer = map_indices_to_colors_rgb(&result.centroids, &result.indices);
        }

        save_image(
            &buffer,
            imgx,
            imgy,
            &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
        )?;
    }

    Ok(())
}

/// Find the image pixels which closest match the supplied colors and save that
/// image as output.
pub fn find_colors(
    input: Vec<PathBuf>,
    colors: Vec<String>,
    replace: bool,
    max_iter: usize,
    factor: Option<f32>,
    runs: usize,
    percentage: bool,
    rgb: bool,
    verbose: bool,
    output: Option<PathBuf>,
    seed: Option<u64>,
) -> Result<(), Box<dyn Error>> {
    // Print filename if multiple files and percentage is set
    let display_filename = (input.len() > 1) && (percentage);
    let converge;

    match factor {
        Some(x) => converge = x,
        None => {
            converge = if !rgb { 8.0 } else { 0.0025 };
        }
    }

    let seed = match seed {
        Some(s) => s,
        None => 0,
    };

    // Default to Lab colors
    if !rgb {
        // Initialize user centroids
        let mut centroids: Vec<Lab> = Vec::with_capacity(colors.len());
        for c in colors {
            centroids.push(
                (parse_color(c.trim_start_matches("#"))?)
                    .into_format()
                    .into(),
            );
        }

        if !replace {
            for file in &input {
                if display_filename {
                    println!("{}", &file.to_string_lossy());
                }
                let img = image::open(&file)?.to_rgb();
                let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
                let img_vec = img.into_raw();
                let mut indices = Vec::with_capacity(img_vec.len());

                let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                get_closest_centroid_lab(&lab, &centroids, &mut indices);

                if percentage {
                    let res = sort_indexed_colors_lab(&centroids, &indices);
                    print_colors_lab(percentage, &res)?;
                }

                let buffer = map_indices_to_colors_lab(&centroids, &indices);
                save_image(
                    &buffer,
                    imgx,
                    imgy,
                    &create_filename(&input, &output, "png", None, file)?,
                )?;
            }
        } else {
            // Replace the k-means colors case
            for file in &input {
                if display_filename {
                    println!("{}", &file.to_string_lossy());
                }
                let img = image::open(&file)?.to_rgb();
                let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
                let img_vec = img.into_raw();

                let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();

                let mut result = KmeansLab::new();
                let k = centroids.len() as u8;
                (0..runs).for_each(|i| {
                    let run_result =
                        get_kmeans_lab(k, max_iter, converge, verbose, &lab, seed + i as u64);
                    if run_result.score < result.score {
                        result = run_result;
                    }
                });

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res = sort_indexed_colors_lab(&result.centroids, &result.indices);
                res.iter_mut().zip(&centroids).for_each(|(s, c)| s.0 = *c);

                if percentage {
                    print_colors_lab(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.2).cmp(&b.2));
                let sorted: Vec<Lab> = res.iter().map(|x| x.0).collect();

                let buffer = map_indices_to_colors_lab(&sorted, &result.indices);
                save_image(
                    &buffer,
                    imgx,
                    imgy,
                    &create_filename(&input, &output, "png", None, file)?,
                )?;
            }
        }

    // Rgb case
    } else {
        // Initialize user centroids
        let mut centroids: Vec<Srgb> = Vec::with_capacity(colors.len());
        for c in colors {
            centroids.push(
                (parse_color(c.trim_start_matches("#"))?)
                    .into_format()
                    .into(),
            );
        }

        if !replace {
            for file in &input {
                if display_filename {
                    println!("{}", &file.to_string_lossy());
                }
                let img = image::open(&file)?.to_rgb();
                let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
                let img_vec = img.into_raw();
                let mut indices = Vec::with_capacity(img_vec.len());

                let rgb: Vec<Srgb> = Srgb::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                get_closest_centroid_rgb(&rgb, &centroids, &mut indices);

                if percentage {
                    let res = sort_indexed_colors_rgb(&centroids, &indices);
                    print_colors_rgb(percentage, &res)?;
                }

                let buffer = map_indices_to_colors_rgb(&centroids, &indices);
                save_image(
                    &buffer,
                    imgx,
                    imgy,
                    &create_filename(&input, &output, "png", None, file)?,
                )?;
            }
        } else {
            // Replace the k-means colors case
            for file in &input {
                if display_filename {
                    println!("{}", &file.to_string_lossy());
                }
                let img = image::open(&file)?.to_rgb();
                let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
                let img_vec = img.into_raw();

                let rgb: Vec<Srgb> = Srgb::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();

                let mut result = KmeansRgb::new();
                let k = centroids.len() as u8;
                (0..runs).for_each(|i| {
                    let run_result =
                        get_kmeans_rgb(k, max_iter, converge, verbose, &rgb, seed + i as u64);
                    if run_result.score < result.score {
                        result = run_result;
                    }
                });

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res = sort_indexed_colors_rgb(&result.centroids, &result.indices);
                res.iter_mut().zip(&centroids).for_each(|(s, c)| s.0 = *c);

                if percentage {
                    print_colors_rgb(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.2).cmp(&b.2));
                let sorted: Vec<Srgb> = res.iter().map(|x| x.0).collect();

                let buffer = map_indices_to_colors_rgb(&sorted, &result.indices);
                save_image(
                    &buffer,
                    imgx,
                    imgy,
                    &create_filename(&input, &output, "png", None, file)?,
                )?;
            }
        }
    }

    Ok(())
}
