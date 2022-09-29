use log::debug;
use new_york_calculate_core::Indicators;

use crate::neat_network_applicant_type::NeatNetworkApplicantType;
use crate::neat_network_applicants::NeatNetworkApplicants;
use crate::{hash_md5, Parse};

pub async fn save_parse_network_applicant(
    parse: &Parse,
    from: u64,
    to: u64,
    days: u64,
    high_score: f64,
    lookback: usize,
    gain: f64,
    stake: f64,
    lag: usize,
    interval: usize,
    inputs: usize,
    outputs: usize,
    profit_matrix: Vec<f64>,
    gain_matrix: Vec<f64>,
    indicators: Vec<Indicators>,
    applicant_type: NeatNetworkApplicantType,
    ticker: String,
    balance: f64,
) -> String {
    let applicant_id = hash_md5(format!("{ticker}:{from}:{to}:{inputs}:{outputs}:{applicant_type}"));

    let value = NeatNetworkApplicants {
        object_id: applicant_id.to_string(),
        from,
        to,
        days,
        high_score,
        lookback,
        gain,
        stake,
        lag,
        interval,
        touches: 0,
        inputs,
        outputs,
        profit_matrix,
        gain_matrix,
        indicators,
        applicant_type,
        ticker,
        balance
    };

    let result = parse.create("NeatNetworkApplicants", value).await;
    debug!("NeatNetworkApplicants: {result}");
    applicant_id
}
