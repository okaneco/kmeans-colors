//! Calculate the k-means colors of an image.
//!
//! The library should be used with `default-features = false`. Image examples
//! and demonstrations of parameters can be seen on the [README page][readme].
//!
//! [readme]: https://github.com/okaneco/kmeans-colors/blob/master/README.md
//!
//! # Overview
//!
//! This crate provides methods for k-means calculation using the Lab color
//! space or RGB color space. Each space has advantages and disadvantages due
//! to the characteristics of the color space.
//!
//! The Lab calculation produces more perceptually accurate results at a
//! slightly slower runtime. RGB calculation will converge faster than Lab but
//! the results may not visually correlate as well to the original image.
//! Overall, properly converged results should not differ that drastically
//! except at lower `k` counts. At `k=1`, the average color of an image,
//! results should match almost exactly.
//!
//! The binary located in `src/bin/kmeans_colors` shows some examples of crate
//! functionality. The crate uses [`palette`][palette] for its `Lab` and `Srgb`
//! color types.
//!
//! Note: If k-means calculation is taking too long, try scaling down the
//! image size. A full-size image is not required for calculating the color
//! palette or dominant color.
//!
//! [palette]: https://github.com/Ogeon/palette/
//!
//! ## Calculating k-means
//!
//! A basic workflow consists of reading a pixel buffer in, converting it into a
//! flat array, then using that array with the k-means functions. The following
//! example converts an array of `u8` into Lab colors then finds the k-means.
//! To find the k-means in RGB, convert the colors into Srgb then call the
//! corresponding functions.
//!
//! ```
//! use palette::{Lab, Pixel, Srgb};
//! use kmeans_colors::{get_kmeans_lab, map_indices_to_colors_lab, KmeansLab};
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
//! # let buffer;
//! // Convert RGB [u8] buffer to Lab for k-means
//! let lab: Vec<Lab> = Srgb::from_raw_slice(&img_vec)
//!     .iter()
//!     .map(|x| x.into_format().into())
//!     .collect();
//!
//! // Iterate over amount of runs keeping best results
//! let mut result = KmeansLab::new();
//! (0..runs).for_each(|i| {
//!     let run_result = get_kmeans_lab(
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
//! buffer = map_indices_to_colors_lab(&result.centroids, &result.indices);
//! # assert_eq!(buffer, [119, 119, 119, 119, 119, 119]);
//! ```
//!
//! Because the initial seeds are random, the k-means calculation should be run
//! multiple times in order to assure that the best result has been found. The
//! algorithm may find itself in local minima that is not the optimal result.
//! This is especially so for Lab but RGB may only need one run.
//!
//! The binary uses `8` as the default `k`. The iteration limit is set to `20`,
//! RGB usually converges in under 10 iterations depending on the `k`. The
//! convergence factor defaults to `8.0` for Lab and `0.0025` for RGB. The
//! number of runs defaults to `3` for one of the binary subcommands. Through
//! testing, these numbers were found to be an adequate tradeoff between
//! performance and accuracy. If the results do not appear correct, raise the
//! iteration limit as convergence was probably not met.
//!
//! ## Getting the dominant color
//!
//! After k-means calculation, the dominant color can be found by sorting the
//! results and taking the contents of the first vector. By default, the
//! `sort_indexed_colors_*` functions sort from darkest to lightest color. The
//! functions return a tuple of the form `(Color, f32, u8)`: a color, the
//! percentage of its presence in the buffer, and the centroid index to which it
//! corresponds.
//!
//! ```no_run
//! # use palette::{Lab, Pixel, Srgb};
//! # use kmeans_colors::{get_kmeans_lab, map_indices_to_colors_lab, KmeansLab};
//! use kmeans_colors::sort_indexed_colors_lab;
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
//! # let mut result = KmeansLab::new();
//! # (0..runs).for_each(|i| {
//! #     let run_result = get_kmeans_lab(
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
//! // Using result from k-means example, sort by percentage
//! let mut res = sort_indexed_colors_lab(&result.centroids, &result.indices);
//! res.sort_unstable_by(|a, b| (b.1).partial_cmp(&a.1).unwrap());
//!
//! // The most appearing color will be the first element of the vec
//! let dominant_color = res.first().unwrap().0;
//! ```

mod kmeans;
mod sort;

pub use kmeans::{
    get_closest_centroid_lab, get_closest_centroid_rgb, get_kmeans_lab, get_kmeans_rgb,
    map_indices_to_colors_lab, map_indices_to_colors_rgb, KmeansLab, KmeansRgb,
};
pub use sort::{sort_indexed_colors_lab, sort_indexed_colors_rgb};
