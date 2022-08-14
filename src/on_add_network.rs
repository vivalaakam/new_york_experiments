use std::collections::HashMap;

use new_york_calculate_core::get_candles_with_cache;
use serde_json::json;
use vivalaakam_neat_rs::Organism;

use crate::get_keys_for_interval::get_keys_for_interval;
use crate::{
    get_result, save_parse_network_result, NeatNetworkApplicants, NeatNetworkResults, NeatNetworks,
    Parse,
};

pub async fn on_add_network(parse: &Parse, network_id: String) {
    let parent = parse
        .get::<NeatNetworks, _, _>("NeatNetworks", network_id)
        .await;

    if parent.is_none() {
        return;
    }

    let parent = parent.unwrap();

    let organism = Organism::new(parent.network.into());

    let results = parse
        .query::<NeatNetworkResults, _, _>("NeatNetworkResults", json!({}), None, None, None)
        .await;

    let mut results_cache: HashMap<String, f64> = HashMap::new();

    for result in results.results {
        let value = *results_cache.get(&result.applicant_id).unwrap_or(&0f64);
        results_cache.insert(
            result.applicant_id,
            value.max(result.wallet * result.drawdown),
        );
    }

    let applicants = parse
        .query::<NeatNetworkApplicants, _, _>("NeatNetworkApplicants", json!({}), None, None, None)
        .await;

    let mut candles = HashMap::new();

    for applicant in &applicants.results {
        let keys = get_keys_for_interval(applicant.from, applicant.to);
        let mut current_candles = vec![];
        for key in &keys {
            let store_key = format!("{}:{}:{}", key, applicant.interval, applicant.lookback);
            if !candles.contains_key(&store_key) {
                let new_candles = get_candles_with_cache(
                    "XRPUSDT".to_string(),
                    applicant.interval,
                    *key,
                    applicant.lookback,
                    None,
                )
                .await;
                candles.insert(store_key.to_string(), new_candles);
            }

            current_candles = [
                current_candles,
                candles.get(&store_key.to_string()).unwrap().to_vec(),
            ]
            .concat();
        }

        current_candles.sort();

        let max_result = *results_cache
            .get(&applicant.object_id.to_string())
            .unwrap_or(&0f64);

        let result = get_result(
            &organism,
            &current_candles,
            applicant.gain,
            applicant.lag,
            applicant.stake,
        );
        let score = result.wallet * result.drawdown;
        if score > max_result {
            let _ = save_parse_network_result(
                &parse,
                parent.object_id.to_string(),
                applicant.object_id.to_string(),
                result,
            )
            .await;

            println!(
                "{}: {:.8} - {:.8}",
                applicant.object_id.to_string(),
                max_result,
                score
            );
        }
    }
}
