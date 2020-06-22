use crate::sort::{CentroidData, Sort};

#[cfg(feature = "palette_color")]
impl<Wp: palette::white_point::WhitePoint> Sort for palette::Lab<Wp> {
    fn get_dominant_color(data: &[CentroidData<Self>]) -> Option<Self> {
        let res = data
            .iter()
            .max_by(|a, b| (a.percentage).partial_cmp(&b.percentage).unwrap())
            .unwrap();

        Some(res.centroid)
    }

    fn sort_indexed_colors(centroids: &Vec<Self>, indices: &[u8]) -> Vec<CentroidData<Self>> {
        // Count occurences of each color - "histogram"
        let mut map: std::collections::HashMap<u8, u64> = std::collections::HashMap::new();
        for (i, _) in centroids.iter().enumerate() {
            map.insert(i as u8, 0);
        }
        for i in indices {
            let count = map.entry(*i).or_insert(0);
            *count += 1;
        }

        let len = indices.len();
        assert!(len > 0);
        let mut colors: Vec<(u8, f32)> = Vec::with_capacity(centroids.len());
        for (i, _) in centroids.iter().enumerate() {
            let count = map.get(&(i as u8));
            match count {
                Some(x) => colors.push((i as u8, (*x as f32) / (len as f32))),
                None => continue,
            }
        }

        // Sort by increasing luminosity
        let mut lab: Vec<(u8, Self)> = centroids
            .iter()
            .enumerate()
            .map(|(i, x)| (i as u8, *x))
            .collect();
        lab.sort_unstable_by(|a, b| (a.1.l).partial_cmp(&b.1.l).unwrap());

        // Pack the colors and their percentages into the return vector.
        // Get the lab's key from the map, if the key value is greater than one
        // attempt to find the index of it in the colors vec. Push that to the
        // output vec tuple if successful.
        lab.iter()
            .filter_map(|x| map.get_key_value(&x.0))
            .filter(|x| *x.1 > 0)
            .filter_map(|x| match colors.get(*x.0 as usize) {
                Some(x) => match colors.iter().position(|a| a.0 == x.0 as u8) {
                    Some(y) => Some(CentroidData {
                        centroid: *(centroids.get(colors.get(y).unwrap().0 as usize).unwrap()),
                        percentage: colors.get(y).unwrap().1,
                        index: y as u8,
                    }),
                    None => None,
                },
                None => None,
            })
            .collect()
    }
}

#[cfg(feature = "palette_color")]
impl Sort for palette::Srgb {
    fn get_dominant_color(data: &[CentroidData<Self>]) -> Option<Self> {
        let res = data
            .iter()
            .max_by(|a, b| (a.percentage).partial_cmp(&b.percentage).unwrap())
            .unwrap();

        Some(res.centroid)
    }

    fn sort_indexed_colors(centroids: &Vec<Self>, indices: &[u8]) -> Vec<CentroidData<Self>> {
        // Count occurences of each color - "histogram"
        let mut map: std::collections::HashMap<u8, u64> = std::collections::HashMap::new();
        for (i, _) in centroids.iter().enumerate() {
            map.insert(i as u8, 0);
        }
        for i in indices {
            let count = map.entry(*i).or_insert(0);
            *count += 1;
        }

        let len = indices.len();
        assert!(len > 0);
        let mut colors: Vec<(u8, f32)> = Vec::with_capacity(centroids.len());
        for (i, _) in centroids.iter().enumerate() {
            let count = map.get(&(i as u8));
            match count {
                Some(x) => colors.push((i as u8, (*x as f32) / (len as f32))),
                None => continue,
            }
        }

        // Sort by increasing luminosity
        let mut lab: Vec<(u8, palette::luma::Luma)> = centroids
            .iter()
            .enumerate()
            .map(|(i, x)| (i as u8, x.into_format().into()))
            .collect();
        lab.sort_unstable_by(|a, b| (a.1.luma).partial_cmp(&b.1.luma).unwrap());

        // Pack the colors and their percentages into the return vector
        lab.iter()
            .filter_map(|x| map.get_key_value(&x.0))
            .filter(|x| *x.1 > 0)
            .filter_map(|x| match colors.get(*x.0 as usize) {
                Some(x) => match colors.iter().position(|a| a.0 == x.0 as u8) {
                    Some(y) => Some(CentroidData {
                        centroid: *(centroids.get(colors.get(y).unwrap().0 as usize).unwrap()),
                        percentage: colors.get(y).unwrap().1,
                        index: y as u8,
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
