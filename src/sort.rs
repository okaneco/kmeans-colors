/// Struct containing a centroid, its percentage within a buffer, and the
/// centroid's index.
#[derive(Clone, Debug, Default)]
pub struct CentroidData<C: crate::Calculate> {
    /// A k-means centroid.
    pub centroid: C,
    /// The percentage a centroid appears in a buffer.
    pub percentage: f32,
    /// The centroid's index.
    pub index: u8,
}

/// A trait for sorting indexed k-means colors.
pub trait Sort: Sized + crate::Calculate {
    /// Returns the centroid with the largest percentage.
    fn get_dominant_color(data: &[CentroidData<Self>]) -> Option<Self>;

    /// Sorts centroids by luminosity and calculates the percentage of each
    /// color in the buffer. Returns a `CentroidResult` sorted from darkest to
    /// lightest.
    fn sort_indexed_colors(centroids: &Vec<Self>, indices: &[u8]) -> Vec<CentroidData<Self>>;
}
