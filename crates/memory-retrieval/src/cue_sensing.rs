use crate::*;
use nalgebra::DMatrix;
use std::collections::{BTreeMap, HashSet};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaBasis {
    pub mean: Vec<f64>,
    pub basis: Vec<Vec<f64>>,
    pub singular_energies: Vec<f64>,
    pub embedding_space: String,
}

/// A published, immutable EPA serving projection for one embedding space.
/// The basis is derived from active Tag vectors and is never Authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaBasisGeneration {
    pub generation_id: ServingGenerationId,
    pub basis: EpaBasis,
}

type RepresentativeSet = (Vec<(u64, Vec<f64>)>, Vec<f64>);

pub fn build_epa_basis(vectors: &[(u64, Vec<f64>)], embedding_space: &str) -> Option<EpaBasis> {
    if vectors.len() < 8 || vectors.iter().any(|(_, vector)| vector.is_empty()) {
        return None;
    }
    let dimension = vectors[0].1.len();
    if vectors.iter().any(|(_, vector)| vector.len() != dimension) {
        return None;
    }
    let vectors = vectors
        .iter()
        .filter_map(|(key, vector)| {
            let norm = vector.iter().map(|value| value * value).sum::<f64>().sqrt();
            (norm > f64::EPSILON).then(|| {
                (
                    *key,
                    vector.iter().map(|value| *value / norm).collect::<Vec<_>>(),
                )
            })
        })
        .collect::<Vec<_>>();
    if vectors.len() < 8 {
        return None;
    }
    let (vectors, weights) = if vectors.len() > 256 {
        representative_vectors(&vectors, embedding_space, dimension)
    } else {
        let weights = vec![1.0; vectors.len()];
        (vectors, weights)
    };
    let mut mean = vec![0.0; dimension];
    let total_weight = weights.iter().sum::<f64>().max(f64::EPSILON);
    for ((_, vector), weight) in vectors.iter().zip(&weights) {
        for (target, value) in mean.iter_mut().zip(vector.iter()) {
            *target += *weight * *value;
        }
    }
    for value in &mut mean {
        *value /= total_weight;
    }
    let data = vectors
        .iter()
        .flat_map(|(_, vector)| {
            vector
                .iter()
                .enumerate()
                .map(|(index, value)| value - mean[index])
        })
        .collect::<Vec<_>>();
    let matrix = DMatrix::from_row_slice(vectors.len(), dimension, &data);
    let svd = matrix.svd(false, true);
    let singular = svd.singular_values.iter().copied().collect::<Vec<_>>();
    let total_energy = singular
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .max(f64::EPSILON);
    let energies = singular
        .iter()
        .map(|value| value * value / total_energy)
        .collect::<Vec<_>>();
    let mut basis = Vec::new();
    if let Some(v_t) = svd.v_t {
        for (row, energy) in v_t.row_iter().zip(&energies) {
            if *energy < 0.01 || basis.len() >= 64 {
                break;
            }
            basis.push(row.iter().copied().collect());
        }
    }
    if basis.is_empty() {
        None
    } else {
        Some(EpaBasis {
            mean,
            basis,
            singular_energies: energies,
            embedding_space: embedding_space.to_owned(),
        })
    }
}

fn representative_vectors(
    vectors: &[(u64, Vec<f64>)],
    embedding_space: &str,
    dimension: usize,
) -> RepresentativeSet {
    let directions = [
        projection_direction(embedding_space, 0, dimension),
        projection_direction(embedding_space, 1, dimension),
    ];
    let mut buckets = BTreeMap::<(u8, u8), Vec<(usize, f64)>>::new();
    for (index, (_, vector)) in vectors.iter().enumerate() {
        let scores = directions
            .iter()
            .map(|direction| {
                vector
                    .iter()
                    .zip(direction)
                    .map(|(value, component)| value * component)
                    .sum::<f64>()
            })
            .collect::<Vec<_>>();
        let bucket = (bucket_score(scores[0]), bucket_score(scores[1]));
        let residual = vector.iter().map(|value| value * value).sum::<f64>();
        buckets.entry(bucket).or_default().push((index, residual));
    }

    let mut selected = HashSet::new();
    for members in buckets.values_mut() {
        members.sort_by(|left, right| {
            left.1
                .total_cmp(&right.1)
                .then_with(|| vectors[left.0].0.cmp(&vectors[right.0].0))
        });
        if let Some((index, _)) = members.first() {
            selected.insert(*index);
        }
        if let Some((index, _)) = members.last() {
            selected.insert(*index);
        }
    }

    let mut remaining = vectors
        .iter()
        .enumerate()
        .filter(|(index, _)| !selected.contains(index))
        .map(|(index, (key, vector))| {
            let residual = vector.iter().map(|value| value * value).sum::<f64>();
            (index, *key, residual)
        })
        .collect::<Vec<_>>();
    remaining.sort_by(|left, right| {
        right
            .2
            .total_cmp(&left.2)
            .then_with(|| left.1.cmp(&right.1))
    });
    for (index, _, _) in remaining {
        if selected.len() >= 256 {
            break;
        }
        selected.insert(index);
    }

    let mut indices = selected.into_iter().collect::<Vec<_>>();
    indices.sort_by_key(|index| vectors[*index].0);
    let representatives = indices
        .into_iter()
        .map(|index| vectors[index].clone())
        .collect::<Vec<_>>();
    let weights = representatives.iter().map(|_| 1.0).collect::<Vec<_>>();
    (representatives, weights)
}

fn projection_direction(space: &str, axis: usize, dimension: usize) -> Vec<f64> {
    let mut direction = (0..dimension)
        .map(|component| {
            let digest = blake3::hash(
                format!("nous-epa-projection-v1\0{space}\0{axis}\0{component}").as_bytes(),
            );
            f64::from(digest.as_bytes()[0]) / 127.5 - 1.0
        })
        .collect::<Vec<_>>();
    let norm = direction
        .iter()
        .map(|value| value * value)
        .sum::<f64>()
        .sqrt();
    if norm > f64::EPSILON {
        for value in &mut direction {
            *value /= norm;
        }
    }
    direction
}

fn bucket_score(score: f64) -> u8 {
    (((score.clamp(-1.0, 1.0) + 1.0) * 4.0).floor() as u8).min(7)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpaObservation {
    pub axis_energy: Vec<f64>,
    pub entropy: f64,
    pub focus: f64,
    pub dominant_axes: Vec<usize>,
    pub resonances: Vec<(usize, usize, f64)>,
}

pub fn observe_epa(basis: &EpaBasis, query: &[f64]) -> Option<EpaObservation> {
    if query.len() != basis.mean.len() || basis.basis.is_empty() {
        return None;
    }
    let centered = query
        .iter()
        .zip(&basis.mean)
        .map(|(query, mean)| query - mean)
        .collect::<Vec<_>>();
    let projections = basis
        .basis
        .iter()
        .map(|axis| axis.iter().zip(&centered).map(|(a, b)| a * b).sum::<f64>())
        .collect::<Vec<_>>();
    let total = projections.iter().map(|value| value * value).sum::<f64>();
    if total <= f64::EPSILON {
        return Some(EpaObservation {
            axis_energy: vec![0.0; projections.len()],
            entropy: 0.0,
            focus: 1.0,
            dominant_axes: Vec::new(),
            resonances: Vec::new(),
        });
    }
    let energy = projections
        .iter()
        .map(|value| value * value / total)
        .collect::<Vec<_>>();
    let denom = (energy.len() as f64).ln().max(1.0);
    let entropy = -energy
        .iter()
        .filter(|value| **value > 0.0)
        .map(|value| value * value.ln())
        .sum::<f64>()
        / denom;
    let dominant_axes = energy
        .iter()
        .enumerate()
        .filter_map(|(index, value)| (*value >= 0.05).then_some(index))
        .collect::<Vec<_>>();
    let mut resonances = Vec::new();
    for left in 0..dominant_axes.len() {
        for right in (left + 1)..dominant_axes.len() {
            let a = dominant_axes[left];
            let b = dominant_axes[right];
            let resonance = (energy[a] * energy[b]).sqrt();
            if resonance >= 0.15 {
                resonances.push((a, b, resonance));
            }
        }
    }
    Some(EpaObservation {
        axis_energy: energy,
        entropy,
        focus: (1.0 - entropy).clamp(0.0, 1.0),
        dominant_axes,
        resonances,
    })
}
