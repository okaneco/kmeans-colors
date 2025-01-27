use rand::distr::weighted::WeightedIndex;
use rand::distr::Distribution;
use rand::Rng;

/// k-means++ centroid initialization.
///
/// # Panics
///
/// Panics if buffer is empty.
///
/// # Reference
///
/// Based on Section 2.2 from `k-means++: The Advantages of Careful Seeding` by
/// Arthur and Vassilvitskii (2007).
pub fn init_plus_plus<C: crate::Calculate + Clone>(
    k: usize,
    mut rng: &mut impl Rng,
    buf: &[C],
    centroids: &mut Vec<C>,
) {
    if k == 0 {
        return;
    }
    let len = buf.len();
    assert!(len > 0);

    let mut weights: Vec<f32> = (0..len).map(|_| 0.0).collect();

    // Choose first centroid at random, uniform sampling from input buffer
    centroids.push(buf.get(rng.random_range(0..len)).unwrap().to_owned());

    // Pick a new centroid with weighted probability of `D(x)^2 / sum(D(x)^2)`,
    // where `D(x)^2` is the distance to the closest centroid
    for _ in 1..k {
        // Calculate the distances to nearest centers, accumulate a sum
        let mut sum = 0.0;
        for (b, dist) in buf.iter().zip(weights.iter_mut()) {
            let mut diff;
            let mut min = f32::MAX;
            for cent in centroids.iter() {
                diff = C::difference(b, cent);
                if diff < min {
                    min = diff;
                }
            }
            *dist = min;
            sum += min;
        }

        // If centroids match all colors, return early
        if !sum.is_normal() {
            return;
        }

        // Divide distances by sum to find D^2 weighting for distribution
        weights.iter_mut().for_each(|x| *x /= sum);

        // Choose next centroid based on weights
        let sampler = WeightedIndex::new(&weights).unwrap();
        centroids.push(buf.get(sampler.sample(&mut rng)).unwrap().to_owned());
    }
}
