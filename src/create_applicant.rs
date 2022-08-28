use new_york_calculate_core::get_candles_with_cache;

use crate::get_keys_for_interval::get_keys_for_interval;
use crate::neat_network_applicant_type::NeatNetworkApplicantType;
use crate::{get_high_fitness, get_now, save_parse_network_applicant, Parse};

pub async fn create_applicant(
    parse: &Parse,
    days: u64,
    from: Option<u64>,
    lag: usize,
    interval: usize,
    look_back: usize,
    gain: f64,
    stake: f64,
    inputs: usize,
    outputs: usize,
    profit_matrix: Option<Vec<f64>>,
    applicant_type: NeatNetworkApplicantType,
    ticker: String,
) -> String {
    let mut candles = vec![];
    let next = (get_now() / 86400000) as u64;
    let from = match from {
        Some(v) => v,
        None => (next - days - 2) * 86400,
    };

    let to = from + (days - 1) * 86400;
    let keys = get_keys_for_interval(from, to);

    for key in keys {
        let new_candles =
            get_candles_with_cache(ticker.to_string(), interval, key, look_back, None).await;
        candles = [candles, new_candles].concat()
    }

    let profit_matrix = match profit_matrix {
        None => vec![1.005, 1.0075, 1.01, 1.0125, 1.025],
        Some(matrix) => matrix.to_vec(),
    };

    let high_score = get_high_fitness(&candles, stake, &profit_matrix);

    let applicant_id = save_parse_network_applicant(
        &parse,
        from,
        to,
        days,
        high_score,
        look_back,
        gain,
        stake,
        lag,
        interval,
        inputs,
        outputs,
        profit_matrix,
        applicant_type,
        ticker,
    )
    .await;

    applicant_id
}
