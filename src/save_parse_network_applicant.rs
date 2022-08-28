use log::debug;

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
    applicant_type: NeatNetworkApplicantType,
    ticker: String,
) -> String {
    let applicant_id = hash_md5(format!("{ticker}:{from}:{to}:{inputs}:{outputs}"));

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
        applicant_type,
        ticker,
    };

    let result = parse.create("NeatNetworkApplicants", value).await;
    debug!("NeatNetworkApplicants: {result}");
    applicant_id
}
