use std::collections::HashMap;

use palette::luma::Luma;
use palette::{Lab, Srgb};

/// Sorts the Lab centroids by luminosity and calculates the percentage of each
/// color in the buffer. Returns a vector of tuples sorted from darkest to
/// lightest holding a centroid, its percentage, and the index of the centroid.
pub fn sort_indexed_colors_lab(centroids: &Vec<Lab>, indices: &[u8]) -> Vec<(Lab, f32, u8)> {
    // Count occurences of each color - "histogram"
    let mut map: HashMap<u8, u32> = HashMap::new();
    for (i, _) in centroids.iter().enumerate() {
        map.insert(i as u8, 0);
    }
    for i in indices {
        let count = map.entry(*i).or_insert(0);
        *count += 1;
    }

    let len = indices.len();
    let mut colors: Vec<(u8, f32)> = Vec::with_capacity(centroids.len());
    for (i, _) in centroids.iter().enumerate() {
        let count = map.get(&(i as u8));
        match count {
            Some(x) => colors.push((i as u8, (*x as f32) / (len as f32))),
            None => continue,
        }
    }

    // Sort by increasing luminosity
    let mut lab: Vec<(u8, Lab)> = centroids
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
                Some(y) => Some((
                    *(centroids.get(colors.get(y).unwrap().0 as usize).unwrap()),
                    colors.get(y).unwrap().1,
                    y as u8,
                )),
                None => None,
            },
            None => None,
        })
        .collect()
}

/// Sorts the RGB centroids by luminosity and calculates the percentage of each
/// color in the buffer. Returns a vector of tuples sorted from darkest to
/// lightest holding a centroid, its percentage, and the index of the centroid.
pub fn sort_indexed_colors_rgb(centroids: &[Srgb], indices: &[u8]) -> Vec<(Srgb, f32, u8)> {
    // Count occurences of each color - "histogram"
    let mut map: HashMap<u8, u32> = HashMap::new();
    for (i, _) in centroids.iter().enumerate() {
        map.insert(i as u8, 0);
    }
    for i in indices {
        let count = map.entry(*i).or_insert(0);
        *count += 1;
    }

    let len = indices.len();
    let mut colors: Vec<(u8, f32)> = Vec::with_capacity(centroids.len());
    for (i, _) in centroids.iter().enumerate() {
        let count = map.get(&(i as u8)).unwrap();
        colors.push((i as u8, *count as f32 / len as f32));
    }

    // Sort by increasing luminosity
    let mut lab: Vec<(u8, Luma)> = centroids
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
                Some(y) => Some((
                    *(centroids.get(colors.get(y).unwrap().0 as usize).unwrap()),
                    colors.get(y).unwrap().1,
                    y as u8,
                )),
                None => None,
            },
            None => None,
        })
        .collect()
}
