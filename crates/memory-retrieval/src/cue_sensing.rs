use crate::*;
use nalgebra::DMatrix;

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
    let vectors = if vectors.len() > 256 {
        let last = vectors.len() - 1;
        (0..256)
            .map(|slot| vectors[(slot * last) / 255].clone())
            .collect::<Vec<_>>()
    } else {
        vectors
    };
    let mut mean = vec![0.0; dimension];
    for (_, vector) in &vectors {
        for (target, value) in mean.iter_mut().zip(vector.iter()) {
            *target += *value;
        }
    }
    for value in &mut mean {
        *value /= vectors.len() as f64;
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
