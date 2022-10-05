use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};
use new_york_utils::Matrix;
use vivalaakam_neat_rs::Organism;

pub fn get_result_steps(
    organism: &Organism,
    candles: &Vec<Candle>,
    balance: f64,
    profit_matrix: &Vec<f64>,
    stake_matrix: &Vec<f64>,
) -> CalculateResult {
    let target = candles.len() - 288;
    let org = organism.clone();

    let mut matrix = vec![];
    let mut responses = vec![CalculateCommand::None];
    for profit in profit_matrix {
        for stake in stake_matrix {
            matrix.push(vec![0.0, 1.0, *profit, *stake]);
            responses.push(CalculateCommand::BuyProfit( *profit, *stake));
        }
    }

    let empty = [0.0, 0.0, 0.0, 0.0].to_vec();

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

            let mut req = [history.to_vec(), empty.to_vec()].concat();

            for val in &matrix {
                req = [req, history.to_vec(), val.to_vec()].concat()
            }

            let mut matrix_req = Matrix::new(history.len() + empty.len(), responses.len());
            let _ = matrix_req.set_data(req);

            let result = org.activate_matrix(&matrix_req);

            let mut max = (f64::MIN, CalculateCommand::None);

            for val in 0..responses.len() {
                let result = result.get(0, val).unwrap_or_default();

                if result > max.0 {
                    max = (result, responses[val].clone());
                }
            }

            max.1
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    calculate_iter.into()
}
