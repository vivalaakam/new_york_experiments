use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};

pub fn get_high_fitness(candles: &Vec<Candle>, stake: f64, profit_matrix: &Vec<f64>) -> f64 {
    let target = candles.len() - 288;
    let profit_matrix = profit_matrix[0..5].to_vec();
    let mut calculate_iter = CalculateIter::new(
        &candles,
        3000.0,
        0.5,
        1f64,
        0.0001f64,
        Box::new(move |candle, ind, _stats| {
            if ind >= target {
                return CalculateCommand::Unknown;
            }

            for j in 0..profit_matrix.len() {
                let ind = profit_matrix.len() - j - 1;
                if (candle.max_profit[ind] / 100f64 + 1f64) > profit_matrix[ind] {
                    return CalculateCommand::BuyProfit(profit_matrix[ind], stake);
                }
            }

            CalculateCommand::None
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    let result: CalculateResult = calculate_iter.into();

    result.wallet
}
