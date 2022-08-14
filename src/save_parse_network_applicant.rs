use log::debug;

use crate::{hash_md5, NeatNetworkApplicants, Parse};

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
) -> String {
    let applicant_id = hash_md5(format!("{}:{}:{}:{}", from, to, inputs, outputs));

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
    };

    let result = parse.create("NeatNetworkApplicants", value).await;
    debug!("NeatNetworkApplicants: {result}");
    applicant_id
}
