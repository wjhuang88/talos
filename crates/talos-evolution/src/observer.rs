//! TurnObserver — captures signals during agent execution.

use crate::{Observation, SignalType};

/// Captures observations during agent turns.
pub struct TurnObserver {
    /// Current session ID
    session_id: Option<String>,
    /// Current turn number
    turn_number: u32,
    /// Accumulated observations for current turn
    observations: Vec<Observation>,
}

impl TurnObserver {
    /// Create a new TurnObserver.
    pub fn new(session_id: Option<String>) -> Self {
        Self {
            session_id,
            turn_number: 0,
            observations: Vec::new(),
        }
    }

    /// Start a new turn.
    pub fn start_turn(&mut self) {
        self.turn_number += 1;
        self.observations.clear();
    }

    /// Record a correction signal.
    pub fn record_correction(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Correction,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record an error signal.
    pub fn record_error(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Error,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record a satisfaction signal.
    pub fn record_satisfaction(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Satisfaction,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Record an inefficiency signal.
    pub fn record_inefficiency(&mut self, context: String, intensity: f64) {
        let obs = Observation::new(
            SignalType::Inefficiency,
            intensity.clamp(0.0, 1.0),
            context,
            self.session_id.clone(),
            Some(self.turn_number),
        );
        self.observations.push(obs);
    }

    /// Get all observations for the current turn.
    pub fn current_observations(&self) -> &[Observation] {
        &self.observations
    }

    /// Get the current turn number.
    pub fn turn_number(&self) -> u32 {
        self.turn_number
    }

    /// Get the session ID.
    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    /// Drain observations from the current turn.
    pub fn drain_observations(&mut self) -> Vec<Observation> {
        std::mem::take(&mut self.observations)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_turn_observer_new() {
        let observer = TurnObserver::new(Some("session-1".to_string()));
        assert_eq!(observer.turn_number(), 0);
        assert_eq!(observer.session_id(), Some("session-1"));
    }

    #[test]
    fn test_record_signals() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();

        observer.record_correction("User said to use functional style".to_string(), 0.8);
        observer.record_error("File not found".to_string(), 0.5);
        observer.record_satisfaction("Good response".to_string(), 0.9);
        observer.record_inefficiency("Took too many steps".to_string(), 0.3);

        let observations = observer.current_observations();
        assert_eq!(observations.len(), 4);
        assert_eq!(observations[0].signal_type, SignalType::Correction);
        assert_eq!(observations[1].signal_type, SignalType::Error);
        assert_eq!(observations[2].signal_type, SignalType::Satisfaction);
        assert_eq!(observations[3].signal_type, SignalType::Inefficiency);
    }

    #[test]
    fn test_drain_observations() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();
        observer.record_correction("test".to_string(), 0.5);

        let drained = observer.drain_observations();
        assert_eq!(drained.len(), 1);
        assert!(observer.current_observations().is_empty());
    }

    #[test]
    fn test_turn_increment() {
        let mut observer = TurnObserver::new(None);
        observer.start_turn();
        assert_eq!(observer.turn_number(), 1);

        observer.start_turn();
        assert_eq!(observer.turn_number(), 2);
    }
}
