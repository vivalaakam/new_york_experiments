use new_york_calculate_core::Candle;
use vivalaakam_neat_rs::Organism;

use crate::get_result;

pub fn get_score_fitness(
    organism: &mut Organism,
    candles: &Vec<Candle>,
    gain: f64,
    lag: usize,
    stake: f64,
) {
    let result = get_result(&organism, candles, gain, lag, stake);

    organism.set_fitness(result.wallet * result.drawdown);
}
