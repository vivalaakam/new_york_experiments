use crate::Buffer;
use new_york_calculate_core::{CalculateCommand, CalculateIter, CalculateResult, Candle};
use std::sync::Mutex;
use vivalaakam_neat_rs::Organism;

pub fn get_result(
    organism: &Organism,
    candles: &Vec<Candle>,
    gain: f64,
    lag: usize,
    stake: f64,
) -> CalculateResult {
    let target = candles.len() - 288;
    let org = organism.clone();
    let buffer = Mutex::new(Buffer::new(lag));
    let stake = stake;
    let gain = gain / 100f64 + 1f64;
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
            let result = org.activate(candle.history.to_vec());
            let mut buffer = buffer.lock().unwrap();
            if result[1] > result[0] {
                buffer.push(1.0);
                if buffer.avg() > 0.5 {
                    CalculateCommand::BuyProfit(gain, stake, 1.0)
                } else {
                    CalculateCommand::None(0.0)
                }
            } else {
                buffer.push(0.0);
                CalculateCommand::None(0.0)
            }
        }),
    );

    let mut cont = Ok(());

    while cont.is_ok() {
        cont = calculate_iter.next();
    }

    calculate_iter.into()
}
