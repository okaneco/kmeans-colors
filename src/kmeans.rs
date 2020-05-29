use core::convert::TryFrom;

#[cfg(feature = "palette_color")]
use palette::white_point::WhitePoint;
#[cfg(feature = "palette_color")]
use palette::{Component, Lab, Laba, Srgb, Srgba};

use rand::{Rng, SeedableRng};

/// A trait for enabling k-means calculation of a data type.
pub trait Calculate: Sized {
    /// Find a points's nearest centroid, index the point with that centroid.
    fn get_closest_centroid<T>(buffer: &[Self], centroids: &[Self], indices: &mut Vec<T>)
    where
        T: TryFrom<usize>;

    /// Find the new centroid locations based on the average of the points that
    /// correspond to the centroid. If no points correspond, the centroid is
    /// re-initialized with a random point.
    fn recalculate_centroids<T: TryFrom<usize>>(
        rng: &mut impl Rng,
        buf: &[Self],
        centroids: &mut [Self],
        indices: &[T],
    ) where
        T: TryFrom<usize> + PartialEq;

    /// Calculate the distance metric for convergence comparison.
    fn check_loop(centroids: &[Self], old_centroids: &[Self]) -> f32;

    /// Generate random point.
    fn create_random(rng: &mut impl Rng) -> Self;

    /// Calculate the geometric distance between two points, the square root is
    /// omitted.
    fn difference(c1: &Self, c2: &Self) -> f32;
}

/// Result of k-means calculation with convergence score, centroids, and indexed
/// buffer.
#[derive(Clone, Debug, Default)]
pub struct Kmeans<C, U = u8>
where
    C: Calculate,
    U: TryFrom<usize>,
{
    /// Sum of squares distance metric for centroids compared to old centroids.
    pub score: f32,
    /// Points determined to be centroids of input buffer.
    pub centroids: Vec<C>,
    /// Buffer of points indexed to centroids.
    pub indices: Vec<U>,
}

impl<C, U> Kmeans<C, U>
where
    C: Calculate,
    U: TryFrom<usize>,
{
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
pub fn get_kmeans<C, U>(
    k: usize,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    buf: &[C],
    seed: u64,
) -> Kmeans<C, U>
where
    C: Calculate + Clone,
    U: TryFrom<usize> + PartialEq,
{
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut centroids: Vec<C> = Vec::with_capacity(k);
    (0..k).for_each(|_| centroids.push(C::create_random(&mut rng)));

    // Initialize indexed buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centroids = centroids.clone();
    let mut indices: Vec<U> = Vec::with_capacity(buf.len());

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
    fn get_closest_centroid<T>(lab: &[Lab<Wp>], centroids: &[Lab<Wp>], indices: &mut Vec<T>)
    where
        T: TryFrom<usize>,
    {
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
            indices.push(T::try_from(index).ok().unwrap());
        }
    }

    fn recalculate_centroids<T>(
        mut rng: &mut impl Rng,
        buf: &[Lab<Wp>],
        centroids: &mut [Lab<Wp>],
        indices: &[T],
    ) where
        T: TryFrom<usize> + PartialEq,
    {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut l = 0.0;
            let mut a = 0.0;
            let mut b = 0.0;
            let mut counter: u64 = 0;
            for (jdx, color) in indices.iter().zip(buf) {
                if *jdx == T::try_from(idx).ok().unwrap() {
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
    fn get_closest_centroid<T>(rgb: &[Srgb], centroids: &[Srgb], indices: &mut Vec<T>)
    where
        T: TryFrom<usize>,
    {
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
            indices.push(T::try_from(index).ok().unwrap());
        }
    }

    fn recalculate_centroids<T>(
        mut rng: &mut impl Rng,
        buf: &[Srgb],
        centroids: &mut [Srgb],
        indices: &[T],
    ) where
        T: TryFrom<usize> + PartialEq,
    {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;
            let mut counter: u64 = 0;
            for (jdx, color) in indices.iter().zip(buf) {
                if *jdx == T::try_from(idx).ok().unwrap() {
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

/// A trait for mapping colors to their corresponding centroids.
#[cfg(feature = "palette_color")]
pub trait MapColor: Sized {
    /// Map pixel indices to each centroid for output buffer.
    fn map_indices_to_centroids<T>(centroids: &[Self], indices: &[T]) -> Vec<Self>
    where
        T: Copy + Into<usize>;
}

#[cfg(feature = "palette_color")]
impl<Wp> MapColor for Lab<Wp>
where
    Wp: WhitePoint,
{
    #[inline]
    fn map_indices_to_centroids<T>(centroids: &[Self], indices: &[T]) -> Vec<Self>
    where
        T: Copy + Into<usize>,
    {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(usize::try_from(*x).unwrap())
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
    fn map_indices_to_centroids<T>(centroids: &[Self], indices: &[T]) -> Vec<Self>
    where
        T: Copy + Into<usize>,
    {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(usize::try_from(*x).unwrap())
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
    fn map_indices_to_centroids<T>(centroids: &[Self], indices: &[T]) -> Vec<Self>
    where
        T: Copy + Into<usize>,
    {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(usize::try_from(*x).unwrap())
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
    fn map_indices_to_centroids<T>(centroids: &[Self], indices: &[T]) -> Vec<Self>
    where
        T: Copy + Into<usize>,
    {
        indices
            .iter()
            .map(|x| {
                *centroids
                    .get(usize::try_from(*x).unwrap())
                    .unwrap_or_else(|| centroids.last().unwrap())
            })
            .collect()
    }
}
