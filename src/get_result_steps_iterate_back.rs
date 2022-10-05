use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};
use vivalaakam_neat_rs::Organism;

pub fn get_result_steps_iterate_back(
    organism: &Organism,
    candles: &Vec<Candle>,
    balance: f64,
    profit_matrix: &Vec<f64>,
    stake_matrix: &Vec<f64>,
    epoch: usize,
) -> CalculateResult {
    let target = candles.len() - 288;
    let org = organism.clone();

    let mut matrix = vec![];

    for profit in profit_matrix {
        for stake in stake_matrix {
            matrix.push(vec![0.0, 1.0, *profit, *stake])
        }
    }

    let empty = [0.0, 0.0, 0.0, 0.0].to_vec();

    let dest = (candles.len() / 50).max(epoch).min(candles.len());
    let candles = &candles[candles.len() - dest..candles.len()].to_vec();

    let mut calculate_iter = CalculateIter::new(
        &candles,
        balance,
        0.5,
        1f64,
        0.0001f64,
        Box::new(move |candle, ind, stats| {
            if ind >= target {
                return CalculateCommand::Unknown;
            }

            let history = [
                candle.history.to_vec(),
                [
                    stats.balance,
                    stats.orders as f64,
                    stats.count,
                    stats.expected,
                    stats.real,
                ]
                    .to_vec(),
            ]
                .concat();

            let result = org.activate([history.to_vec(), empty.to_vec()].concat());

            let mut max = (result[0], None);

            for val in &matrix {
                let result = org.activate([history.to_vec(), val.to_vec()].concat());

                if result[0] > max.0 {
                    max = (result[0], Some(val))
                }
            }

            match max.1 {
                None => CalculateCommand::None,
                Some(val) => CalculateCommand::BuyProfit(val[2], val[3]),
            }
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    calculate_iter.into()
}
