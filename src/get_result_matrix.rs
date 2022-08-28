use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};
use vivalaakam_neat_rs::{argmax, Organism};

pub fn get_result_matrix(
    organism: &Organism,
    candles: &Vec<Candle>,
    stake: f64,
    profit_matrix: &Vec<f64>,
) -> CalculateResult {
    let target = candles.len() - 288;
    let org = organism.clone();
    let profit_matrix = profit_matrix.to_vec();
    let mut calculate_iter = CalculateIter::new(
        &candles,
        3000.0,
        0.5,
        5,
        1f64,
        0.0001f64,
        Box::new(move |candle, ind| {
            if ind >= target {
                return CalculateCommand::Unknown;
            }
            let result = argmax(org.activate(candle.history.to_vec()));

            if result > 0 {
                let gain = profit_matrix[result - 1];
                if gain > 1.0 {
                    return CalculateCommand::BuyProfit(gain, stake, 1.0);
                }
            }

            CalculateCommand::None(0.0)
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    calculate_iter.into()
}
