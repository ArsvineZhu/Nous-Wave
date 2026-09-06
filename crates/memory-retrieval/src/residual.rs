//! Bounded residual query planning. This module owns only finite-vector math;
//! persistence and candidate identity remain in the service/retrieval owners.

#[derive(Debug, Clone, PartialEq)]
pub struct ResidualPlan {
    pub vectors: Vec<Vec<f32>>,
    pub energy_ratios: Vec<f64>,
    pub stop_reason: Option<String>,
}

pub fn normalize(vector: &[f32]) -> Option<Vec<f32>> {
    if vector.is_empty() || vector.iter().any(|value| !value.is_finite()) {
        return None;
    }
    let norm = vector
        .iter()
        .map(|value| f64::from(*value) * f64::from(*value))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm < 1e-12 {
        return None;
    }
    Some(
        vector
            .iter()
            .map(|value| (*value as f64 / norm) as f32)
            .collect(),
    )
}

pub fn dominant_direction(candidates: &[Vec<f32>], similarities: &[f32]) -> Option<Vec<f32>> {
    if candidates.is_empty() || candidates.len() != similarities.len() {
        return None;
    }
    let best = similarities
        .iter()
        .copied()
        .filter(|value| value.is_finite() && *value > 0.0)
        .fold(f32::NEG_INFINITY, f32::max);
    if !best.is_finite() {
        return None;
    }
    let dimension = candidates.first()?.len();
    let mut direction = vec![0.0_f64; dimension];
    for (candidate, similarity) in candidates.iter().zip(similarities) {
        if candidate.len() != dimension
            || candidate.iter().any(|value| !value.is_finite())
            || !similarity.is_finite()
            || *similarity <= 0.0
            || best - *similarity > 0.08
        {
            continue;
        }
        for (slot, value) in direction.iter_mut().zip(candidate) {
            *slot += f64::from(*similarity) * f64::from(*value);
        }
    }
    normalize(
        &direction
            .iter()
            .map(|value| *value as f32)
            .collect::<Vec<_>>(),
    )
}

pub fn plan(
    query: &[f32],
    candidates: &[Vec<f32>],
    similarities: &[f32],
    max_rounds: usize,
) -> ResidualPlan {
    let Some(query) = normalize(query) else {
        return ResidualPlan {
            vectors: vec![],
            energy_ratios: vec![],
            stop_reason: Some("nonfinite_or_zero_query".into()),
        };
    };
    let Some(first) = dominant_direction(candidates, similarities) else {
        return ResidualPlan {
            vectors: vec![],
            energy_ratios: vec![],
            stop_reason: Some("no_valid_dominant_direction".into()),
        };
    };
    let mut directions = vec![first];
    let mut residual = query
        .iter()
        .map(|value| f64::from(*value))
        .collect::<Vec<_>>();
    let mut energy = 1.0_f64;
    let mut vectors = vec![];
    let mut energy_ratios = vec![];
    let mut stop_reason = None;
    for round in 0..max_rounds {
        let mut direction = directions[round]
            .iter()
            .map(|value| f64::from(*value))
            .collect::<Vec<_>>();
        for prior in directions.iter().take(round) {
            let projection = direction
                .iter()
                .zip(prior)
                .map(|(left, right)| left * f64::from(*right))
                .sum::<f64>();
            for (slot, value) in direction.iter_mut().zip(prior) {
                *slot -= projection * f64::from(*value);
            }
        }
        let norm = direction
            .iter()
            .map(|value| value * value)
            .sum::<f64>()
            .sqrt();
        if !norm.is_finite() || norm < 1e-5 {
            stop_reason = Some("collinear_or_degenerate_direction".into());
            break;
        }
        for value in &mut direction {
            *value /= norm;
        }
        let projection = residual
            .iter()
            .zip(&direction)
            .map(|(left, right)| left * right)
            .sum::<f64>();
        for (slot, value) in residual.iter_mut().zip(&direction) {
            *slot -= projection * value;
        }
        let next_energy = residual.iter().map(|value| value * value).sum::<f64>();
        let ratio = next_energy / energy.max(f64::EPSILON);
        if !ratio.is_finite() {
            stop_reason = Some("nonfinite_residual".into());
            break;
        }
        energy = next_energy;
        energy_ratios.push(ratio);
        if ratio < 0.08 {
            stop_reason = Some("energy_below_threshold".into());
            break;
        }
        let Some(vector) = normalize(
            &residual
                .iter()
                .map(|value| *value as f32)
                .collect::<Vec<_>>(),
        ) else {
            stop_reason = Some("zero_residual".into());
            break;
        };
        vectors.push(vector);
        // A later direction is derived from the same bounded probe. If no
        // novel orthogonal direction is available, stop without inventing a
        // second candidate pool.
        if round + 1 < max_rounds {
            let mut next = None;
            for candidate in candidates {
                if candidate.len() != residual.len() {
                    continue;
                }
                let mut candidate = candidate
                    .iter()
                    .map(|value| f64::from(*value))
                    .collect::<Vec<_>>();
                for prior in &directions {
                    let projection = candidate
                        .iter()
                        .zip(prior)
                        .map(|(left, right)| left * f64::from(*right))
                        .sum::<f64>();
                    for (slot, value) in candidate.iter_mut().zip(prior) {
                        *slot -= projection * f64::from(*value);
                    }
                }
                if candidate
                    .iter()
                    .map(|value| value * value)
                    .sum::<f64>()
                    .sqrt()
                    >= 1e-5
                {
                    next = normalize(
                        &candidate
                            .iter()
                            .map(|value| *value as f32)
                            .collect::<Vec<_>>(),
                    );
                    if next.is_some() {
                        break;
                    }
                }
            }
            if let Some(next) = next {
                directions.push(next);
            } else {
                stop_reason = Some("no_novel_direction".into());
                break;
            }
        }
    }
    if stop_reason.is_none() && vectors.len() == max_rounds {
        stop_reason = Some("round_budget_exhausted".into());
    }
    ResidualPlan {
        vectors,
        energy_ratios,
        stop_reason,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn residual_energy_decreases_for_non_collinear_probe() {
        let plan = plan(
            &[1.0, 1.0, 0.0],
            &[vec![1.0, 0.0, 0.0], vec![0.0, 1.0, 0.0]],
            &[0.7, 0.6],
            2,
        );
        assert!(!plan.vectors.is_empty());
        assert!(plan.energy_ratios[0] < 1.0);
    }

    #[test]
    fn collinear_probe_terminates() {
        let plan = plan(
            &[1.0, 0.0],
            &[vec![1.0, 0.0], vec![1.0, 0.0]],
            &[1.0, 0.9],
            3,
        );
        assert!(plan.stop_reason.is_some());
    }
}
