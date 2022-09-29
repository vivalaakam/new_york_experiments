use new_york_calculate_core::Candle;
use vivalaakam_neat_rs::Organism;

use crate::NeatNetworkApplicants;

pub fn get_score_fitness(
    organism: &mut Organism,
    candles: &Vec<Candle>,
    applicant: &NeatNetworkApplicants,
    epoch: usize,
) {
    let result = applicant.get_result(organism, candles, epoch);

    organism.set_fitness(result.wallet * result.drawdown);
}
