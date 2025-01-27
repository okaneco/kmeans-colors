#[cfg(feature = "palette_color")]
use num_traits::{Float, FromPrimitive, Zero};
#[cfg(feature = "palette_color")]
use palette::{rgb::Rgb, rgb::Rgba, Lab};
use rand::Rng;

use crate::kmeans::{Calculate, Hamerly, HamerlyCentroids, HamerlyPoint};

#[cfg(feature = "palette_color")]
impl<Wp, T> Calculate for Lab<Wp, T>
where
    T: Float + FromPrimitive + Zero,
    Lab<Wp, T>: core::ops::AddAssign<Lab<Wp, T>> + Default,
{
    #[allow(clippy::cast_possible_truncation)]
    fn get_closest_centroid(lab: &[Lab<Wp, T>], centroids: &[Lab<Wp, T>], indices: &mut Vec<u8>) {
        for color in lab.iter() {
            let mut index = 0;
            let mut diff;
            let mut min = f32::MAX;
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

    #[allow(clippy::cast_precision_loss)]
    fn recalculate_centroids(
        mut rng: &mut impl Rng,
        buf: &[Lab<Wp, T>],
        centroids: &mut [Lab<Wp, T>],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut temp = Lab::<Wp, T>::default();
            let mut counter: u64 = 0;
            for (&jdx, &color) in indices.iter().zip(buf) {
                if jdx as usize == idx {
                    temp += color;
                    counter += 1;
                }
            }
            if counter != 0 {
                *cent = temp / T::from_f64(counter as f64).unwrap();
            } else {
                *cent = Self::create_random(&mut rng);
            }
        }
    }

    fn check_loop(centroids: &[Lab<Wp, T>], old_centroids: &[Lab<Wp, T>]) -> f32 {
        let mut temp = Lab::<Wp, T>::default();
        for (&c0, &c1) in centroids.iter().zip(old_centroids) {
            temp += c0 - c1;
        }

        ((temp.l).powi(2) + (temp.a).powi(2) + (temp.b).powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }

    #[inline]
    fn create_random(rng: &mut impl Rng) -> Lab<Wp, T> {
        Lab::<Wp, T>::new(
            T::from_f64(rng.random_range(0.0..=100.0)).unwrap(),
            T::from_f64(rng.random_range(-128.0..=127.0)).unwrap(),
            T::from_f64(rng.random_range(-128.0..=127.0)).unwrap(),
        )
    }

    #[inline]
    fn difference(c1: &Lab<Wp, T>, c2: &Lab<Wp, T>) -> f32 {
        let temp = *c1 - *c2;

        ((temp.l).powi(2) + (temp.a).powi(2) + (temp.b).powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }
}

#[cfg(feature = "palette_color")]
impl<S, T> Calculate for Rgb<S, T>
where
    T: Float + FromPrimitive + Zero,
    Rgb<S, T>: core::ops::AddAssign<Rgb<S, T>> + Default,
{
    #[allow(clippy::cast_possible_truncation)]
    fn get_closest_centroid(rgb: &[Rgb<S, T>], centroids: &[Rgb<S, T>], indices: &mut Vec<u8>) {
        for color in rgb.iter() {
            let mut index = 0;
            let mut diff;
            let mut min = f32::MAX;
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

    #[allow(clippy::cast_precision_loss)]
    fn recalculate_centroids(
        mut rng: &mut impl Rng,
        buf: &[Rgb<S, T>],
        centroids: &mut [Rgb<S, T>],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut temp = Rgb::<S, T>::new(T::zero(), T::zero(), T::zero());
            let mut counter: u64 = 0;
            for (&jdx, &color) in indices.iter().zip(buf) {
                if jdx as usize == idx {
                    temp += color;
                    counter += 1;
                }
            }
            if counter != 0 {
                *cent = temp / T::from_f64(counter as f64).unwrap();
            } else {
                *cent = Self::create_random(&mut rng);
            }
        }
    }

    fn check_loop(centroids: &[Rgb<S, T>], old_centroids: &[Rgb<S, T>]) -> f32 {
        let mut temp = Rgb::<S, T>::default();
        for (&c0, &c1) in centroids.iter().zip(old_centroids) {
            temp += c0 - c1;
        }

        ((temp.red).powi(2) + (temp.green).powi(2) + (temp.blue).powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }

    #[inline]
    fn create_random(rng: &mut impl Rng) -> Rgb<S, T> {
        Rgb::<S, T>::new(
            T::from_f64(rng.random_range(0.0..=1.0)).unwrap(),
            T::from_f64(rng.random_range(0.0..=1.0)).unwrap(),
            T::from_f64(rng.random_range(0.0..=1.0)).unwrap(),
        )
    }

    #[inline]
    fn difference(c1: &Rgb<S, T>, c2: &Rgb<S, T>) -> f32 {
        let temp = *c1 - *c2;

        ((temp.red).powi(2) + (temp.green).powi(2) + (temp.blue).powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }
}

#[cfg(feature = "palette_color")]
impl<Wp, T> Hamerly for Lab<Wp, T>
where
    T: Float + FromPrimitive + Zero,
    Lab<Wp, T>: core::ops::AddAssign<Lab<Wp, T>> + Default,
{
    fn compute_half_distances(centers: &mut HamerlyCentroids<Self>) {
        // Find each center's closest center
        for ((i, ci), half_dist) in centers
            .centroids
            .iter()
            .enumerate()
            .zip(centers.half_distances.iter_mut())
        {
            let mut diff;
            let mut min = f32::MAX;
            for (j, cj) in centers.centroids.iter().enumerate() {
                // Don't compare centroid to itself
                if i == j {
                    continue;
                }
                diff = Self::difference(ci, cj);
                if diff < min {
                    min = diff;
                }
            }
            *half_dist = min.sqrt() * 0.5;
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centers: &HamerlyCentroids<Self>,
        points: &mut [HamerlyPoint],
    ) {
        for (val, point) in buffer.iter().zip(points.iter_mut()) {
            // Assign max of lower bound and half distance to z
            let z = centers
                .half_distances
                .get(point.index as usize)
                .unwrap()
                .max(point.lower_bound);

            if point.upper_bound <= z {
                continue;
            }

            // Tighten upper bound
            point.upper_bound =
                Self::difference(val, centers.centroids.get(point.index as usize).unwrap()).sqrt();

            if point.upper_bound <= z {
                continue;
            }

            // Find the two closest centers to current point and their distances
            if centers.centroids.len() < 2 {
                continue;
            }

            let mut min1 = Self::difference(val, centers.centroids.first().unwrap());
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

            if c1 as u8 != point.index {
                point.index = c1 as u8;
                point.upper_bound = min1.sqrt();
            }
            point.lower_bound = min2.sqrt();
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn recalculate_centroids_hamerly(
        mut rng: &mut impl Rng,
        buf: &[Self],
        centers: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    ) {
        for ((idx, cent), delta) in centers
            .centroids
            .iter_mut()
            .enumerate()
            .zip(centers.deltas.iter_mut())
        {
            let mut temp = Lab::<Wp, T>::default();
            let mut counter: u64 = 0;
            for (point, &color) in points.iter().zip(buf) {
                if point.index as usize == idx {
                    temp += color;
                    counter += 1;
                }
            }
            if counter != 0 {
                let new_color = temp / T::from_f64(counter as f64).unwrap();
                *delta = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            } else {
                let new_color = Self::create_random(&mut rng);
                *delta = Self::difference(cent, &new_color).sqrt();
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

        for point in points.iter_mut() {
            point.upper_bound += centers.deltas.get(point.index as usize).unwrap();
            point.lower_bound -= delta_p;
        }
    }
}

#[cfg(feature = "palette_color")]
impl<S, T> Hamerly for Rgb<S, T>
where
    T: Float + FromPrimitive + Zero,
    Rgb<S, T>: core::ops::AddAssign<Rgb<S, T>> + Default,
{
    fn compute_half_distances(centers: &mut HamerlyCentroids<Self>) {
        // Find each center's closest center
        for ((i, ci), half_dist) in centers
            .centroids
            .iter()
            .enumerate()
            .zip(centers.half_distances.iter_mut())
        {
            let mut diff;
            let mut min = f32::MAX;
            for (j, cj) in centers.centroids.iter().enumerate() {
                // Don't compare centroid to itself
                if i == j {
                    continue;
                }
                diff = Self::difference(ci, cj);
                if diff < min {
                    min = diff;
                }
            }
            *half_dist = min.sqrt() * 0.5;
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centers: &HamerlyCentroids<Self>,
        points: &mut [HamerlyPoint],
    ) {
        for (val, point) in buffer.iter().zip(points.iter_mut()) {
            // Assign max of lower bound and half distance to z
            let z = centers
                .half_distances
                .get(point.index as usize)
                .unwrap()
                .max(point.lower_bound);

            if point.upper_bound <= z {
                continue;
            }

            // Tighten upper bound
            point.upper_bound =
                Self::difference(val, centers.centroids.get(point.index as usize).unwrap()).sqrt();

            if point.upper_bound <= z {
                continue;
            }

            // Find the two closest centers to current point and their distances
            if centers.centroids.len() < 2 {
                continue;
            }

            let mut min1 = Self::difference(val, centers.centroids.first().unwrap());
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

            if c1 as u8 != point.index {
                point.index = c1 as u8;
                point.upper_bound = min1.sqrt();
            }
            point.lower_bound = min2.sqrt();
        }
    }

    #[allow(clippy::cast_precision_loss)]
    fn recalculate_centroids_hamerly(
        mut rng: &mut impl Rng,
        buf: &[Self],
        centers: &mut HamerlyCentroids<Self>,
        points: &[HamerlyPoint],
    ) {
        for ((idx, cent), delta) in centers
            .centroids
            .iter_mut()
            .enumerate()
            .zip(centers.deltas.iter_mut())
        {
            let mut temp = Rgb::<S, T>::default();
            let mut counter: u64 = 0;
            for (point, &color) in points.iter().zip(buf) {
                if point.index as usize == idx {
                    temp += color;
                    counter += 1;
                }
            }
            if counter != 0 {
                let new_color = temp / T::from_f64(counter as f64).unwrap();
                *delta = Self::difference(cent, &new_color).sqrt();
                *cent = new_color;
            } else {
                let new_color = Self::create_random(&mut rng);
                *delta = Self::difference(cent, &new_color).sqrt();
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

        for point in points.iter_mut() {
            point.upper_bound += centers.deltas.get(point.index as usize).unwrap();
            point.lower_bound -= delta_p;
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
impl<Wp, T> MapColor for Lab<Wp, T>
where
    T: Copy,
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
impl<Wp, T> MapColor for palette::Laba<Wp, T>
where
    T: Copy,
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
impl<S, T> MapColor for Rgb<S, T>
where
    T: Copy,
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
impl<S, T> MapColor for Rgba<S, T>
where
    T: Copy,
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
