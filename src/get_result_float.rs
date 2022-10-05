use std::sync::Mutex;

use new_york_calculate_core::utils::range;
use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};
use vivalaakam_neat_rs::Organism;

use crate::Buffer;

pub fn get_result_float(
    organism: &Organism,
    candles: &Vec<Candle>,
    balance: f64,
    lag: usize,
    stake: f64,
) -> CalculateResult {
    let target = candles.len() - 288;
    let org = organism.clone();
    let buffer = Mutex::new(Buffer::new(lag));
    let mut calculate_iter = CalculateIter::new(
        &candles,
        balance,
        0.5,
        1f64,
        0.0001f64,
        Box::new(move |candle, ind, _stats| {
            if ind >= target {
                return CalculateCommand::Unknown;
            }
            let result = org.activate(candle.history.to_vec());
            let mut buffer = buffer.lock().unwrap();

            if result[0] >= 0.25 {
                let interpolated = range(0.25, 1.0, 1.005, 1.025, result[0]);

                buffer.push(interpolated);
                if buffer.avg() >= 1.005 {
                    CalculateCommand::BuyProfit(buffer.avg(), stake)
                } else {
                    CalculateCommand::None
                }
            } else {
                buffer.push(1.0);
                CalculateCommand::None
            }
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    calculate_iter.into()
}
