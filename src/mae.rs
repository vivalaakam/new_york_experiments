pub fn mae(expected: &Vec<f64>, real: &Vec<f64>) -> f64 {
    let mut sum = 0.0;

    for i in 0..expected.len() {
        sum += (expected[i] - real[i]).powi(2);
    }

    sum / expected.len() as f64
}
