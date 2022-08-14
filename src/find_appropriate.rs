use new_york_utils::levenshtein;
use vivalaakam_neat_rs::Organism;

pub fn find_appropriate(population: &Vec<Organism>, start: usize) -> Option<&Organism> {
    let mut child = None;

    let mut min_score = i32::MAX;
    let mut min_j = start;

    for j in start + 1..population.len() {
        let score = levenshtein(
            population[start].get_genotype(),
            population[j].get_genotype(),
        )
        .unwrap_or(i32::MAX);

        if score > 0 && score < min_score {
            min_score = score;
            min_j = j;
        }
    }

    if min_j != start {
        child = population.get(min_j);
    }

    child
}
