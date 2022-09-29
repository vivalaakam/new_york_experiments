use new_york_calculate_core::get_candles_with_cache;
use new_york_utils::make_id;
use serde_json::json;
use vivalaakam_neat_rs::{Activation, Config, Genome, Organism};

use crate::{Buffer, find_appropriate, get_now, get_score_fitness, load_networks, NeatNetworkApplicantType, NeatNetworkResults, Parse, save_parse_network, save_parse_network_result};
use crate::cleanup_results::cleanup_results;
use crate::get_keys_for_interval::get_keys_for_interval;
use crate::neat_network_applicants::NeatNetworkApplicants;

fn can_go_next(applicant: &NeatNetworkApplicants, epoch: usize, candles_len: usize, stagnation: usize, limit: usize) -> bool {
    match applicant.applicant_type {
        NeatNetworkApplicantType::StepsIterate => epoch < candles_len || stagnation < limit,
        _ => stagnation < limit
    }
}

pub async fn neat_score_applicant(
    parse: &Parse,
    applicant: NeatNetworkApplicants,
    config: Config,
    can_best: bool,
    can_crossover: bool,
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

    let stream = make_id(5);

    let keys = get_keys_for_interval(applicant.from, applicant.to);

    let mut candles = vec![];

    for key in keys {
        let new_candles = get_candles_with_cache(
            applicant.ticker.to_string(),
            applicant.interval,
            key,
            applicant.lookback,
            Some(applicant.indicators.to_vec()),
        )
            .await;
        candles = [candles, new_candles].concat();
    }

    candles.sort();

    let mut prev_score = 0f64;

    let results = parse
        .query::<NeatNetworkResults, _, _>(
            "NeatNetworkResults",
            json!({"applicantId": applicant.object_id.to_string(), "isUnique": true}),
            None,
            None,
            None,
        )
        .await;

    for result in results.results {
        prev_score = prev_score.max(result.wallet * result.drawdown);
    }

    let mut population = vec![];

    if can_best == true {
        let networks = load_networks(&parse, applicant.inputs, applicant.outputs).await;

        for network in networks {
            let mut organism = Organism::new(network.network.into());
            organism.set_id(network.object_id);
            get_score_fitness(&mut organism, &candles, &applicant, 0);
            population.push(organism);
        }

        population.sort();

        if can_crossover == true {
            if population.len() > population_size {
                population = population[0..population_size].to_vec();
            }

            let max_ind = population.len();

            for i in 0..max_ind {
                for j in 0..max_ind {
                    if i != j
                        && population[j].genome.get_nodes().len()
                        > applicant.inputs + applicant.outputs
                    {
                        match population[i].genome.mutate_crossover(&population[j].genome) {
                            Some(genome) => {
                                let mut organism = Organism::new(genome);
                                get_score_fitness(&mut organism, &candles, &applicant, 0);
                                population.push(organism);
                            }
                            _ => {}
                        }
                    }
                }
            }

            population.sort();
        }

        if population.len() > population_size {
            population = population[0..population_size].to_vec();
        }
    }

    let parent = population.pop();
    while population.len() < population_size * 2 {
        let mut organism = match parent.as_ref() {
            None => Organism::new(Genome::generate_genome(
                applicant.inputs,
                applicant.outputs,
                vec![],
                Some(Activation::Sigmoid),
                &config,
            )),
            Some(organism) => organism.mutate(None, &config).unwrap(),
        };

        get_score_fitness(&mut organism, &candles, &applicant, 0);
        if organism.get_fitness() > 0f64 {
            population.push(organism);
        }
    }

    population.sort();
    population = population[0..population_size].to_vec();

    let mut epoch = 0;
    println!(
        "{} - {} - {} : high: {:.8} prev: {prev_score:.8}",
        applicant.from, applicant.to, stream, applicant.high_score
    );
    let mut buffer = Buffer::new(10);
    let mut best_result = None;

    let candles_len = candles.len();

    while can_go_next(&applicant, epoch, candles_len, population[0].get_stagnation(), stagnation) == true {
        let start = get_now();
        let mut new_population = vec![];

        for i in 0..population.len() {
            let child = find_appropriate(&population, i);

            match population[i].mutate(child, &config) {
                None => {}
                Some(organism) => new_population.push(organism),
            }

            match applicant.applicant_type {
                NeatNetworkApplicantType::StepsIterate | NeatNetworkApplicantType::StepsIterateBack => {
                    match population.get_mut(i) {
                        None => {}
                        Some(organism) => {
                            get_score_fitness(organism, &candles, &applicant, epoch);
                        }
                    }
                }
                _ => {}
            }
        }

        for organism in new_population.iter_mut() {
            get_score_fitness(organism, &candles, &applicant, epoch);
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

            if best.get_id().is_none() {
                let result = applicant.get_result(&best, &candles, epoch);
                let network_id =
                    save_parse_network(&parse, best, applicant.inputs, applicant.outputs).await;
                let result_id = save_parse_network_result(
                    &parse,
                    network_id.to_string(),
                    applicant.object_id.to_string(),
                    result,
                    false,
                    stream.to_string(),
                )
                    .await;

                best_result = Some(result_id);

                best.set_id(network_id);
            }
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
            if let Some(network_id) = best.get_id() {
                network_id_ret = Some(network_id.to_string());
            }

            if let Some(best_result_id) = best_result {
                parse
                    .update::<NeatNetworkResults, _, _, _>(
                        "NeatNetworkResults",
                        best_result_id.to_string(),
                        json!({ "isUnique": true }),
                    )
                    .await;
            }
        }
    }

    let current_results = parse
        .query::<NeatNetworkResults, _, _>(
            "NeatNetworkResults",
            json!({
                "isUnique": false,
                "applicantId": applicant.object_id,
                "stream": stream.to_string()
            }),
            Some(10000),
            None,
            None,
        )
        .await;

    for row in current_results.results {
        cleanup_results(&parse, &row).await;
    }

    let mut exists = parse
        .query::<NeatNetworkResults, _, _>(
            "NeatNetworkResults",
            json!({
                "isUnique": true,
                "applicantId": applicant.object_id,
            }),
            Some(10000),
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

    network_id_ret
}
