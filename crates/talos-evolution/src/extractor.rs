//! PatternExtractor — extracts patterns from observations.

use crate::{Observation, Pattern, SignalType};

/// Extracts patterns from observations using rule-based logic.
pub struct PatternExtractor;

impl PatternExtractor {
    /// Extract a pattern from a single observation.
    pub fn extract_from_observation(obs: &Observation) -> Option<Pattern> {
        match obs.signal_type {
            SignalType::Correction => Self::extract_correction_pattern(obs),
            SignalType::Error => Self::extract_error_pattern(obs),
            SignalType::Satisfaction => Self::extract_satisfaction_pattern(obs),
            SignalType::Inefficiency => Self::extract_inefficiency_pattern(obs),
        }
    }

    fn extract_correction_pattern(obs: &Observation) -> Option<Pattern> {
        if obs.intensity < 0.5 {
            return None;
        }

        let mut pattern = Pattern::new(
            format!("User preference: {}", obs.context),
            format!("Remember: {}", obs.context),
            "preference".to_string(),
        );
        pattern.confidence = obs.intensity * 0.5;
        pattern.evidence_count = 1;
        Some(pattern)
    }

    fn extract_error_pattern(obs: &Observation) -> Option<Pattern> {
        if obs.intensity < 0.3 {
            return None;
        }

        let mut pattern = Pattern::new(
            format!("Error to avoid: {}", obs.context),
            format!("Avoid: {}", obs.context),
            "error_avoidance".to_string(),
        );
        pattern.confidence = obs.intensity * 0.4;
        pattern.evidence_count = 1;
        Some(pattern)
    }

    fn extract_satisfaction_pattern(_obs: &Observation) -> Option<Pattern> {
        None
    }

    fn extract_inefficiency_pattern(obs: &Observation) -> Option<Pattern> {
        if obs.intensity < 0.4 {
            return None;
        }

        let mut pattern = Pattern::new(
            format!("Inefficiency detected: {}", obs.context),
            format!("Optimize: {}", obs.context),
            "efficiency".to_string(),
        );
        pattern.confidence = obs.intensity * 0.3;
        pattern.evidence_count = 1;
        Some(pattern)
    }

    /// Check if a new pattern contradicts an existing pattern.
    pub fn detects_conflict(new_pattern: &Pattern, existing: &[Pattern]) -> Option<String> {
        for p in existing {
            if p.category == new_pattern.category {
                let new_lower = new_pattern.description.to_lowercase();
                let existing_lower = p.description.to_lowercase();

                if new_lower.contains("avoid") && !existing_lower.contains("avoid")
                    || !new_lower.contains("avoid") && existing_lower.contains("avoid")
                {
                    return Some(format!(
                        "Conflict between '{}' and '{}'",
                        new_pattern.description, p.description
                    ));
                }
            }
        }
        None
    }

    /// Merge evidence from multiple observations into a pattern.
    pub fn merge_evidence(pattern: &mut Pattern, observations: &[Observation]) {
        let relevant: Vec<&Observation> = observations
            .iter()
            .filter(|o| {
                matches!(
                    o.signal_type,
                    SignalType::Correction | SignalType::Error | SignalType::Inefficiency
                )
            })
            .collect();

        if relevant.is_empty() {
            return;
        }

        let total_intensity: f64 = relevant.iter().map(|o| o.intensity).sum();
        let avg_intensity = total_intensity / relevant.len() as f64;

        pattern.evidence_count += relevant.len() as u32;
        pattern.confidence = (pattern.confidence + avg_intensity) / 2.0;
        pattern.last_updated = chrono::Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_correction_pattern() {
        let obs = Observation::new(
            SignalType::Correction,
            0.8,
            "Use functional style".to_string(),
            None,
            None,
        );

        let pattern = PatternExtractor::extract_from_observation(&obs);
        assert!(pattern.is_some());

        let pattern = pattern.unwrap();
        assert_eq!(pattern.category, "preference");
        assert!(pattern.confidence > 0.0);
    }

    #[test]
    fn test_extract_low_intensity_returns_none() {
        let obs = Observation::new(
            SignalType::Correction,
            0.2,
            "Minor correction".to_string(),
            None,
            None,
        );

        let pattern = PatternExtractor::extract_from_observation(&obs);
        assert!(pattern.is_none());
    }

    #[test]
    fn test_detects_conflict() {
        let existing = vec![Pattern::new(
            "Avoid imperative style".to_string(),
            "Use functional style".to_string(),
            "preference".to_string(),
        )];

        let new_pattern = Pattern::new(
            "Use imperative style".to_string(),
            "Use imperative style".to_string(),
            "preference".to_string(),
        );

        let conflict = PatternExtractor::detects_conflict(&new_pattern, &existing);
        assert!(conflict.is_some());
    }

    #[test]
    fn test_merge_evidence() {
        let mut pattern = Pattern::new(
            "Test pattern".to_string(),
            "Test instruction".to_string(),
            "test".to_string(),
        );
        pattern.confidence = 0.5;
        pattern.evidence_count = 1;

        let observations = vec![
            Observation::new(
                SignalType::Correction,
                0.7,
                "context1".to_string(),
                None,
                None,
            ),
            Observation::new(SignalType::Error, 0.6, "context2".to_string(), None, None),
        ];

        PatternExtractor::merge_evidence(&mut pattern, &observations);

        assert_eq!(pattern.evidence_count, 3);
        assert!(pattern.confidence > 0.5);
    }
}
