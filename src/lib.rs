//! Calculate the k-means of a set of data.
//!
//! # Overview
//!
//! This crate provides traits for calculating and implementing a k-means
//! clustering algorithm. The original implementation of this library was
//! created for finding k-means colors in image buffers. Applications of crate
//! functionality can be seen on the [README page][readme].
//!
//! [readme]: https://github.com/okaneco/kmeans-colors/blob/master/README.md
//!
//! When using the library, set `default-features = false` in the Cargo.toml to
//! avoid bringing in the binary dependencies. If working with colors,
//! implementations have been provided for the [`palette`][palette] `Lab` and
//! `Srgb` color types behind the `palette_color` feature.
//!
//! The binary located in `src/bin/kmeans_colors` shows examples of crate
//! functionality.
//!
//! [palette]: https://github.com/Ogeon/palette/
//!
//! ## The `Calculate` trait
//! k-means calculations can be provided for other data types by implementing
//! the [`Calculate`](trait.Calculate.html) trait. See the `Lab` and `Srgb`
//! implementations in [`kmeans.rs`](../src/kmeans_colors/kmeans.rs.html#120)
//! for examples.
//!
//! ## Calculating k-means with `palette_color`
//!
//! The `palette_color` feature provides implementations of the `Calculate`
//! trait for the `Lab` color space and `Srgb` color space. Each space has
//! advantages and drawbacks due to the characteristics of the color space.
//!
//! The `Lab` calculation produces more perceptually accurate results at a
//! slightly slower runtime. `Srgb` calculation will converge faster than `Lab`
//! but the results may not visually correlate as well to the original image.
//! Overall, properly converged results should not differ that drastically
//! except at lower `k` counts. At `k=1`, the average color of an image,
//! results should match almost exactly.
//!
//! Note: If k-means calculation is taking too long, try scaling down the
//! image size. A full-size image is not required for calculating the color
//! palette or dominant color.
//!
//! ### Calculating k-means
//!
//! A basic workflow consists of reading a pixel buffer in, converting it into a
//! flat array, then using that array with the k-means functions. The following
//! example converts an array of `u8` into `Lab` colors then finds the k-means.
//!
//! ```
//! use palette::{Lab, Pixel, Srgb};
//! use kmeans_colors::{get_kmeans, Calculate, Kmeans, Sort};
//!
//! // An image buffer of one black pixel and one white pixel
//! let img_vec = [0u8, 0, 0, 255, 255, 255];
//!
//! # let runs = 3;
//! # let k = 1;
//! # let max_iter = 20;
//! # let converge = 8.0;
//! # let verbose = false;
//! # let seed = 0;
//! // Convert RGB [u8] buffer to Lab for k-means
//! let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
//!     .iter()
//!     .map(|x| x.into_format().into())
//!     .collect();
//!
//! // Iterate over amount of runs keeping best results
//! let mut result = Kmeans::new();
//! (0..runs).for_each(|i| {
//!     let run_result = get_kmeans(
//!         k,
//!         max_iter,
//!         converge,
//!         verbose,
//!         &lab,
//!         seed + i as u64,
//!     );
//!     if run_result.score < result.score {
//!         result = run_result;
//!     }
//! });
//!
//! // Convert indexed colors back to RGB [u8] for output
//! let buffer = Lab::map_indices_to_centroids(&result.centroids, &result.indices);
//! # assert_eq!(buffer, [119, 119, 119, 119, 119, 119]);
//! ```
//!
//! Because the initial seeds are random, the k-means calculation should be run
//! multiple times in order to assure that the best result has been found. The
//! algorithm may find itself in local minima that is not the optimal result.
//! This is especially true for `Lab` but `Srgb` may only need one run.
//!
//! The binary uses `8` as the default `k`. The iteration limit is set to `20`,
//! RGB usually converges in under 10 iterations depending on the `k`. The
//! convergence factor defaults to `10.0` for `Lab` and `0.0025` for `Srgb`. The
//! number of runs defaults to `3` for one of the binary subcommands. Through
//! testing, these numbers were found to be an adequate tradeoff between
//! performance and accuracy. If the results do not appear correct, raise the
//! iteration limit as convergence was probably not met.
//!
//! ### Getting the dominant color
//!
//! After k-means calculation, the dominant color can be found by sorting the
//! results and taking the centroid of the first item. The
//! [`sort_indexed_colors`][sort] function sorts the colors from darkest to
//! lightest and returns an array of [`CentroidData`](struct.CentroidData.html).
//!
//! [sort]: trait.Sort.html#tymethod.sort_indexed_colors
//! ```no_run
//! # use palette::{Lab, Pixel, Srgb};
//! # use kmeans_colors::{get_kmeans, Kmeans};
//! use kmeans_colors::Sort;
//!
//! # let img_vec = [0u8, 0, 0, 255, 255, 255];
//! # let runs = 3;
//! # let k = 1;
//! # let max_iter = 20;
//! # let converge = 8.0;
//! # let verbose = false;
//! # let seed = 0;
//! # let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
//! #    .iter()
//! #    .map(|x| x.into_format().into())
//! #    .collect();
//! # let mut result = Kmeans::new();
//! # (0..runs).for_each(|i| {
//! #     let run_result = get_kmeans(
//! #         k,
//! #         max_iter,
//! #         converge,
//! #         verbose,
//! #         &lab,
//! #         seed + i as u64,
//! #     );
//! #     if run_result.score < result.score {
//! #         result = run_result;
//! #     }
//! # });
//! // Using the results from the previous example, process the centroid data
//! let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
//!
//! // We can find the dominant color directly
//! let dominant_color = Lab::get_dominant_color(&res);
//! # assert_eq!(
//! #    Srgb::from(dominant_color.unwrap()).into_format::<u8>(),
//! #    Srgb::new(119u8, 119, 119)
//! # );
//!
//! // Or we can manually sort the vec by percentage, and the most appearing
//! // color will be the first element
//! res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
//! let dominant_color = res.first().unwrap().centroid;
//! ```

mod kmeans;
mod sort;

pub use kmeans::{get_kmeans, Calculate, Kmeans};
pub use sort::{CentroidData, Sort};
