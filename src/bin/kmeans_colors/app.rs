use std::error::Error;
use std::path::PathBuf;

use palette::{FromColor, IntoColor, Lab, Pixel, Srgb, Srgba};

use crate::args::Opt;
use crate::filename::{create_filename, create_filename_palette};
use crate::utils::{parse_color, print_colors, save_image, save_image_alpha, save_palette};
use kmeans_colors::{get_kmeans, get_kmeans_hamerly, Calculate, Kmeans, MapColor, Sort};

pub fn run(opt: Opt) -> Result<(), Box<dyn Error>> {
    if opt.input.is_empty() {
        eprintln!("No input files specified.")
    }

    let seed = opt.seed.unwrap_or(0);

    for file in &opt.input {
        if opt.verbose {
            println!("{}", &file.to_string_lossy());
        }
        let img = image::open(&file)?.into_rgba8();
        let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
        let img_vec = img.into_raw();
        let converge;
        match opt.factor {
            Some(x) => converge = x,
            None => {
                converge = if !opt.rgb { 5.0 } else { 0.0025 };
            }
        }

        // Defaults to Lab, first case.
        if !opt.rgb {
            // Convert Srgb image buffer to Lab for kmeans
            let lab: Vec<Lab>;
            if !opt.transparent {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            } else {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            }

            // Iterate over amount of runs keeping best results
            let mut result = Kmeans::new();
            if opt.k > 1 {
                for i in 0..opt.runs {
                    let run_result = get_kmeans_hamerly(
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
                }
            } else {
                for i in 0..opt.runs {
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
                }
            }

            // Print and/or sort results, output to palette
            if opt.print || opt.percentage || opt.palette {
                let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
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
                    .map(|x| Srgb::from_color(*x).into_format())
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
                let mut indices = Vec::with_capacity(img_vec.len());
                let lab: Vec<Lab> = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
                Lab::get_closest_centroid(&lab, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| Srgba::from_color(*x).into_format())
                    .collect::<Vec<Srgba<u8>>>();

                let data = Srgba::from_raw_slice(&img_vec);
                let lab: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            } else {
                rgb = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            }

            // Iterate over amount of runs keeping best results
            let mut result = Kmeans::new();
            if opt.k > 1 {
                for i in 0..opt.runs {
                    let run_result = get_kmeans_hamerly(
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
                }
            } else {
                for i in 0..opt.runs {
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
                }
            }

            // Print and/or sort results, output to palette
            if opt.print || opt.percentage || opt.palette {
                let mut res = Srgb::sort_indexed_colors(&result.centroids, &result.indices);
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
                let mut indices = Vec::with_capacity(img_vec.len());
                let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
                Srgb::get_closest_centroid(&rgb, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| x.into_format().into())
                    .collect::<Vec<Srgba<u8>>>();

                let data = Srgba::from_raw_slice(&img_vec);
                let rgb: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
    transparent: bool,
) -> Result<(), Box<dyn Error>> {
    // Print filename if multiple files and percentage is set
    let display_filename = (input.len() > 1) && (percentage);
    let converge;

    match factor {
        Some(x) => converge = x,
        None => {
            converge = if !rgb { 5.0 } else { 0.0025 };
        }
    }

    let seed = seed.unwrap_or(0);

    // Default to Lab colors
    if !rgb {
        // Initialize user centroids
        let mut centroids: Vec<Lab> = Vec::with_capacity(colors.len());
        for c in colors {
            centroids.push(
                (parse_color(c.trim_start_matches('#'))?)
                    .into_format()
                    .into_color(),
            );
        }

        for file in &input {
            if display_filename {
                println!("{}", &file.to_string_lossy());
            }

            let img = image::open(&file)?.into_rgba8();
            let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
            let img_vec = img.into_raw();

            let lab: Vec<Lab>;
            if !transparent {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            } else {
                lab = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            }

            if !replace {
                let mut indices = Vec::with_capacity(img_vec.len());

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                Lab::get_closest_centroid(&lab, &centroids, &mut indices);

                if percentage {
                    let res = Lab::sort_indexed_colors(&centroids, &indices);
                    print_colors(percentage, &res)?;
                }

                if !transparent {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|x| Srgb::from_color(*x).into_format())
                        .collect::<Vec<Srgb<u8>>>();
                    let lab: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &indices);

                    save_image(
                        Srgb::into_raw_slice(&lab),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                } else {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|x| Srgb::from_color(*x).into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                        .iter()
                        .map(|x| x.into_format::<_, f32>().into_color())
                        .collect();
                    Srgb::get_closest_centroid(&rgb, rgb_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let data = Srgba::from_raw_slice(&img_vec);
                    let lab: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                }
            } else {
                // Replace the k-means colors case
                let mut result = Kmeans::new();
                let k = centroids.len();
                if k > 1 {
                    for i in 0..runs {
                        let run_result = get_kmeans_hamerly(
                            k,
                            max_iter,
                            converge,
                            verbose,
                            &lab,
                            seed + i as u64,
                        );
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                } else {
                    for i in 0..runs {
                        let run_result =
                            get_kmeans(k, max_iter, converge, verbose, &lab, seed + i as u64);
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                }

                // This is the easiest way to make this work for transparent without a larger restructuring
                let cloned_res = result.centroids.clone();

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
                res.iter_mut()
                    .zip(&centroids)
                    .for_each(|(s, c)| s.centroid = *c);

                if percentage {
                    print_colors(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.index).cmp(&b.index));
                let sorted: Vec<Lab> = res.iter().map(|x| x.centroid).collect();

                if !transparent {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|x| Srgb::from_color(*x).into_format())
                        .collect::<Vec<Srgb<u8>>>();
                    let lab: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &result.indices);
                    save_image(
                        Srgb::into_raw_slice(&lab),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                } else {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|x| Srgb::from_color(*x).into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                        .iter()
                        .map(|x| x.into_format::<_, f32>().into_color())
                        .collect();
                    let temp_centroids = cloned_res
                        .iter()
                        .map(|x| Srgb::from_color(*x))
                        .collect::<Vec<Srgb>>();
                    Srgb::get_closest_centroid(&rgb, &temp_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let data = Srgba::from_raw_slice(&img_vec);
                    let lab: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                }
            }
        }

    // Rgb case
    } else {
        // Initialize user centroids
        let mut centroids: Vec<Srgb> = Vec::with_capacity(colors.len());
        for c in colors {
            centroids.push((parse_color(c.trim_start_matches('#'))?).into_format());
        }

        for file in &input {
            if display_filename {
                println!("{}", &file.to_string_lossy());
            }
            let img = image::open(&file)?.into_rgba8();
            let (imgx, imgy) = (img.dimensions().0, img.dimensions().1);
            let img_vec = img.into_raw();

            let rgb: Vec<Srgb>;
            if !transparent {
                rgb = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            } else {
                rgb = Srgba::from_raw_slice(&img_vec)
                    .iter()
                    .filter(|x| x.alpha == 255)
                    .map(|x| x.into_format::<_, f32>().into_color())
                    .collect();
            }

            if !replace {
                let mut indices = Vec::with_capacity(img_vec.len());

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                Srgb::get_closest_centroid(&rgb, &centroids, &mut indices);

                if percentage {
                    let res = Srgb::sort_indexed_colors(&centroids, &indices);
                    print_colors(percentage, &res)?;
                }

                if !transparent {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb<u8>>>();
                    let rgb: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &indices);

                    save_image(
                        Srgb::into_raw_slice(&rgb),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                } else {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                        .iter()
                        .map(|x| x.into_format::<_, f32>().into_color())
                        .collect();
                    Srgb::get_closest_centroid(&rgb, rgb_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let data = Srgba::from_raw_slice(&img_vec);
                    let rgb: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                }
            } else {
                // Replace the k-means colors case
                let mut result = Kmeans::new();
                let k = centroids.len();
                if k > 1 {
                    for i in 0..runs {
                        let run_result = get_kmeans_hamerly(
                            k,
                            max_iter,
                            converge,
                            verbose,
                            &rgb,
                            seed + i as u64,
                        );
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                } else {
                    for i in 0..runs {
                        let run_result =
                            get_kmeans(k, max_iter, converge, verbose, &rgb, seed + i as u64);
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                }

                let cloned_res = result.centroids.clone();

                // We want to sort the user centroids based on the kmeans colors
                // sorted by luminosity using the u8 returned in `sorted`. This
                // corresponds to the index of the colors from darkest to lightest.
                // We replace the colors in `sorted` with our centroids for printing
                // purposes.
                let mut res = Srgb::sort_indexed_colors(&result.centroids, &result.indices);
                res.iter_mut()
                    .zip(&centroids)
                    .for_each(|(s, c)| s.centroid = *c);

                if percentage {
                    print_colors(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.index).cmp(&b.index));
                let sorted: Vec<Srgb> = res.iter().map(|x| x.centroid).collect();

                if !transparent {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb<u8>>>();
                    let rgb: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &result.indices);

                    save_image(
                        Srgb::into_raw_slice(&rgb),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                } else {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    let rgb: Vec<Srgb> = Srgba::from_raw_slice(&img_vec)
                        .iter()
                        .map(|x| x.into_format::<_, f32>().into_color())
                        .collect();
                    Srgb::get_closest_centroid(&rgb, &cloned_res, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let data = Srgba::from_raw_slice(&img_vec);
                    let lab: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
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
                        &create_filename(&input, &output, "png", None, file)?,
                    )?;
                }
            }
        }
    }

    Ok(())
}
