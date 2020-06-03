#[cfg(feature = "palette_color")]
use palette::white_point::WhitePoint;
#[cfg(feature = "palette_color")]
use palette::{Component, Lab, Laba, Srgb, Srgba};

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
    (0..k).for_each(|_| centroids.push(C::create_random(&mut rng)));

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

#[cfg(feature = "palette_color")]
impl<Wp: WhitePoint> Calculate for Lab<Wp> {
    fn get_closest_centroid(lab: &[Lab<Wp>], centroids: &[Lab<Wp>], indices: &mut Vec<u8>) {
        for color in lab.iter() {
            let mut index = 0;
            let mut diff;
            let mut min = core::f32::MAX;
            for (idx, cent) in centroids.iter().enumerate() {
                diff = Self::difference(color, cent);
                if diff < min {
                    min = diff;
                    index = idx;
                }
            }
            indices.push(index as u8);
        }
    }

    fn recalculate_centroids(
        mut rng: &mut impl Rng,
        buf: &[Lab<Wp>],
        centroids: &mut [Lab<Wp>],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut l = 0.0;
            let mut a = 0.0;
            let mut b = 0.0;
            let mut counter: u64 = 0;
            for (jdx, color) in indices.iter().zip(buf) {
                if *jdx == idx as u8 {
                    l += color.l;
                    a += color.a;
                    b += color.b;
                    counter += 1;
                }
            }
            if counter != 0 {
                *cent = Lab {
                    l: l / (counter as f32),
                    a: a / (counter as f32),
                    b: b / (counter as f32),
                    white_point: core::marker::PhantomData,
                };
            } else {
                *cent = Self::create_random(&mut rng);
            }
        }
    }

    fn check_loop(centroids: &[Lab<Wp>], old_centroids: &[Lab<Wp>]) -> f32 {
        let mut l = 0.0;
        let mut a = 0.0;
        let mut b = 0.0;
        for c in centroids.iter().zip(old_centroids) {
            l += (c.0).l - (c.1).l;
            a += (c.0).a - (c.1).a;
            b += (c.0).b - (c.1).b;
        }

        l * l + a * a + b * b
    }

    #[inline]
    fn create_random(rng: &mut impl Rng) -> Lab<Wp> {
        Lab::with_wp(
            rng.gen_range(0.0, 100.0),
            rng.gen_range(-128.0, 127.0),
            rng.gen_range(-128.0, 127.0),
        )
    }

    #[inline]
    fn difference(c1: &Lab<Wp>, c2: &Lab<Wp>) -> f32 {
        (c1.l - c2.l) * (c1.l - c2.l)
            + (c1.a - c2.a) * (c1.a - c2.a)
            + (c1.b - c2.b) * (c1.b - c2.b)
    }
}

#[cfg(feature = "palette_color")]
impl Calculate for Srgb {
    fn get_closest_centroid(rgb: &[Srgb], centroids: &[Srgb], indices: &mut Vec<u8>) {
        for color in rgb.iter() {
            let mut index = 0;
            let mut diff;
            let mut min = core::f32::MAX;
            for (idx, cent) in centroids.iter().enumerate() {
                diff = Self::difference(color, cent);
                if diff < min {
                    min = diff;
                    index = idx;
                }
            }
            indices.push(index as u8);
        }
    }

    fn recalculate_centroids(
        mut rng: &mut impl Rng,
        buf: &[Srgb],
        centroids: &mut [Srgb],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;
            let mut counter: u64 = 0;
            for (jdx, color) in indices.iter().zip(buf) {
                if *jdx == idx as u8 {
                    red += color.red;
                    green += color.green;
                    blue += color.blue;
                    counter += 1;
                }
            }
            if counter != 0 {
                *cent = Srgb {
                    red: red / (counter as f32),
                    green: green / (counter as f32),
                    blue: blue / (counter as f32),
                    standard: core::marker::PhantomData,
                };
            } else {
                *cent = Self::create_random(&mut rng);
            }
        }
    }

    fn check_loop(centroids: &[Srgb], old_centroids: &[Srgb]) -> f32 {
        let mut red = 0.0;
        let mut green = 0.0;
        let mut blue = 0.0;
        for c in centroids.iter().zip(old_centroids) {
            red += (c.0).red - (c.1).red;
            green += (c.0).green - (c.1).green;
            blue += (c.0).blue - (c.1).blue;
        }

        red * red + green * green + blue * blue
    }

    #[inline]
    fn create_random(rng: &mut impl Rng) -> Srgb {
        Srgb::new(rng.gen(), rng.gen(), rng.gen())
    }

    #[inline]
    fn difference(c1: &Srgb, c2: &Srgb) -> f32 {
        (c1.red - c2.red) * (c1.red - c2.red)
            + (c1.green - c2.green) * (c1.green - c2.green)
            + (c1.blue - c2.blue) * (c1.blue - c2.blue)
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
    pub fn new() -> Self {
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
/// However, this method incurs additional overhead that makes it perform about
/// the same or slightly worse at low center counts. Benchmark the functions to
/// see which performs better for your use case.
///
/// Below about `k=6`, the default LLoyd's algorithm seems to perform better for
/// three dimensional points like colors but it depends on the data. If there
/// are many similar points, the algorithm may end up with the extra overhead
/// and having to compute the inner loop many times as the centers fluctuate.
/// Hamerly's algorithm excels when there is clear segmentation of clusters.
/// Those clusters tend to converge early and all the points that belong
/// to them can skip their distance calculations.
///
/// ## Referenece
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
    (0..k).for_each(|_| centers.centroids.push(Calculate::create_random(&mut rng)));

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

#[cfg(feature = "palette_color")]
impl<Wp: WhitePoint> Hamerly for Lab<Wp> {
    fn compute_half_distances(centers: &mut HamerlyCentroids<Self>) {
        // Find each center's closest center
        for i in 0..centers.centroids.len() {
            let mut diff;
            let mut min = f32::MAX;
            for j in 0..centers.centroids.len() {
                // Don't compare centroid to itself
                if i == j {
                    continue;
                }
                diff = Self::difference(
                    &centers.centroids.get(i).unwrap(),
                    &centers.centroids.get(j).unwrap(),
                );
                if diff < min {
                    min = diff;
                }
            }
            centers.half_distances[i] = min.sqrt() * 0.5;
        }
    }

    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centers: &HamerlyCentroids<Self>,
        points: &mut [HamerlyPoint],
    ) {
        for (i, val) in buffer.iter().enumerate() {
            assert!(i < buffer.len());
            // Assign max of lower bound and half distance to z
            let z = centers
                .half_distances
                .get(points[i].index as usize)
                .unwrap()
                .max(points[i].lower_bound);

            if points[i].upper_bound <= z {
                continue;
            }

            // Tighten upper bound
            points[i].upper_bound = Self::difference(
                val,
                centers.centroids.get(points[i].index as usize).unwrap(),
            )
            .sqrt();

            if points[i].upper_bound <= z {
                continue;
            }

            // Find the two closest centers to current point and their distances
            if centers.centroids.len() < 2 {
                continue;
            }

            let mut min1 = Self::difference(val, centers.centroids.get(0).unwrap());
            let mut min2 = f32::MAX;
            let mut c1 = 0;
            for j in 1..centers.centroids.len() {
                let diff = Self::difference(val, centers.centroids.get(j).unwrap());
                if diff < min1 {
                    min2 = min1;
                    min1 = diff;
                    c1 = j;
                    continue;
                }
                if diff < min2 {
                    min2 = diff;
                }
            }

            if c1 as u8 != points[i].index {
                points[i].index = c1 as u8;
                points[i].upper_bound = min1.sqrt();
            }
            points[i].lower_bound = min2.sqrt();
        }
    }

    fn recalculate_centroids_hamerly(
        mut rng: &mut impl Rng,
        buf: &[Self],
        centers: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    ) {
        for (idx, cent) in centers.centroids.iter_mut().enumerate() {
            let mut l = 0.0;
            let mut a = 0.0;
            let mut b = 0.0;
            let mut counter: u64 = 0;
            for (point, color) in points.iter().zip(buf) {
                if point.index == idx as u8 {
                    l += color.l;
                    a += color.a;
                    b += color.b;
                    counter += 1;
                }
            }
            if counter != 0 {
                let new_color = Lab {
                    l: l / (counter as f32),
                    a: a / (counter as f32),
                    b: b / (counter as f32),
                    white_point: core::marker::PhantomData,
                };
                centers.deltas[idx] = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            } else {
                let new_color = Self::create_random(&mut rng);
                centers.deltas[idx] = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            }
        }
    }

    fn update_bounds(centers: &HamerlyCentroids<Self>, points: &mut [HamerlyPoint]) {
        let mut delta_p = 0.0;
        for c in centers.deltas.iter() {
            if *c > delta_p {
                delta_p = *c;
            }
        }

        for i in 0..points.len() {
            points[i].upper_bound += centers.deltas.get(points[i].index as usize).unwrap();
            points[i].lower_bound -= delta_p;
        }
    }
}

#[cfg(feature = "palette_color")]
impl Hamerly for Srgb {
    fn compute_half_distances(centers: &mut HamerlyCentroids<Self>) {
        // Find each center's closest center
        for i in 0..centers.centroids.len() {
            let mut diff;
            let mut min = f32::MAX;
            for j in 0..centers.centroids.len() {
                // Don't compare centroid to itself
                if i == j {
                    continue;
                }
                diff = Self::difference(
                    &centers.centroids.get(i).unwrap(),
                    &centers.centroids.get(j).unwrap(),
                );
                if diff < min {
                    min = diff;
                }
            }
            centers.half_distances[i] = min.sqrt() * 0.5;
        }
    }

    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centers: &HamerlyCentroids<Self>,
        points: &mut [HamerlyPoint],
    ) {
        for (i, val) in buffer.iter().enumerate() {
            assert!(i < buffer.len());
            // Assign max of lower bound and half distance to z
            let z = centers
                .half_distances
                .get(points[i].index as usize)
                .unwrap()
                .max(points[i].lower_bound);

            if points[i].upper_bound <= z {
                continue;
            }

            // Tighten upper bound
            points[i].upper_bound = Self::difference(
                val,
                centers.centroids.get(points[i].index as usize).unwrap(),
            )
            .sqrt();

            if points[i].upper_bound <= z {
                continue;
            }

            // Find the two closest centers to current point and their distances
            if centers.centroids.len() < 2 {
                continue;
            }

            let mut min1 = Self::difference(val, centers.centroids.get(0).unwrap());
            let mut min2 = f32::MAX;
            let mut c1 = 0;
            for j in 1..centers.centroids.len() {
                let diff = Self::difference(val, centers.centroids.get(j).unwrap());
                if diff < min1 {
                    min2 = min1;
                    min1 = diff;
                    c1 = j;
                    continue;
                }
                if diff < min2 {
                    min2 = diff;
                }
            }

            if c1 as u8 != points[i].index {
                points[i].index = c1 as u8;
                points[i].upper_bound = min1.sqrt();
            }
            points[i].lower_bound = min2.sqrt();
        }
    }

    fn recalculate_centroids_hamerly(
        mut rng: &mut impl Rng,
        buf: &[Self],
        centers: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    ) {
        for (idx, cent) in centers.centroids.iter_mut().enumerate() {
            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;
            let mut counter: u64 = 0;
            for (point, color) in points.iter().zip(buf) {
                if point.index == idx as u8 {
                    red += color.red;
                    green += color.green;
                    blue += color.blue;
                    counter += 1;
                }
            }
            if counter != 0 {
                let new_color = Srgb {
                    red: red / (counter as f32),
                    green: green / (counter as f32),
                    blue: blue / (counter as f32),
                    standard: core::marker::PhantomData,
                };
                centers.deltas[idx] = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            } else {
                let new_color = Self::create_random(&mut rng);
                centers.deltas[idx] = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            }
        }
    }

    fn update_bounds(centers: &HamerlyCentroids<Self>, points: &mut [HamerlyPoint]) {
        let mut delta_p = 0.0;
        for c in centers.deltas.iter() {
            if *c > delta_p {
                delta_p = *c;
            }
        }

        for i in 0..points.len() {
            points[i].upper_bound += centers.deltas.get(points[i].index as usize).unwrap();
            points[i].lower_bound -= delta_p;
        }
    }
}

/// A trait for mapping colors to their corresponding centroids.
#[cfg(feature = "palette_color")]
pub trait MapColor: Sized {
    /// Map pixel indices to each centroid for output buffer.
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self>;
}

#[cfg(feature = "palette_color")]
impl<Wp> MapColor for Lab<Wp>
where
    Wp: WhitePoint,
{
    #[inline]
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self> {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}

#[cfg(feature = "palette_color")]
impl<Wp> MapColor for Laba<Wp>
where
    Wp: WhitePoint,
{
    #[inline]
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self> {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}

#[cfg(feature = "palette_color")]
impl<C> MapColor for Srgb<C>
where
    C: Component,
{
    #[inline]
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self> {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}

#[cfg(feature = "palette_color")]
impl<C> MapColor for Srgba<C>
where
    C: Component,
{
    #[inline]
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<Self> {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}
