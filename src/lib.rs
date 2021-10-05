//! Calculate the k-means of a set of data.
//!
//! # Overview
//!
//! This crate provides traits for implementing and calculating a k-means
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
//! usage.
//!
//! [palette]: https://github.com/Ogeon/palette/
//!
//! ## The `Calculate` trait
//! k-means calculations can be provided for other data types by implementing
//! the [`Calculate`](trait.Calculate.html) trait. Further,
//! [`Hamerly`](trait.Hamerly.html) can be implemented to enable use of the
//! Hamerly optimization and [`get_kmeans_hamerly`][hamerly]. See the `Lab` and
//! `Srgb` implementations in [`colors/kmeans.rs`][kmeans] for examples. These
//! implementations can be used as groundwork for implementing with other types
//! and should not require much modification beyond the distance calculations.
//!
//! [hamerly]: fn.get_kmeans_hamerly.html
//! [kmeans]: ../src/kmeans_colors/colors/kmeans.rs.html#9
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
//! use palette::{FromColor, IntoColor, Lab, Pixel, Srgb};
//! use kmeans_colors::{get_kmeans, Calculate, Kmeans, MapColor, Sort};
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
//!     .map(|x| x.into_format().into_color())
//!     .collect();
//!
//! // Iterate over the runs, keep the best results
//! let mut result = Kmeans::new();
//! for i in 0..runs {
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
//! }
//!
//! // Convert indexed colors back to Srgb<u8> for output
//! let rgb = &result.centroids
//!     .iter()
//!     .map(|x| Srgb::from_color(*x).into_format())
//!     .collect::<Vec<Srgb<u8>>>();
//! let buffer = Srgb::map_indices_to_centroids(&rgb, &result.indices);
//! # assert_eq!(Srgb::into_raw_slice(&buffer), [119, 119, 119, 119, 119, 119]);
//! ```
//!
//! k-means++ is used for centroid initialization. Because the initialization is
//! random, the k-means calculation may be run multiple times to assure that
//! the best result has been found. The algorithm can find itself in a
//! sub-optimal result due to initial centroids, however, one run may suffice if
//! the convergence threshold has been met.
//!
//! The binary uses `8` as the default `k`. The iteration limit is set to `20`.
//! The convergence factor defaults to `5.0` for `Lab` and `0.0025` for `Srgb`.
//! The number of runs defaults to `3` for one of the binary subcommands.
//! If the results do not appear correct, raise the iteration limit as
//! convergence was probably not met.
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
//! # use palette::{FromColor, IntoColor, Lab, Pixel, Srgb};
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
//! #    .map(|x| x.into_format().into_color())
//! #    .collect();
//! # let mut result = Kmeans::new();
//! # for i in 0..runs {
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
//! # }
//! // Using the results from the previous example, process the centroid data
//! let mut res = Lab::sort_indexed_colors(&result.centroids, &result.indices);
//!
//! // We can find the dominant color directly
//! let dominant_color = Lab::get_dominant_color(&res);
//! # assert_eq!(
//! #    Srgb::from_color(dominant_color.unwrap()).into_format::<u8>(),
//! #    Srgb::new(119u8, 119, 119)
//! # );
//!
//! // Or we can manually sort the vec by percentage, and the most appearing
//! // color will be the first element
//! res.sort_unstable_by(|a, b| (b.percentage).partial_cmp(&a.percentage).unwrap());
//! let dominant_color = res.first().unwrap().centroid;
//! ```
#![warn(missing_docs, rust_2018_idioms, unsafe_code)]

#[cfg(feature = "palette_color")]
mod colors;

mod kmeans;
mod plus_plus;
mod sort;

#[cfg(feature = "palette_color")]
pub use colors::MapColor;

pub use kmeans::{
    get_kmeans, get_kmeans_hamerly, Calculate, Hamerly, HamerlyCentroids, HamerlyPoint, Kmeans,
};
pub use plus_plus::init_plus_plus;
pub use sort::{CentroidData, Sort};
