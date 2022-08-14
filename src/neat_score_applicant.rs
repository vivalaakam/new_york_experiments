use new_york_calculate_core::get_candles_with_cache;
use serde_json::json;
use vivalaakam_neat_rs::{Activation, Config, Genome, Organism};

use crate::get_keys_for_interval::get_keys_for_interval;
use crate::{
    find_appropriate, get_now, get_result, get_score_fitness, load_networks, save_parse_network,
    save_parse_network_result, Buffer, NeatNetworkApplicants, NeatNetworkResults, Parse,
};

pub async fn neat_score_applicant(
    parse: &Parse,
    applicant: NeatNetworkApplicants,
    config: Config,
    can_best: bool,
    stagnation: usize,
    population_size: usize,
) -> Option<String> {
    parse
        .update::<NeatNetworkApplicants, _, _, _>(
            "NeatNetworkApplicants",
            applicant.object_id.to_string(),
            json!({
                "touches": applicant.touches + 1
            }),
        )
        .await;

    let inputs = applicant.lookback * 15;
    let keys = get_keys_for_interval(applicant.from, applicant.to);

    let mut candles = vec![];

    for key in keys {
        let new_candles = get_candles_with_cache(
            "XRPUSDT".to_string(),
            applicant.interval,
            key,
            applicant.lookback,
            None,
        )
        .await;
        candles = [candles, new_candles].concat();
    }

    candles.sort();

    let mut prev_score = 0f64;

    let results = parse
        .query::<NeatNetworkResults, _, _>(
            "NeatNetworkResults",
            json!({"applicantId": applicant.object_id.to_string()}),
            None,
            None,
            None,
        )
        .await;

    for result in results.results {
        prev_score = prev_score.max(result.wallet * result.drawdown);
    }

    let mut stack = vec![];
    if can_best == true {
        let networks = load_networks(&parse, applicant.inputs, applicant.outputs).await;

        for network in networks {
            let mut organism = Organism::new(network.network.into());
            organism.set_id(network.object_id);
            get_score_fitness(
                &mut organism,
                &candles,
                applicant.gain,
                applicant.lag,
                applicant.stake,
            );
            stack.push(organism);
        }
    }

    stack.sort_by(|a, b| a.get_fitness().partial_cmp(&b.get_fitness()).unwrap());
    if stack.len() > population_size {
        stack = stack[stack.len() - population_size..stack.len()].to_vec();
    }

    let parent = stack.pop();

    let mut population = vec![];

    while population.len() < population_size * 10 {
        let mut organism = match parent.as_ref() {
            None => Organism::new(Genome::generate_genome(
                inputs,
                2,
                vec![],
                Some(Activation::Relu),
                &config,
            )),
            Some(organism) => organism.mutate(None, &config).unwrap(),
        };

        get_score_fitness(
            &mut organism,
            &candles,
            applicant.gain,
            applicant.lag,
            applicant.stake,
        );
        population.push(organism);
    }

    population.sort();
    population = population[0..population_size].to_vec();

    let mut epoch = 0;
    println!(
        "{} - {} : high: {:.8} prev: {prev_score:.8}",
        applicant.from, applicant.to, applicant.high_score
    );
    let mut buffer = Buffer::new(10);
    while population[0].get_stagnation() < stagnation {
        let start = get_now();
        let mut new_population = vec![];

        for i in 0..population.len() {
            let child = find_appropriate(&population, i);

            match population[i].mutate(child, &config) {
                None => {}
                Some(organism) => new_population.push(organism),
            }
        }

        for organism in new_population.iter_mut() {
            get_score_fitness(
                organism,
                &candles,
                applicant.gain,
                applicant.lag,
                applicant.stake,
            );
        }

        population = [population, new_population].concat();
        population.sort();
        population = population[0..population_size].to_vec();

        let duration = (get_now() - start) as f64 / 1000.0;
        buffer.push(duration);

        if let Some(best) = population.first_mut() {
            println!(
                "{epoch: >6} {:.8} {: >3} ( dur: {:.3}, avg: {:.3} )",
                best.get_fitness(),
                best.get_stagnation(),
                duration,
                buffer.avg()
            );

            best.inc_stagnation();
        }

        epoch += 1;
    }

    let mut network_id_ret = None;

    if let Some(best) = population.first_mut() {
        println!(
            "best {:.8} {: >3}",
            best.get_fitness(),
            best.get_stagnation(),
        );

        if best.get_fitness() > prev_score {
            let result = get_result(
                &best,
                &candles,
                applicant.gain,
                applicant.lag,
                applicant.stake,
            );
            let network_id = save_parse_network(&parse, best, inputs, 2).await;
            let _result_id = save_parse_network_result(
                &parse,
                network_id.to_string(),
                applicant.object_id.to_string(),
                result,
            )
            .await;

            network_id_ret = Some(network_id.to_string())
        }
    }

    network_id_ret
}
