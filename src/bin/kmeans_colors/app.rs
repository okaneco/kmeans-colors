use crate::args::Opt;
use crate::filename::{create_filename, create_filename_palette};
use crate::utils::{cached_srgba_to_lab, print_colors, save_image, save_image_alpha, save_palette};

use fxhash::FxHashMap;
use kmeans_colors::{get_kmeans, get_kmeans_hamerly, Calculate, Kmeans, MapColor, Sort};
use palette::cast::{AsComponents, ComponentsAs};
use palette::{white_point::D65, FromColor, IntoColor, Lab, LinSrgba, Srgb, Srgba};

pub fn run(opt: Opt) -> Result<(), Box<dyn std::error::Error>> {
    if opt.input.is_empty() {
        eprintln!("No input files specified.")
    }

    let seed = opt.seed.unwrap_or(0);

    // Cached results of Srgb<u8> -> Lab conversions; not cleared between runs
    let mut lab_cache = FxHashMap::default();
    // Vec of pixels converted to Lab; cleared and reused between runs
    let mut lab_pixels: Vec<Lab<D65, f32>> = Vec::new();
    // Vec of pixels converted to Srgb<f32>; cleared and reused between runs
    let mut rgb_pixels: Vec<Srgb<f32>> = Vec::new();

    for file in &opt.input {
        if opt.verbose {
            println!("{}", &file.to_string_lossy());
        }
        let img = image::open(file)?.into_rgba8();
        let (imgx, imgy) = img.dimensions();
        let img_vec: &[Srgba<u8>] = img.as_raw().components_as();
        let converge = opt.factor.unwrap_or(if !opt.rgb { 5.0 } else { 0.0025 });

        // Defaults to Lab, first case.
        if !opt.rgb {
            lab_pixels.clear();

            // Convert Srgb image buffer to Lab for kmeans
            if !opt.transparent {
                cached_srgba_to_lab(img_vec.iter(), &mut lab_cache, &mut lab_pixels);
            } else {
                cached_srgba_to_lab(
                    img_vec.iter().filter(|x: &&Srgba<u8>| x.alpha == 255),
                    &mut lab_cache,
                    &mut lab_pixels,
                );
            };

            // Iterate over amount of runs keeping best results
            let mut result = Kmeans::new();
            if opt.k > 1 {
                for i in 0..opt.runs {
                    let run_result = get_kmeans_hamerly(
                        opt.k as usize,
                        opt.max_iter,
                        converge,
                        opt.verbose,
                        &lab_pixels,
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
                        &lab_pixels,
                        seed + i as u64,
                    );
                    if run_result.score < result.score {
                        result = run_result;
                    }
                }
            }

            // Print and/or sort results, output to palette
            if opt.print || opt.percentage || opt.palette {
                let mut res =
                    Lab::<D65, f32>::sort_indexed_colors(&result.centroids, &result.indices);
                if opt.sort {
                    res.sort_unstable_by(|a, b| (b.percentage).total_cmp(&a.percentage));
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
                    .map(|&x| Srgb::from_linear(x.into_color()))
                    .collect::<Vec<Srgb<u8>>>();
                let rgb: Vec<Srgb<u8>> = Srgb::map_indices_to_centroids(centroids, &result.indices);

                save_image(
                    rgb.as_components(),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                    false,
                )?;
            } else {
                // For transparent images, we get_closest_centroid based
                // on the centroids we calculated and only paint in the pixels
                // that have a full alpha
                let mut indices = Vec::with_capacity(img_vec.len());

                lab_pixels.clear();
                cached_srgba_to_lab(img_vec.iter(), &mut lab_cache, &mut lab_pixels);
                Lab::<D65, f32>::get_closest_centroid(&lab_pixels, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|&x| Srgba::<f32>::from_linear(LinSrgba::from_color(x)).into_format())
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
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            }
        } else {
            rgb_pixels.clear();

            // Read image buffer into Srgb format
            if !opt.transparent {
                rgb_pixels.extend(
                    img_vec
                        .iter()
                        .map(|x| Srgb::<f32>::from_color(x.into_format::<_, f32>())),
                );
            } else {
                rgb_pixels.extend(
                    img_vec
                        .iter()
                        .filter(|x| x.alpha == 255)
                        .map(|x| Srgb::<f32>::from_color(x.into_format::<_, f32>())),
                );
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
                        &rgb_pixels,
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
                        &rgb_pixels,
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
                    res.sort_unstable_by(|a, b| (b.percentage).total_cmp(&a.percentage));
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
                    rgb.as_components(),
                    imgx,
                    imgy,
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                    false,
                )?;
            } else {
                // For transparent images, we get_closest_centroid based
                // on the centroids we calculated and only paint in the pixels
                // that have a full alpha
                let mut indices = Vec::with_capacity(img_vec.len());

                rgb_pixels.clear();
                rgb_pixels.extend(
                    img_vec
                        .iter()
                        .map(|x| Srgb::<f32>::from_color(x.into_format::<_, f32>())),
                );
                Srgb::get_closest_centroid(&rgb_pixels, &result.centroids, &mut indices);

                let centroids = &result
                    .centroids
                    .iter()
                    .map(|x| x.into_format().into())
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
                    &create_filename(&opt.input, &opt.output, &opt.extension, Some(opt.k), file)?,
                )?;
            }
        }
    }

    Ok(())
}
