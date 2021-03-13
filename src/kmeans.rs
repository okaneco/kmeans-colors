use rand::{Rng, SeedableRng};

/// A trait for enabling k-means calculation of a data type.
pub trait Calculate: Sized {
    /// Find a points's nearest centroid, index the point with that centroid.
    fn get_closest_centroid(buffer: &[Self], centroids: &[Self], indices: &mut Vec<u8>);

    /// Find the new centroid locations based on the average of the points that
    /// correspond to the centroid. If no points correspond, the centroid is
    /// re-initialized with a random point.
    fn recalculate_centroids(
        rng: &mut impl Rng,
        buf: &[Self],
        centroids: &mut [Self],
        indices: &[u8],
    );

    /// Calculate the distance metric for convergence comparison.
    fn check_loop(centroids: &[Self], old_centroids: &[Self]) -> f32;

    /// Generate random point.
    fn create_random(rng: &mut impl Rng) -> Self;

    /// Calculate the geometric distance between two points, the square root is
    /// omitted.
    fn difference(c1: &Self, c2: &Self) -> f32;
}

/// Struct result of k-means calculation with convergence score, centroids, and
/// indexed buffer.
#[derive(Clone, Debug, Default)]
pub struct Kmeans<C: Calculate> {
    /// Sum of squares distance metric for centroids compared to old centroids.
    pub score: f32,
    /// Points determined to be centroids of input buffer.
    pub centroids: Vec<C>,
    /// Buffer of points indexed to centroids.
    pub indices: Vec<u8>,
}

impl<C: Calculate> Kmeans<C> {
    /// Create a new `Kmeans` struct to contain k-means results.
    pub fn new() -> Self {
        Kmeans {
            score: core::f32::MAX,
            centroids: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Find the k-means centroids of a buffer.
///
/// `max_iter` and `converge` are used together to determine when the k-means
/// calculation has converged. When the `score` is less than `converge` or the
/// number of iterations reaches `max_iter`, the calculation is complete.
///
/// - `k` - number of clusters.
/// - `max_iter` - maximum number of iterations.
/// - `converge` - threshold for convergence.
/// - `verbose` - flag for printing convergence information to console.
/// - `buf` - array of points.
/// - `seed` - seed for the random number generator.
pub fn get_kmeans<C: Calculate + Clone>(
    k: usize,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    buf: &[C],
    seed: u64,
) -> Kmeans<C> {
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut centroids: Vec<C> = Vec::with_capacity(k);
    crate::plus_plus::init_plus_plus(k, &mut rng, buf, &mut centroids);

    // Initialize indexed buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centroids = centroids.clone();
    let mut indices: Vec<u8> = Vec::with_capacity(buf.len());

    // Main loop: find nearest centroids and recalculate means until convergence
    loop {
        C::get_closest_centroid(&buf, &centroids, &mut indices);
        C::recalculate_centroids(&mut rng, &buf, &mut centroids, &indices);

        score = C::check_loop(&centroids, &old_centroids);
        if verbose {
            println!("Score: {}", score);
        }

        // Verify that either the maximum iteration count has been met or the
        // centroids haven't moved beyond a certain threshold since the
        // previous iteration.
        if iterations >= max_iter || score <= converge {
            if verbose {
                println!("Iterations: {}", iterations);
            }
            break;
        }

        indices.clear();
        iterations += 1;
        old_centroids.clone_from(&centroids);
    }

    Kmeans {
        score,
        centroids,
        indices,
    }
}

/// A trait for calculating k-means with the Hamerly algorithm.
pub trait Hamerly: Calculate {
    /// Find the nearest centers and compute their half-distances.
    fn compute_half_distances(centroids: &mut HamerlyCentroids<Self>);

    /// Find a point's nearest centroid, index the point with that centroid.
    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centroids: &HamerlyCentroids<Self>,
        indices: &mut [HamerlyPoint],
    );

    /// Find the new centroid locations based on the average of the points that
    /// correspond to the centroid. If no points correspond, the centroid is
    /// re-initialized with a random point.
    fn recalculate_centroids_hamerly(
        rng: &mut impl Rng,
        buf: &[Self],
        centroids: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    );

    /// Update the lower and upper bounds of each point.
    fn update_bounds(centroids: &HamerlyCentroids<Self>, points: &mut [HamerlyPoint]);
}

/// Struct used for caching data required to compute k-means with the Hamerly
/// algorithm.
#[derive(Clone, Debug)]
pub struct HamerlyCentroids<C: Hamerly> {
    /// Centroid points.
    pub centroids: Vec<C>,
    /// Distances the centroids have moved since the previous iteration.
    pub deltas: Vec<f32>,
    /// Half-distances to nearest centroid.
    pub half_distances: Vec<f32>,
}

impl<C: Hamerly> HamerlyCentroids<C> {
    /// Create a new `HamerlyCentroids` with capacity.
    pub fn new(capacity: usize) -> Self {
        HamerlyCentroids {
            centroids: Vec::with_capacity(capacity),
            deltas: (0..capacity).map(|_| 0.0).collect(),
            half_distances: (0..capacity).map(|_| 0.0).collect(),
        }
    }
}

/// Struct that holds the necessary caching information for points in the
/// Hamerly algorithm implementation.
#[derive(Copy, Clone, Debug)]
pub struct HamerlyPoint {
    /// Index of this point's centroid.
    pub index: u8,
    /// Closest centroid's distance to this point.
    pub upper_bound: f32,
    /// Minimum distance that any centroid beyond the closest centroid can be
    /// to this point.
    pub lower_bound: f32,
}

impl HamerlyPoint {
    /// Create a new `HamerlyPoint`.
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for HamerlyPoint {
    fn default() -> Self {
        HamerlyPoint {
            index: 0,
            upper_bound: f32::MAX,
            lower_bound: 0.0,
        }
    }
}

/// Find the k-means centroids of a buffer using the Hamerly algorithm. Takes
/// the same arguments as [`get_kmeans`](fn.get_kmeans.html) and produces the
/// same results.
///
/// Hamerly uses the triangle inequality and caches one lower and upper bound
/// for each point, which allows it to skip the inner loop of distance
/// calculation for each point more often. Asymptotically, this algorithm
/// performs better than the default algorithm for lower dimensional k-means
/// taking advantage of the fact than some centroids converge very quickly.
/// However, this method incurs additional overhead that may perform worse than
/// the naive method at low center counts like `k=1`. Benchmark the functions to
/// see which performs better for your use case.
///
/// Example implementations for `Lab` and `Srgb` can be found in
/// [`colors/kmeans.rs`][hamerly].
///
/// [hamerly]: ../src/kmeans_colors/colors/kmeans.rs.html#165
///
/// ## Reference
///
/// Hamerly, G., & Drake, J. (2017). Chapter 2 Accelerating Lloyd's Algorithm
/// for k-Means Clustering.
///
/// Hamerly, G. (2010). Making k-means even faster. In: SIAM international
/// conference on data mining.
pub fn get_kmeans_hamerly<C: Hamerly + Clone>(
    k: usize,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    buf: &[C],
    seed: u64,
) -> Kmeans<C> {
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut centers: HamerlyCentroids<C> = HamerlyCentroids::new(k);
    crate::plus_plus::init_plus_plus(k, &mut rng, buf, &mut centers.centroids);

    // Initialize points buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centers = centers.centroids.clone();
    let mut points: Vec<HamerlyPoint> = (0..buf.len()).map(|_| HamerlyPoint::new()).collect();

    // Main loop: find nearest centroids and recalculate means until convergence
    loop {
        C::compute_half_distances(&mut centers);
        C::get_closest_centroid_hamerly(&buf, &centers, &mut points);
        C::recalculate_centroids_hamerly(&mut rng, &buf, &mut centers, &points);

        score = Calculate::check_loop(&centers.centroids, &old_centers);
        if verbose {
            println!("Score: {}", score);
        }

        // Verify that either the maximum iteration count has been met or the
        // centroids haven't moved beyond a certain threshold since the
        // previous iteration.
        if iterations >= max_iter || score <= converge {
            if verbose {
                println!("Iterations: {}", iterations);
            }
            break;
        }

        C::update_bounds(&centers, &mut points);
        old_centers.clone_from(&centers.centroids);
        iterations += 1;
    }

    Kmeans {
        score,
        centroids: centers.centroids,
        indices: points.iter().map(|x| x.index).collect(),
    }
}
