use fxhash::FxHashMap;
use palette::cast::{AsComponents, ComponentsAs};
use palette::{white_point::D65, FromColor, IntoColor, Lab, Srgb, Srgba};

use crate::args::Command;
use crate::err::CliError;
use crate::filename::create_filename;
use crate::utils::{cached_srgba_to_lab, parse_color, print_colors, save_image, save_image_alpha};
use kmeans_colors::{get_kmeans, get_kmeans_hamerly, Calculate, Kmeans, MapColor, Sort};

/// Find the image pixels which closest match the supplied colors and save that
/// image as output.
pub fn find_colors(
    Command::Find {
        input,
        colors,
        replace,
        max_iter,
        factor,
        runs,
        percentage,
        rgb,
        verbose,
        output,
        seed,
        transparent,
    }: Command,
) -> Result<(), Box<dyn std::error::Error>> {
    // Print filename if multiple files and percentage is set
    let display_filename = (input.len() > 1) && (percentage);
    let converge = factor.unwrap_or(if !rgb { 5.0 } else { 0.0025 });

    let seed = seed.unwrap_or(0);

    // Cached results of Srgb<u8> -> Lab conversions; not cleared between runs
    let mut lab_cache = FxHashMap::default();
    // Vec of pixels converted to Lab; cleared and reused between runs
    let mut lab_pixels: Vec<Lab<D65, f32>> = Vec::new();
    // Vec of pixels converted to Srgb<f32>; cleared and reused between runs
    let mut rgb_pixels: Vec<Srgb<f32>> = Vec::new();

    // Default to Lab colors
    if !rgb {
        // Initialize user centroids
        let centroids: Vec<Lab<D65, f32>> = colors
            .iter()
            .map(|c| {
                parse_color(c.trim_start_matches('#')).map(|c| c.into_linear::<f32>().into_color())
            })
            .collect::<Result<_, CliError>>()?;

        for file in &input {
            if display_filename {
                println!("{}", &file.to_string_lossy());
            }

            let img = image::open(file)?.into_rgba8();
            let (imgx, imgy) = img.dimensions();
            let img_vec: &[Srgba<u8>] = img.as_raw().components_as();

            lab_pixels.clear();

            if !transparent {
                cached_srgba_to_lab(img_vec.iter(), &mut lab_cache, &mut lab_pixels);
            } else {
                cached_srgba_to_lab(
                    img_vec.iter().filter(|x: &&Srgba<u8>| x.alpha == 255),
                    &mut lab_cache,
                    &mut lab_pixels,
                );
            }

            if !replace {
                let mut indices = Vec::with_capacity(img_vec.len());

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                Lab::<D65, f32>::get_closest_centroid(&lab_pixels, &centroids, &mut indices);

                if percentage {
                    let res = Lab::<D65, f32>::sort_indexed_colors(&centroids, &indices);
                    print_colors(percentage, &res)?;
                }

                if !transparent {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|&x| Srgb::from_linear(x.into_color()))
                        .collect::<Vec<Srgb<u8>>>();
                    let lab: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &indices);

                    save_image(
                        lab.as_components(),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                        false,
                    )?;
                } else {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|&x| Srgb::from_linear(x.into_color()))
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    rgb_pixels.clear();
                    rgb_pixels.extend(
                        img_vec
                            .iter()
                            .map(|x| Srgb::from_color(x.into_format::<_, f32>())),
                    );
                    Srgb::get_closest_centroid(&rgb_pixels, rgb_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let rgba: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
                        .iter()
                        .zip(img_vec)
                        .map(|(x, orig)| {
                            if orig.alpha == 255 {
                                *x
                            } else {
                                Srgba::new(0u8, 0, 0, 0)
                            }
                        })
                        .collect();

                    save_image_alpha(
                        rgba.as_components(),
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
                            &lab_pixels,
                            seed + i as u64,
                        );
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                } else {
                    for i in 0..runs {
                        let run_result = get_kmeans(
                            k,
                            max_iter,
                            converge,
                            verbose,
                            &lab_pixels,
                            seed + i as u64,
                        );
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
                let mut res =
                    Lab::<D65, f32>::sort_indexed_colors(&result.centroids, &result.indices);
                res.iter_mut()
                    .zip(&centroids)
                    .for_each(|(s, c)| s.centroid = *c);

                if percentage {
                    print_colors(percentage, &res)?;
                }

                // Sorting the centroids now
                res.sort_unstable_by(|a, b| (a.index).cmp(&b.index));
                let sorted: Vec<Lab<D65, f32>> = res.iter().map(|x| x.centroid).collect();

                if !transparent {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|&x| Srgb::from_linear(x.into_color()))
                        .collect::<Vec<Srgb<u8>>>();
                    let rgb: Vec<Srgb<u8>> =
                        Srgb::map_indices_to_centroids(rgb_centroids, &result.indices);
                    save_image(
                        rgb.as_components(),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                        false,
                    )?;
                } else {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|&x| Srgb::from_linear(x.into_color()))
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    rgb_pixels.clear();
                    rgb_pixels.extend(
                        img_vec
                            .iter()
                            .map(|x| Srgb::from_color(x.into_format::<_, f32>())),
                    );
                    let temp_centroids = cloned_res
                        .iter()
                        .map(|&x| Srgb::from_linear(x.into_color()))
                        .collect::<Vec<Srgb>>();
                    Srgb::get_closest_centroid(&rgb_pixels, &temp_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let rgba: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
                        .iter()
                        .zip(img_vec)
                        .map(|(x, orig)| {
                            if orig.alpha == 255 {
                                *x
                            } else {
                                Srgba::new(0u8, 0, 0, 0)
                            }
                        })
                        .collect();

                    save_image_alpha(
                        rgba.as_components(),
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
            let img = image::open(file)?.into_rgba8();
            let (imgx, imgy) = img.dimensions();
            let img_vec: &[Srgba<u8>] = img.as_raw().components_as();

            rgb_pixels.clear();

            if !transparent {
                rgb_pixels.extend(
                    img_vec
                        .iter()
                        .map(|x| Srgb::from_color(x.into_format::<_, f32>())),
                );
            } else {
                rgb_pixels.extend(
                    img_vec
                        .iter()
                        .filter(|x| x.alpha == 255)
                        .map(|x| Srgb::from_color(x.into_format::<_, f32>())),
                );
            }

            if !replace {
                let mut indices = Vec::with_capacity(img_vec.len());

                // We only need to do one pass of getting the closest colors to the
                // custom centroids
                Srgb::get_closest_centroid(&rgb_pixels, &centroids, &mut indices);

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
                        rgb.as_components(),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                        false,
                    )?;
                } else {
                    let rgb_centroids = &centroids
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    rgb_pixels.clear();
                    rgb_pixels.extend(
                        img_vec
                            .iter()
                            .map(|&x| Srgb::from_color(x.into_format::<_, f32>())),
                    );
                    Srgb::get_closest_centroid(&rgb_pixels, rgb_centroids, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let rgb: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
                        .iter()
                        .zip(img_vec)
                        .map(|(x, orig)| {
                            if orig.alpha == 255 {
                                *x
                            } else {
                                Srgba::new(0u8, 0, 0, 0)
                            }
                        })
                        .collect();

                    save_image_alpha(
                        rgb.as_components(),
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
                            &rgb_pixels,
                            seed + i as u64,
                        );
                        if run_result.score < result.score {
                            result = run_result;
                        }
                    }
                } else {
                    for i in 0..runs {
                        let run_result = get_kmeans(
                            k,
                            max_iter,
                            converge,
                            verbose,
                            &rgb_pixels,
                            seed + i as u64,
                        );
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
                        rgb.as_components(),
                        imgx,
                        imgy,
                        &create_filename(&input, &output, "png", None, file)?,
                        false,
                    )?;
                } else {
                    let rgb_centroids = &sorted
                        .iter()
                        .map(|x| x.into_format())
                        .collect::<Vec<Srgb>>();

                    let mut indices = Vec::with_capacity(img_vec.len());
                    rgb_pixels.clear();
                    rgb_pixels.extend(
                        img_vec
                            .iter()
                            .map(|x| Srgb::from_color(x.into_format::<_, f32>())),
                    );
                    Srgb::get_closest_centroid(&rgb_pixels, &cloned_res, &mut indices);

                    let centroids = &rgb_centroids
                        .iter()
                        .map(|x| Srgba::from(*x).into_format())
                        .collect::<Vec<Srgba<u8>>>();

                    let rgba: Vec<Srgba<u8>> = Srgba::map_indices_to_centroids(centroids, &indices)
                        .iter()
                        .zip(img_vec)
                        .map(|(x, orig)| {
                            if orig.alpha == 255 {
                                *x
                            } else {
                                Srgba::new(0u8, 0, 0, 0)
                            }
                        })
                        .collect();

                    save_image_alpha(
                        rgba.as_components(),
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
