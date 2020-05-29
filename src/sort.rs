use core::convert::TryFrom;
#[cfg(feature = "palette_color")]
use core::hash::Hash;
#[cfg(feature = "palette_color")]
use std::collections::HashMap;

#[cfg(feature = "palette_color")]
use palette::luma::Luma;
#[cfg(feature = "palette_color")]
use palette::white_point::WhitePoint;
#[cfg(feature = "palette_color")]
use palette::{Lab, Srgb};

use crate::Calculate;

/// A struct containing a centroid, its percentage within a buffer, and the
/// centroid's index.
#[derive(Clone, Debug, Default)]
pub struct CentroidData<C, U = u8>
where
    C: Calculate,
    U: TryFrom<usize>,
{
    /// A k-means centroid.
    pub centroid: C,
    /// The percentage a centroid appears in a buffer.
    pub percentage: f32,
    /// The centroid's index.
    pub index: U,
}

/// A trait for sorting indexed k-means colors.
#[cfg(feature = "palette_color")]
pub trait Sort: Sized + Calculate {
    /// Returns the centroid with the largest percentage.
    fn get_dominant_color<C, U>(data: &[CentroidData<C, U>]) -> Option<C>
    where
        C: Copy + Calculate,
        U: TryFrom<usize>;

    /// Sorts centroids by luminosity and calculates the percentage of each
    /// color in the buffer. Returns a `CentroidResult` sorted from darkest to
    /// lightest.
    fn sort_indexed_colors<C, U>(
        centroids: &Vec<Self>,
        indices: &[U],
    ) -> Vec<CentroidData<Self, U>>
    where
        C: Calculate,
        U: Copy + Eq + Hash + Into<usize> + TryFrom<usize>;
}

#[cfg(feature = "palette_color")]
impl<Wp: WhitePoint> Sort for Lab<Wp> {
    fn get_dominant_color<C, U>(data: &[CentroidData<C, U>]) -> Option<C>
    where
        C: Copy + Calculate,
        U: TryFrom<usize>,
    {
        let res = data
            .iter()
            .max_by(|a, b| (a.percentage).partial_cmp(&b.percentage).unwrap())
            .unwrap();

        Some(res.centroid)
    }

    fn sort_indexed_colors<C, U>(centroids: &Vec<Self>, indices: &[U]) -> Vec<CentroidData<Self, U>>
    where
        C: Calculate,
        U: Copy + Eq + Hash + Into<usize> + TryFrom<usize>,
    {
        // Count occurences of each color - "histogram"
        let mut map: HashMap<U, u64> = HashMap::new();
        for (i, _) in centroids.iter().enumerate() {
            map.insert(U::try_from(i).ok().unwrap(), 0);
        }
        for i in indices {
            let count = map.entry(*i).or_insert(0);
            *count += 1;
        }

        let len = indices.len();
        let mut colors: Vec<(U, f32)> = Vec::with_capacity(centroids.len());
        for (i, _) in centroids.iter().enumerate() {
            let count = map.get(&U::try_from(i).ok().unwrap());
            match count {
                Some(x) => colors.push((U::try_from(i).ok().unwrap(), (*x as f32) / (len as f32))),
                None => continue,
            }
        }

        // Sort by increasing luminosity
        let mut lab: Vec<(U, Lab<Wp>)> = centroids
            .iter()
            .enumerate()
            .map(|(i, x)| (U::try_from(i).ok().unwrap(), *x))
            .collect();
        lab.sort_unstable_by(|a, b| (a.1.l).partial_cmp(&b.1.l).unwrap());

        // Pack the colors and their percentages into the return vector.
        // Get the lab's key from the map, if the key value is greater than one
        // attempt to find the index of it in the colors vec. Push that to the
        // output vec tuple if successful.
        lab.iter()
            .filter_map(|x| map.get_key_value(&x.0))
            .filter(|x| *x.1 > 0)
            .filter_map(|x| match colors.get(usize::try_from(*x.0).unwrap()) {
                Some(x) => match colors.iter().position(|a| a.0 == x.0) {
                    Some(y) => Some(CentroidData {
                        centroid: *(centroids
                            .get(usize::try_from(colors.get(y).unwrap().0).unwrap())
                            .unwrap()),
                        percentage: colors.get(y).unwrap().1,
                        index: U::try_from(y).ok().unwrap(),
                    }),
                    None => None,
                },
                None => None,
            })
            .collect()
    }
}

#[cfg(feature = "palette_color")]
impl Sort for Srgb {
    fn get_dominant_color<C, U>(data: &[CentroidData<C, U>]) -> Option<C>
    where
        C: Copy + Calculate,
        U: TryFrom<usize>,
    {
        let res = data
            .iter()
            .max_by(|a, b| (a.percentage).partial_cmp(&b.percentage).unwrap())
            .unwrap();

        Some(res.centroid)
    }

    fn sort_indexed_colors<C, U>(centroids: &Vec<Self>, indices: &[U]) -> Vec<CentroidData<Self, U>>
    where
        C: Calculate,
        U: Copy + Eq + Hash + Into<usize> + TryFrom<usize>,
    {
        // Count occurences of each color - "histogram"
        let mut map: HashMap<U, u64> = HashMap::new();
        for (i, _) in centroids.iter().enumerate() {
            map.insert(U::try_from(i).ok().unwrap(), 0);
        }
        for i in indices {
            let count = map.entry(*i).or_insert(0);
            *count += 1;
        }

        let len = indices.len();
        let mut colors: Vec<(U, f32)> = Vec::with_capacity(centroids.len());
        for (i, _) in centroids.iter().enumerate() {
            let count = map.get(&U::try_from(i).ok().unwrap());
            match count {
                Some(x) => colors.push((U::try_from(i).ok().unwrap(), (*x as f32) / (len as f32))),
                None => continue,
            }
        }

        // Sort by increasing luminosity
        let mut lab: Vec<(U, Luma)> = centroids
            .iter()
            .enumerate()
            .map(|(i, x)| (U::try_from(i).ok().unwrap(), x.into_format().into()))
            .collect();
        lab.sort_unstable_by(|a, b| (a.1.luma).partial_cmp(&b.1.luma).unwrap());

        // Pack the colors and their percentages into the return vector
        lab.iter()
            .filter_map(|x| map.get_key_value(&x.0))
            .filter(|x| *x.1 > 0)
            .filter_map(|x| match colors.get(usize::try_from(*x.0).unwrap()) {
                Some(x) => match colors.iter().position(|a| a.0 == x.0) {
                    Some(y) => Some(CentroidData {
                        centroid: *(centroids
                            .get(usize::try_from(colors.get(y).unwrap().0).unwrap())
                            .unwrap()),
                        percentage: colors.get(y).unwrap().1,
                        index: U::try_from(y).ok().unwrap(),
                    }),
                    None => None,
                },
                None => None,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::{CentroidData, Sort};
    #[cfg(feature = "palette_color")]
    use palette::Srgb;

    #[cfg(feature = "palette_color")]
    #[test]
    fn dominant_color() {
        let res = vec![
            CentroidData::<Srgb> {
                centroid: Srgb::new(0.0, 0.0, 0.0),
                percentage: 0.5,
                index: 0,
            },
            CentroidData::<Srgb> {
                centroid: Srgb::new(0.5, 0.5, 0.5),
                percentage: 0.80,
                index: 0,
            },
            CentroidData::<Srgb> {
                centroid: Srgb::new(1.0, 1.0, 1.0),
                percentage: 0.15,
                index: 0,
            },
        ];
        assert_eq!(
            Srgb::get_dominant_color(&res).unwrap(),
            Srgb::new(0.5, 0.5, 0.5)
        );
    }
}
