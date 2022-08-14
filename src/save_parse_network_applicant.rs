use log::debug;
use serde_json::json;

use crate::{hash_md5, Parse};

pub async fn save_parse_network_applicant(
    parse: &Parse,
    from: u64,
    to: u64,
    days_count: u64,
    high_score: f64,
    lookback: usize,
    gain: f64,
    stake: f64,
    lag: usize,
    interval: usize,
) -> String {
    let applicant_id = hash_md5(format!("{}:{}", from, to));

    let created = parse
        .create(
            "NeatNetworkApplicants",
            json!({
                "objectId": applicant_id,
                "from": from,
                "to": to,
                "days": days_count,
                "highScore": high_score,
                "lookback": lookback,
                "gain": gain,
                "stake": stake,
                "lag": lag,
                "interval": interval,
                "touches": 0,
            }),
        )
        .await;
    debug!("{}", created);
    applicant_id
}
