use core::ops::{AddAssign, DivAssign};

use num_traits::{Float, FromPrimitive};

use crate::{Calculate, Hamerly};

impl<T, const N: usize> Calculate for [T; N]
where
    T: Float + FromPrimitive + AddAssign + DivAssign + Default,
    [T; N]: Default,
{
    fn get_closest_centroid(buffer: &[Self], centroids: &[Self], indices: &mut Vec<u8>) {
        indices.extend(buffer.iter().map(|color| {
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
            index as u8
        }))
    }

    fn recalculate_centroids(
        rng: &mut impl rand::Rng,
        buf: &[Self],
        centroids: &mut [Self],
        indices: &[u8],
    ) {
        for (idx, cent) in centroids.iter_mut().enumerate() {
            let mut acc = Self::default();
            let mut counter: u64 = 0;
            for (&jdx, &color) in indices.iter().zip(buf) {
                if jdx as usize == idx {
                    acc.iter_mut().zip(color.iter()).for_each(|(t, &c)| *t += c);
                    counter += 1;
                }
            }
            if counter != 0 {
                let counter_float = T::from_f64(counter as f64).unwrap();
                acc.iter_mut().for_each(|t| *t /= counter_float);
                *cent = acc;
            } else {
                *cent = Self::create_random(rng);
            }
        }
    }

    fn check_loop(centroids: &[Self], old_centroids: &[Self]) -> f32 {
        let mut acc = Self::default();
        for (new_cent, old_cent) in centroids.iter().zip(old_centroids) {
            acc.iter_mut()
                .zip(new_cent.iter())
                .zip(old_cent.iter())
                .for_each(|((t, &new), &old)| *t += new - old);
        }

        acc.iter()
            .fold(T::default(), |accum, t| accum + t.powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }

    // 2023-08 TODO: create_random should take a min and max from a builder object
    fn create_random(rng: &mut impl rand::Rng) -> Self {
        let mut out = Self::default();
        out.iter_mut()
            .for_each(|o| *o = T::from_f64(rng.gen_range(0.0..=1.0)).unwrap());
        out
    }

    fn difference(c1: &Self, c2: &Self) -> f32 {
        c1.iter()
            .zip(c2.iter())
            .fold(T::default(), |acc, (&l, &r)| acc + (l - r).powi(2))
            .to_f32()
            .unwrap_or(f32::MAX)
    }
}

impl<T, const N: usize> Hamerly for [T; N]
where
    T: Float + FromPrimitive + AddAssign + DivAssign + Default,
    [T; N]: Default,
{
    fn compute_half_distances(centroids: &mut crate::HamerlyCentroids<Self>) {
        // Find each center's closest center
        for ((i, ci), half_dist) in centroids
            .centroids
            .iter()
            .enumerate()
            .zip(centroids.half_distances.iter_mut())
        {
            let mut diff;
            let mut min = f32::MAX;
            for (j, cj) in centroids.centroids.iter().enumerate() {
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

    fn get_closest_centroid_hamerly(
        buffer: &[Self],
        centroids: &crate::HamerlyCentroids<Self>,
        indices: &mut [crate::HamerlyPoint],
    ) {
        for (val, point) in buffer.iter().zip(indices.iter_mut()) {
            // Assign max of lower bound and half distance to z
            let z = centroids
                .half_distances
                .get(point.index as usize)
                .unwrap()
                .max(point.lower_bound);

            if point.upper_bound <= z {
                continue;
            }

            // Tighten upper bound
            point.upper_bound =
                Self::difference(val, centroids.centroids.get(point.index as usize).unwrap())
                    .sqrt();

            if point.upper_bound <= z {
                continue;
            }

            // Find the two closest centers to current point and their distances
            if centroids.centroids.len() < 2 {
                continue;
            }

            let mut min1 = Self::difference(val, centroids.centroids.get(0).unwrap());
            let mut min2 = f32::MAX;
            let mut c1 = 0;
            for j in 1..centroids.centroids.len() {
                let diff = Self::difference(val, centroids.centroids.get(j).unwrap());
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

    fn recalculate_centroids_hamerly(
        rng: &mut impl rand::Rng,
        buf: &[Self],
        centroids: &mut crate::HamerlyCentroids<Self>,
        points: &[crate::HamerlyPoint],
    ) {
        for ((idx, cent), delta) in centroids
            .centroids
            .iter_mut()
            .enumerate()
            .zip(centroids.deltas.iter_mut())
        {
            let mut acc = Self::default();
            let mut counter: u64 = 0;
            for (point, &color) in points.iter().zip(buf) {
                if point.index as usize == idx {
                    acc.iter_mut().zip(color.iter()).for_each(|(t, &c)| *t += c);
                    counter += 1;
                }
            }
            if counter != 0 {
                let counter_float = T::from_f64(counter as f64).unwrap();
                acc.iter_mut().for_each(|t| *t /= counter_float);
            } else {
                acc = Self::create_random(rng);
            }
            *delta = Self::difference(cent, &acc).sqrt();
            *cent = acc;
        }
    }

    fn update_bounds(
        centroids: &crate::HamerlyCentroids<Self>,
        points: &mut [crate::HamerlyPoint],
    ) {
        let mut delta_p = 0.0;
        for &c in centroids.deltas.iter() {
            if c > delta_p {
                delta_p = c;
            }
        }

        points.iter_mut().for_each(|point| {
            point.upper_bound += centroids.deltas.get(point.index as usize).unwrap();
            point.lower_bound -= delta_p;
        })
    }
}
