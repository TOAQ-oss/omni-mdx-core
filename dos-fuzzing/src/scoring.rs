use itertools::Itertools;

/// The number of samples generated per pattern (e.g., sizes 100, 200, 300...).
pub const SAMPLE_SIZE: usize = 5;

/// Threshold for the standard deviation of slopes.
/// If the deviation exceeds this, the parser's time complexity is non-linear (O(N^2) or worse).
const ACCEPTANCE_STDDEV: f64 = 300.0;

/// Mathematically proves if the execution time grows super-linearly.
pub fn is_superlinear(time_samples: &[(f64, f64)]) -> (f64, bool) {
    let mut slopes = Vec::with_capacity(SAMPLE_SIZE * (SAMPLE_SIZE - 1) / 2);

    // Calculate the slope (dy/dx) between every possible pair of data points
    for (p1, p2) in (0..time_samples.len()).tuple_combinations() {
        let (x1, y1) = time_samples[p1];
        let (x2, y2) = time_samples[p2];
        let dx = x2 - x1;
        let dy = y2 - y1;
        if dx > 0.0 {
            slopes.push(dy / dx);
        }
    }

    if slopes.is_empty() {
        return (0.0, false);
    }

    // Calculate Standard Deviation
    let mean: f64 = slopes.iter().sum::<f64>() / slopes.len() as f64;
    let variance: f64 = slopes
        .iter()
        .map(|&slope| (slope - mean).powi(2))
        .sum::<f64>()
        / slopes.len() as f64;
    let stddev = variance.sqrt();

    (stddev, stddev > ACCEPTANCE_STDDEV)
}
