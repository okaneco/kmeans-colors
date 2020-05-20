//! Calculate the k-means colors of an image.

mod kmeans;
mod sort;

pub use kmeans::{
    get_closest_centroid_lab, get_closest_centroid_rgb, get_kmeans_lab, get_kmeans_rgb,
    map_indices_to_colors_lab, map_indices_to_colors_rgb, KmeansLab, KmeansRgb,
};
pub use sort::{sort_indexed_colors_lab, sort_indexed_colors_rgb};
