#[cfg(feature = "palette_color")]
use palette::{Lab, Srgb};

use rand::Rng;

use crate::kmeans::{Calculate, Hamerly, HamerlyCentroids, HamerlyPoint};

#[cfg(feature = "palette_color")]
impl<Wp: palette::white_point::WhitePoint> Calculate for Lab<Wp> {
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
            rng.gen_range(0.0..=100.0),
            rng.gen_range(-128.0..=127.0),
            rng.gen_range(-128.0..=127.0),
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

#[cfg(feature = "palette_color")]
impl<Wp: palette::white_point::WhitePoint> Hamerly for Lab<Wp> {
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
                diff = Self::difference(&ci, &cj);
                if diff < min {
                    min = diff;
                }
            }
            *half_dist = min.sqrt() * 0.5;
        }
    }

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

            if c1 as u8 != point.index {
                point.index = c1 as u8;
                point.upper_bound = min1.sqrt();
            }
            point.lower_bound = min2.sqrt();
        }
    }

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
impl Hamerly for Srgb {
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
                diff = Self::difference(&ci, &cj);
                if diff < min {
                    min = diff;
                }
            }
            *half_dist = min.sqrt() * 0.5;
        }
    }

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

            if c1 as u8 != point.index {
                point.index = c1 as u8;
                point.upper_bound = min1.sqrt();
            }
            point.lower_bound = min2.sqrt();
        }
    }

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
impl<Wp> MapColor for Lab<Wp>
where
    Wp: palette::white_point::WhitePoint,
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
impl<Wp> MapColor for palette::Laba<Wp>
where
    Wp: palette::white_point::WhitePoint,
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
    C: palette::Component,
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
impl<C> MapColor for palette::Srgba<C>
where
    C: palette::Component,
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
