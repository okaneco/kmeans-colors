use std::error::Error;
use std::path::PathBuf;

use palette::{Lab, Pixel, Srgb, Srgba};

use crate::args::Opt;
use crate::filename::{create_filename, create_filename_palette};
use crate::utils::{parse_color, print_colors, save_image, save_image_alpha, save_palette};
use kmeans_colors::{get_kmeans, Calculate, Kmeans, MapColor, Sort};

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
        let img = image::open(&file)?.to_rgba();
        let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
        let img_vec = img.into_raw();
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
            let lab: Vec<Lab>;
            if !opt.transparent {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();
            } else {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format().into())
                    .collect();
            }

            // Iterate over amount of runs keeping best results
            let mut result = Kmeans::new();
            (0..opt.runs).for_each(|i| {
                let run_result = get_kmeans(
                    opt.k as usize,
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
                let mut res =
                    Lab::sort_indexed_colors::<Lab, _>(&result.centroids, &result.indices);
                if opt.sort {
                    res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
                }

                if opt.print || opt.percentage {
                    print_colors(opt.percentage, &res)?;
                }

                if opt.palette {
                    save_palette(
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
            if !opt.transparent {
                // Convert centroids to Srgb<u8> before mapping to buffer
                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| Srgb::from(*x).into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let lab: Vec<Srgb<u8>> = Srgb::map_indices_to_centroids(centroids, &result.indices);

                save_image(
                    Srgb::into_raw_slice(&lab),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            } else {
                // For transparent images, we get_closest_centroid based
                // on the centroids we calculated and only paint in the pixels
                // that have a full alpha
                let mut indices: Vec<u8> = Vec::with_capacity(img_vec.len());
                let lab: Vec<Lab> = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();
                Lab::get_closest_centroid(&lab, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| Srgba::from(*x).into_format())
                    .collect::<Vec<Srgba<u8>>>();

                let data = Srgba::from_raw_slice(&img_vec);
                let lab: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(&centroids, &indices)
                    .iter()
                    .zip(data)
                    .map(|(x, orig)| {
                        if orig.alpha == 255 {
                            *x
                        } else {
                            Srgba::new(0u8, 0, 0, 0)
                        }
                    })
                    .collect();
                save_image_alpha(
                    Srgba::into_raw_slice(&lab),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            }
        } else {
            // Read image buffer into Srgb format
            let rgb: Vec<Srgb>;
            if !opt.transparent {
                rgb = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();
            } else {
                rgb = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format().into())
                    .collect();
            }

            let mut result = Kmeans::new();

            // Iterate over amount of runs keeping best results
            (0..opt.runs).for_each(|i| {
                let run_result = get_kmeans(
                    opt.k as usize,
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
                let mut res =
                    Srgb::sort_indexed_colors::<Srgb, _>(&result.centroids, &result.indices);
                if opt.sort {
                    res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
                }

                if opt.print || opt.percentage {
                    print_colors(opt.percentage, &res)?;
                }

                if opt.palette {
                    save_palette(
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
            if !opt.transparent {
                // Pre-convert centroids into output format
                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| x.into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let rgb: Vec<Srgb<u8>> = Srgb::map_indices_to_centroids(centroids, &result.indices);

                save_image(
                    Srgb::into_raw_slice(&rgb),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            } else {
                // For transparent images, we get_closest_centroid based
                // on the centroids we calculated and only paint in the pixels
                // that have a full alpha
                let mut indices: Vec<u8> = Vec::with_capacity(img_vec.len());
                let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect();
                Srgb::get_closest_centroid(&rgb, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect::<Vec<Srgba<u8>>>();

                let data = Srgba::from_raw_slice(&img_vec);
                let rgb: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(&centroids, &indices)
                    .iter()
                    .zip(data)
                    .map(|(x, orig)| {
                        if orig.alpha == 255 {
                            *x
                        } else {
                            Srgba::new(0u8, 0, 0, 0)
                        }
                    })
                    .collect();
                save_image_alpha(
                    Srgba::into_raw_slice(&rgb),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            }
        }
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
                Lab::get_closest_centroid(&lab, &centroids, &mut indices);

                if percentage {
                    let res = Lab::sort_indexed_colors::<Lab, _>(&centroids, &indices);
                    print_colors(percentage, &res)?;
                }

                let rgb_centroids = &centroids
                    .iter()
                    .map(|x| Srgb::from(*x).into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let lab: Vec<Srgb<u8>> = Srgb::map_indices_to_centroids(&rgb_centroids, &indices);

                save_image(
                    Srgb::into_raw_slice(&lab),
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

                let mut result = Kmeans::new();
                let k = centroids.len() as u8;
                (0..runs).for_each(|i| {
                    let run_result = get_kmeans(
                        k as usize,
                        max_iter,
                        converge,
                        verbose,
                        &lab,
                        seed + i as u64,
                    );
                    if run_result.score < result.score {
                        result = run_result;
                    }
                });

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res =
                    Lab::sort_indexed_colors::<Lab, _>(&result.centroids, &result.indices);
                res.iter_mut()
                    .zip(&centroids)
                    .for_each(|(s, c)| s.centroid = *c);

                if percentage {
                    print_colors(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.index).cmp(&b.index));
                let sorted: Vec<Lab> = res.iter().map(|x| x.centroid).collect();

                let rgb_centroids = &sorted
                    .iter()
                    .map(|x| Srgb::from(*x).into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let lab: Vec<Srgb<u8>> =
                    Srgb::map_indices_to_centroids(&rgb_centroids, &result.indices);

                save_image(
                    Srgb::into_raw_slice(&lab),
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
                Srgb::get_closest_centroid(&rgb, &centroids, &mut indices);

                if percentage {
                    let res = Srgb::sort_indexed_colors::<Srgb, _>(&centroids, &indices);
                    print_colors(percentage, &res)?;
                }

                let rgb_centroids = &centroids
                    .iter()
                    .map(|x| x.into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let rgb: Vec<Srgb<u8>> = Srgb::map_indices_to_centroids(&rgb_centroids, &indices);

                save_image(
                    Srgb::into_raw_slice(&rgb),
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

                let mut result = Kmeans::new();
                let k = centroids.len();
                (0..runs).for_each(|i| {
                    let run_result =
                        get_kmeans(k, max_iter, converge, verbose, &rgb, seed + i as u64);
                    if run_result.score < result.score {
                        result = run_result;
                    }
                });

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res =
                    Srgb::sort_indexed_colors::<Srgb, _>(&result.centroids, &result.indices);
                res.iter_mut()
                    .zip(&centroids)
                    .for_each(|(s, c)| s.centroid = *c);

                if percentage {
                    print_colors(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.index).cmp(&b.index));
                let sorted: Vec<Srgb> = res.iter().map(|x| x.centroid).collect();

                let rgb_centroids = &sorted
                    .iter()
                    .map(|x| x.into_format())
                    .collect::<Vec<Srgb<u8>>>();
                let rgb: Vec<Srgb<u8>> =
                    Srgb::map_indices_to_centroids(&rgb_centroids, &result.indices);

                save_image(
                    Srgb::into_raw_slice(&rgb),
                    imgx,
                    imgy,
                    &create_filename(&input, &output, "png", None, file)?,
                )?;
            }
        }
    }

    Ok(())
}
