//! Finite vector math for request-local residual cue recovery.
#[derive(Debug, Clone)]
pub struct ResidualState {
    residual: Vec<f64>,
    directions: Vec<Vec<f64>>,
    pub energy_ratios: Vec<f64>,
}

pub fn normalize(vector: &[f32]) -> Option<Vec<f32>> {
    if vector.is_empty() || vector.iter().any(|v| !v.is_finite()) {
        return None;
    }
    let norm = vector
        .iter()
        .map(|v| f64::from(*v).powi(2))
        .sum::<f64>()
        .sqrt();
    if !norm.is_finite() || norm < 1e-12 {
        return None;
    }
    Some(
        vector
            .iter()
            .map(|v| (f64::from(*v) / norm) as f32)
            .collect(),
    )
}

pub fn dominant_direction(candidates: &[Vec<f32>], similarities: &[f32]) -> Option<Vec<f32>> {
    if candidates.len() != similarities.len() {
        return None;
    }
    let best = similarities
        .iter()
        .copied()
        .filter(|v| v.is_finite() && *v > 0.0)
        .max_by(f32::total_cmp)?;
    let mut direction = vec![0.0; candidates.first()?.len()];
    for (candidate, similarity) in candidates.iter().zip(similarities) {
        if candidate.len() != direction.len()
            || !similarity.is_finite()
            || *similarity <= 0.0
            || best - similarity > 0.08
        {
            continue;
        }
        let Some(candidate) = normalize(candidate) else {
            continue;
        };
        for (slot, value) in direction.iter_mut().zip(candidate) {
            *slot += value * similarity;
        }
    }
    normalize(&direction).or_else(|| candidates.iter().find_map(|v| normalize(v)))
}

impl ResidualState {
    pub fn new(query: &[f32]) -> Option<Self> {
        Some(Self {
            residual: normalize(query)?.into_iter().map(f64::from).collect(),
            directions: vec![],
            energy_ratios: vec![],
        })
    }

    /// Each step observes the most recent search neighborhood. Energy is always
    /// measured against the original unit query, never against the prior level.
    pub fn next(
        &mut self,
        candidates: &[Vec<f32>],
        similarities: &[f32],
    ) -> Result<Vec<f32>, &'static str> {
        let direction =
            dominant_direction(candidates, similarities).ok_or("no_valid_dominant_direction")?;
        if direction.len() != self.residual.len() {
            return Err("invalid_dimension");
        }
        let mut direction: Vec<f64> = direction.into_iter().map(f64::from).collect();
        for prior in &self.directions {
            let dot = direction.iter().zip(prior).map(|(a, b)| a * b).sum::<f64>();
            for (value, prior) in direction.iter_mut().zip(prior) {
                *value -= dot * prior;
            }
        }
        let norm = direction.iter().map(|v| v * v).sum::<f64>().sqrt();
        if !norm.is_finite() || norm < 1e-5 {
            return Err("collinear_or_degenerate_direction");
        }
        for value in &mut direction {
            *value /= norm;
        }
        let dot = self
            .residual
            .iter()
            .zip(&direction)
            .map(|(a, b)| a * b)
            .sum::<f64>();
        for (value, direction) in self.residual.iter_mut().zip(&direction) {
            *value -= dot * direction;
        }
        self.directions.push(direction);
        let energy = self.residual.iter().map(|v| v * v).sum::<f64>();
        if !energy.is_finite() {
            return Err("nonfinite_residual");
        }
        self.energy_ratios.push(energy);
        if energy < 0.08 {
            return Err("energy_below_threshold");
        }
        normalize(&self.residual.iter().map(|v| *v as f32).collect::<Vec<_>>())
            .ok_or("nonfinite_residual")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn absolute_energy_decreases_and_repeated_direction_stops() {
        let mut state = ResidualState::new(&[1.0, 1.0, 1.0]).unwrap();
        state.next(&[vec![1.0, 0.0, 0.0]], &[0.8]).unwrap();
        state.next(&[vec![0.0, 1.0, 0.0]], &[0.7]).unwrap();
        assert!((state.energy_ratios[0] - 2.0 / 3.0).abs() < 1e-6);
        assert!((state.energy_ratios[1] - 1.0 / 3.0).abs() < 1e-6);
        assert_eq!(
            state.next(&[vec![0.0, 1.0, 0.0]], &[0.7]),
            Err("collinear_or_degenerate_direction")
        );
    }
    #[test]
    fn nonfinite_and_exhausted_energy_stop() {
        assert!(ResidualState::new(&[f32::NAN]).is_none());
        let mut state = ResidualState::new(&[1.0, 0.0]).unwrap();
        assert_eq!(
            state.next(&[vec![1.0, 0.0]], &[1.0]),
            Err("energy_below_threshold")
        );
    }
}
