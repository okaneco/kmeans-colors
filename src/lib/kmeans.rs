use palette::{Lab, Pixel, Srgb};
use rand::{Rng, SeedableRng};

/// Result of k-means operation in Lab space.
pub struct KmeansLab {
    /// Sum of squares distance metric for centroids compared to old centroids.
    pub score: f32,

    /// Colors determined to be centroids of input buffer.
    pub centroids: Vec<Lab>,

    /// Buffer of pixels indexed to centroids.
    pub indices: Vec<u8>,
}

impl KmeansLab {
    pub fn new() -> Self {
        KmeansLab {
            score: core::f32::MAX,
            centroids: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Result of k-means operation in Rgb space.
pub struct KmeansRgb {
    /// Sum of squares distance metric for centroids compared to old centroids.
    pub score: f32,

    /// Colors determined to be centroids of input buffer.
    pub centroids: Vec<Srgb>,

    /// Buffer of pixels indexed to centroids.
    pub indices: Vec<u8>,
}

impl KmeansRgb {
    pub fn new() -> Self {
        KmeansRgb {
            score: core::f32::MAX,
            centroids: Vec::new(),
            indices: Vec::new(),
        }
    }
}

/// Find k-means colors of an image in Lab space.
pub fn get_kmeans_lab(
    k: u8,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    lab: &[Lab],
    seed: u64,
) -> KmeansLab {
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
    let mut centroids: Vec<Lab> = Vec::with_capacity(k as usize);
    (0..k).for_each(|_| centroids.push(create_random_lab(&mut rng)));

    // Initialize indexed color buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centroids = centroids.clone();
    let mut indices: Vec<u8> = Vec::with_capacity(lab.len());

    // Main loop: find nearest centroids and recalculate means until convergence
    loop {
        get_closest_centroid_lab(&lab, &centroids, &mut indices);
        recalculate_centroids_lab(&mut rng, &mut centroids, &lab, &indices);

        score = check_loop_lab(&centroids, &old_centroids);
        if verbose {
            println!("Score: {}", score);
        }

        // Verify that either the maximum iteration count has been met or the
        // centroids haven't moved beyond a certain threshold compared to
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

    KmeansLab {
        score,
        centroids,
        indices,
    }
}

/// Find a pixel's nearest centroid color in Lab, index the pixel with that
/// centroid.
pub fn get_closest_centroid_lab(lab: &[Lab], centroids: &[Lab], indices: &mut Vec<u8>) {
    for color in lab.iter() {
        let mut index = 0;
        let mut diff;
        let mut min = core::f32::MAX;
        for (idx, cent) in centroids.iter().enumerate() {
            diff = diff_colors_lab(color, cent);
            if diff < min {
                min = diff;
                index = idx;
            }
        }
        indices.push(index as u8);
    }
}

/// Find the new centroid locations based on the average of the colors that
/// correspond to the centroid in Lab. If no colors correspond, the centroid is
/// re-initialized with a random color.
pub fn recalculate_centroids_lab(
    mut rng: &mut impl Rng,
    centroids: &mut Vec<Lab>,
    lab: &[Lab],
    indices: &[u8],
) {
    for (idx, cent) in centroids.iter_mut().enumerate() {
        let mut l = 0.0;
        let mut a = 0.0;
        let mut b = 0.0;
        let mut counter: u32 = 0;
        for (jdx, color) in indices.iter().zip(lab) {
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
            *cent = create_random_lab(&mut rng);
        }
    }
}

/// Calculate the distance metric for convergence comparison.
pub fn check_loop_lab(centroids: &[Lab], old_centroids: &[Lab]) -> f32 {
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

/// Generate random Lab color.
pub fn create_random_lab(rng: &mut impl Rng) -> Lab {
    Lab::new(
        rng.gen_range(0.0, 100.0),
        rng.gen_range(-128.0, 127.0),
        rng.gen_range(-128.0, 127.0),
    )
}

/// Calculate the geometric distance between two Lab colors, the square root is
/// omitted.
#[rustfmt::skip]
pub fn diff_colors_lab(c1: &Lab, c2: &Lab) -> f32 {
    (c1.l - c2.l) * (c1.l - c2.l) +
    (c1.a - c2.a) * (c1.a - c2.a) +
    (c1.b - c2.b) * (c1.b - c2.b)
}

/// Map pixel indices to centroid colors for output from Lab to Srgb.
pub fn map_indices_to_colors_lab(centroids: &[Lab], indices: &[u8]) -> Vec<u8> {
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

/// Find k-means colors of an image in Rgb space.
pub fn get_kmeans_rgb(
    k: u8,
    max_iter: usize,
    converge: f32,
    verbose: bool,
    rgb: &[Srgb],
    seed: u64,
) -> KmeansRgb {
    // Initialize the random centroids
    let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);

    let mut centroids: Vec<Srgb> = Vec::with_capacity(k as usize);
    (0..k).for_each(|_| centroids.push(create_random_rgb(&mut rng)));

    // Initialize indexed color buffer and convergence variables
    let mut iterations = 0;
    let mut score;
    let mut old_centroids = centroids.clone();
    let mut indices: Vec<u8> = Vec::with_capacity(rgb.len());

    // Main loop: find nearest centroids and recalculate means until convergence
    loop {
        get_closest_centroid_rgb(&rgb, &centroids, &mut indices);
        recalculate_centroids_rgb(&mut rng, &mut centroids, &rgb, &indices);

        score = check_loop_rgb(&centroids, &old_centroids);
        if verbose {
            println!("Score: {}", score);
        }

        // Verify that either the maximum iteration count has been met or the
        // centroids haven't moved beyond a certain threshold compared to
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

    KmeansRgb {
        score,
        centroids,
        indices,
    }
}

/// Find a pixel's nearest centroid color in Rgb, index the pixel with that
/// centroid.
pub fn get_closest_centroid_rgb(rgb: &[Srgb], centroids: &[Srgb], indices: &mut Vec<u8>) {
    for color in rgb.iter() {
        let mut index = 0;
        let mut diff;
        let mut min = core::f32::MAX;
        for (idx, cent) in centroids.iter().enumerate() {
            diff = diff_colors_rgb(color, cent);
            if diff < min {
                min = diff;
                index = idx;
            }
        }
        indices.push(index as u8);
    }
}

/// Find the new centroid locations based on the average of the colors that
/// correspond to the centroid in Rgb. If no colors correspond, the centroid is
/// re-initialized with a random color.
pub fn recalculate_centroids_rgb(
    mut rng: &mut impl Rng,
    centroids: &mut [Srgb],
    rgb: &[Srgb],
    indices: &[u8],
) {
    for (idx, cent) in centroids.iter_mut().enumerate() {
        let mut red = 0.0;
        let mut green = 0.0;
        let mut blue = 0.0;
        let mut counter: u32 = 0;
        for (jdx, color) in indices.iter().zip(rgb) {
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
            *cent = create_random_rgb(&mut rng);
        }
    }
}

/// Calculate the distance metric for convergence comparison.
pub fn check_loop_rgb(centroids: &[Srgb], old_centroids: &[Srgb]) -> f32 {
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

/// Generate random Rgb color.
pub fn create_random_rgb(rng: &mut impl Rng) -> Srgb {
    Srgb::new(rng.gen(), rng.gen(), rng.gen())
}

/// Calculate the geometric distance between two Rgb colors, the square root is
/// omitted.
pub fn diff_colors_rgb(c1: &Srgb, c2: &Srgb) -> f32 {
    (c1.red - c2.red) * (c1.red - c2.red)
        + (c1.green - c2.green) * (c1.green - c2.green)
        + (c1.blue - c2.blue) * (c1.blue - c2.blue)
}

/// Map pixel indices to centroid colors for output in Srgb.
pub fn map_indices_to_colors_rgb(centroids: &[Srgb], indices: &[u8]) -> Vec<u8> {
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
