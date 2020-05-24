#[cfg(feature = "palette_color")]
use palette::{Lab, Pixel, Srgb};

use rand::{Rng, SeedableRng};

/// A trait for enabling k-means calculation of a data type.
pub trait Calculate: Sized {
    /// Find a points's nearest centroid, index the point with that centroid.
    fn get_closest_centroid(buffer: &[Self], centroids: &[Self], indices: &mut Vec<u8>);

    /// Find the new centroid locations based on the average of the points that
    /// correspond to the centroid. If no point correspond, the centroid is
    /// re-initialized with a random point.
    fn recalculate_centroids(
        rng: &mut impl Rng,
        centroids: &mut [Self],
        buf: &[Self],
        indices: &[u8],
    );

    /// Calculate the distance metric for convergence comparison.
    fn check_loop(centroids: &[Self], old_centroids: &[Self]) -> f32;

    /// Generate random point.
    fn create_random(rng: &mut impl Rng) -> Self;

    /// Calculate the geometric distance between two points, the square root is
    /// omitted.
    fn difference(c1: &Self, c2: &Self) -> f32;

    /// Map point indices to each centroid for output buffer.
    fn map_indices_to_centroids(centroids: &[Self], indices: &[u8]) -> Vec<u8>;
}

/// Result of k-means calculation with convergence score, centroids, and indexed
/// buffer.
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

/// Find the k-means centroids of a buffer. `max_iter` and `converge` are used
/// together to determine when the k-means calculation has converged. When the
/// `score` is less than `converge` or the number of iterations reaches
/// `max_iter`, the calculation is complete.
///
/// - `k` - number of clusters.
/// - `max_iter` - maximum number of iterations.
/// - `converge` - threshold for convergence.
/// - `verbose` - flag for printing convergence information to console.
/// - `buf` - array of points.
/// - `seed` - seed for the random number generator.
pub fn get_kmeans<C: Calculate + Clone>(
    k: u8,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    buf: &[C],
    seed: u64,
) -> Kmeans<C> {
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut centroids: Vec<C> = Vec::with_capacity(k as usize);
    (0..k).for_each(|_| centroids.push(C::create_random(&mut rng)));

    // Initialize indexed buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centroids = centroids.clone();
    let mut indices: Vec<u8> = Vec::with_capacity(buf.len());

    // Main loop: find nearest centroids and recalculate means until convergence
    loop {
        C::get_closest_centroid(&buf, &centroids, &mut indices);
        C::recalculate_centroids(&mut rng, &mut centroids, &buf, &indices);

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
impl Calculate for Lab {
    fn get_closest_centroid(lab: &[Lab], centroids: &[Lab], indices: &mut Vec<u8>) {
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
        centroids: &mut [Lab],
        buf: &[Lab],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut l = 0.0;
            let mut a = 0.0;
            let mut b = 0.0;
            let mut counter: u32 = 0;
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

    fn check_loop(centroids: &[Lab], old_centroids: &[Lab]) -> f32 {
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

    fn create_random(rng: &mut impl Rng) -> Lab {
        Lab::new(
            rng.gen_range(0.0, 100.0),
            rng.gen_range(-128.0, 127.0),
            rng.gen_range(-128.0, 127.0),
        )
    }

    fn difference(c1: &Lab, c2: &Lab) -> f32 {
        (c1.l - c2.l) * (c1.l - c2.l)
            + (c1.a - c2.a) * (c1.a - c2.a)
            + (c1.b - c2.b) * (c1.b - c2.b)
    }

    fn map_indices_to_centroids(centroids: &[Lab], indices: &[u8]) -> Vec<u8> {
        let srgb: Vec<Srgb<u8>> = indices
            .iter()
            .map(|x| {
                centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .map(|x| Srgb::from(*x).into_format())
            .collect();

        Srgb::into_raw_slice(&srgb).to_vec()
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
        centroids: &mut [Srgb],
        buf: &[Srgb],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;
            let mut counter: u32 = 0;
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

    fn create_random(rng: &mut impl Rng) -> Srgb {
        Srgb::new(rng.gen(), rng.gen(), rng.gen())
    }

    fn difference(c1: &Srgb, c2: &Srgb) -> f32 {
        (c1.red - c2.red) * (c1.red - c2.red)
            + (c1.green - c2.green) * (c1.green - c2.green)
            + (c1.blue - c2.blue) * (c1.blue - c2.blue)
    }

    fn map_indices_to_centroids(centroids: &[Srgb], indices: &[u8]) -> Vec<u8> {
        let srgb: Vec<Srgb<u8>> = indices
            .iter()
            .map(|x| {
                centroids
                    .get(*x as usize)
                    .unwrap_or_else(|| centroids.last().unwrap())
                    .into_format()
            })
            .collect();

        Srgb::into_raw_slice(&srgb).to_vec()
    }
}
