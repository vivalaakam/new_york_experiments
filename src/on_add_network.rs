use std::collections::HashMap;

use new_york_calculate_core::{get_candles_with_cache, utils::ceil_to_nearest};
use new_york_utils::make_id;
use serde_json::json;
use vivalaakam_neat_rs::Organism;

use crate::{
    hash_md5, NeatNetworkApplicants, NeatNetworkResults, NeatNetworks, Parse,
    save_parse_network_result,
};
use crate::cleanup_results::cleanup_results;
use crate::get_keys_for_interval::get_keys_for_interval;

pub async fn on_add_network(parse: &Parse, network_id: String) {
    let parent = parse
        .get::<NeatNetworks, _, _>("NeatNetworks", network_id)
        .await;

    if parent.is_none() {
        return;
    }

    let parent = parent.unwrap();

    let organism = Organism::new(parent.network.into());

    let applicants = parse
        .query::<NeatNetworkApplicants, _, _>(
            "NeatNetworkApplicants",
            json!({
                "inputs": parent.inputs,
                "outputs": parent.outputs,
            }),
            Some(10000),
            None,
            None,
        )
        .await;

    let applicants_key = applicants
        .results
        .to_vec()
        .into_iter()
        .map(|applicant| applicant.object_id)
        .collect::<Vec<_>>();

    let mut cont = true;
    let mut skip = 0;
    let mut results_cache: HashMap<String, f64> = HashMap::new();

    while cont == true {
        let results = parse
            .query::<NeatNetworkResults, _, _>(
                "NeatNetworkResults",
                json!({
                    "applicantId": {
                        "$in": applicants_key
                    }
                }),
                None,
                Some(skip),
                None,
            )
            .await;

        for result in &results.results {
            let value = *results_cache.get(&result.applicant_id).unwrap_or(&f64::MAX);
            results_cache.insert(
                result.applicant_id.to_string(),
                ceil_to_nearest(value.min(result.wallet * result.drawdown), 0.00000001),
            );
        }
        skip += results.results.len();
        if results.results.len() < 1000 {
            cont = false;
        }
    }

    let mut candles = HashMap::new();

    let mut best = (0.0, None);

    for applicant in &applicants.results {
        let keys = get_keys_for_interval(applicant.from, applicant.to);
        let mut current_candles = vec![];
        for key in &keys {
            let store_key = format!(
                "{}:{}:{}:{}:{}",
                key,
                applicant.ticker,
                applicant.interval,
                applicant.lookback,
                hash_md5(format!("{:?}", applicant.indicators.to_vec()))
            );
            if !candles.contains_key(&store_key) {
                let new_candles = get_candles_with_cache(
                    applicant.ticker.to_string(),
                    applicant.interval,
                    *key,
                    applicant.lookback,
                    Some(applicant.indicators.to_vec()),
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

        let result = applicant.get_result(&organism, &current_candles, current_candles.len());

        let score = ceil_to_nearest(result.wallet * result.drawdown, 0.00000001);
        if score > max_result {
            let _ = save_parse_network_result(
                &parse,
                parent.object_id.to_string(),
                applicant.object_id.to_string(),
                result,
                true,
                make_id(5)
            )
                .await;

            let mut exists = parse
                .query::<NeatNetworkResults, _, _>(
                    "NeatNetworkResults",
                    json!({"applicantId": applicant.object_id.to_string(), "isUnique": true}),
                    None,
                    None,
                    None,
                )
                .await;

            if exists.results.len() > 10 {
                exists.results.sort_by(|a, b| {
                    (b.wallet * b.drawdown)
                        .partial_cmp(&(a.wallet * a.drawdown))
                        .unwrap()
                });

                while exists.results.len() > 10 {
                    if let Some(last) = exists.results.pop() {
                        cleanup_results(&parse, &last).await;
                    }
                }
            }

            if score / max_result > best.0 {
                best = (score / max_result, Some(applicant.object_id.to_string()))
            }

            println!(
                "{} ({}): {:.8} - {:.8} {:.8}/d {:.2}",
                applicant.object_id.to_string(),
                applicant.days,
                max_result,
                score,
                score / applicant.days as f64,
                (score / max_result) * 100.0 - 100.0
            );
        }
    }

    if let Some(best_id) = best.1 {
        println!("{best_id}");
    }
}
