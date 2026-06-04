//! Deterministic replay of agent experiments from seeds.
//!
//! This crate provides tools for recording, replaying, and comparing
//! agent experiments in a fully deterministic manner using seed values.

#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Ternary signal: positive (+1), negative (-1), or neutral (0).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TernarySignal {
    Positive,
    Negative,
    Neutral,
}

impl TernarySignal {
    /// Convert to numeric value.
    pub fn value(&self) -> i8 {
        match self {
            TernarySignal::Positive => 1,
            TernarySignal::Negative => -1,
            TernarySignal::Neutral => 0,
        }
    }

    /// Convert from numeric value.
    pub fn from_value(v: i8) -> Option<Self> {
        match v {
            1 => Some(TernarySignal::Positive),
            -1 => Some(TernarySignal::Negative),
            0 => Some(TernarySignal::Neutral),
            _ => None,
        }
    }
}

/// A single step in an experiment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub index: usize,
    pub signal: TernarySignal,
    pub metadata: HashMap<String, String>,
}

/// A recorded experiment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recording {
    pub name: String,
    pub seed: u64,
    pub steps: Vec<Step>,
}

impl Recording {
    /// Create a new empty recording.
    pub fn new(name: impl Into<String>, seed: u64) -> Self {
        Recording {
            name: name.into(),
            seed,
            steps: Vec::new(),
        }
    }

    /// Add a step to the recording.
    pub fn add_step(&mut self, signal: TernarySignal) {
        let index = self.steps.len();
        self.steps.push(Step {
            index,
            signal,
            metadata: HashMap::new(),
        });
    }

    /// Add a step with metadata.
    pub fn add_step_with_metadata(
        &mut self,
        signal: TernarySignal,
        metadata: HashMap<String, String>,
    ) {
        let index = self.steps.len();
        self.steps.push(Step {
            index,
            signal,
            metadata,
        });
    }

    /// Get the total number of steps.
    pub fn len(&self) -> usize {
        self.steps.len()
    }

    /// Check if the recording is empty.
    pub fn is_empty(&self) -> bool {
        self.steps.is_empty()
    }

    /// Extract the sequence of signals.
    pub fn signals(&self) -> Vec<TernarySignal> {
        self.steps.iter().map(|s| s.signal).collect()
    }

    /// Compute the cumulative score.
    pub fn cumulative_score(&self) -> i64 {
        self.steps.iter().map(|s| s.signal.value() as i64).sum()
    }
}

/// A simple deterministic RNG based on a seed (xorshift64).
#[derive(Debug, Clone)]
pub struct SeedRng {
    state: u64,
}

impl SeedRng {
    /// Create a new RNG from a seed.
    pub fn new(seed: u64) -> Self {
        SeedRng {
            state: if seed == 0 { 1 } else { seed },
        }
    }

    /// Generate the next u64.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }

    /// Generate a ternary signal.
    pub fn next_ternary(&mut self) -> TernarySignal {
        match self.next_u64() % 3 {
            0 => TernarySignal::Neutral,
            1 => TernarySignal::Positive,
            _ => TernarySignal::Negative,
        }
    }

    /// Generate a bool.
    pub fn next_bool(&mut self) -> bool {
        self.next_u64() % 2 == 0
    }

    /// Get the current state.
    pub fn state(&self) -> u64 {
        self.state
    }

    /// Reset to original seed.
    pub fn reset(&mut self, seed: u64) {
        self.state = if seed == 0 { 1 } else { seed };
    }
}

/// Manages seeds for experiments.
#[derive(Debug, Clone)]
pub struct SeedManager {
    seeds: HashMap<String, u64>,
    counter: u64,
}

impl SeedManager {
    /// Create a new seed manager.
    pub fn new() -> Self {
        SeedManager {
            seeds: HashMap::new(),
            counter: 1,
        }
    }

    /// Register a seed with a name.
    pub fn register(&mut self, name: impl Into<String>, seed: u64) {
        self.seeds.insert(name.into(), seed);
    }

    /// Generate and register a new seed.
    pub fn generate(&mut self) -> (String, u64) {
        let name = format!("seed_{}", self.counter);
        // Simple deterministic generation
        let seed = self.counter.wrapping_mul(0x5851F42D4C957F2D);
        self.seeds.insert(name.clone(), seed);
        self.counter += 1;
        (name, seed)
    }

    /// Get a seed by name.
    pub fn get(&self, name: &str) -> Option<u64> {
        self.seeds.get(name).copied()
    }

    /// List all seeds.
    pub fn list(&self) -> &HashMap<String, u64> {
        &self.seeds
    }

    /// Remove a seed.
    pub fn remove(&mut self, name: &str) -> Option<u64> {
        self.seeds.remove(name)
    }

    /// Count seeds.
    pub fn len(&self) -> usize {
        self.seeds.len()
    }

    /// Check if empty.
    pub fn is_empty(&self) -> bool {
        self.seeds.is_empty()
    }
}

impl Default for SeedManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Engine for deterministic replay of experiments.
#[derive(Debug, Clone)]
pub struct ReplayEngine {
    rng: SeedRng,
    step_count: usize,
    max_steps: usize,
}

impl ReplayEngine {
    /// Create a new replay engine with a seed.
    pub fn new(seed: u64) -> Self {
        ReplayEngine {
            rng: SeedRng::new(seed),
            step_count: 0,
            max_steps: usize::MAX,
        }
    }

    /// Set max steps for replay.
    pub fn with_max_steps(mut self, max: usize) -> Self {
        self.max_steps = max;
        self
    }

    /// Replay a recording from its seed.
    pub fn replay(&mut self, recording: &Recording) -> Recording {
        self.rng.reset(recording.seed);
        self.step_count = 0;
        let limit = self.max_steps.min(recording.steps.len());

        let mut result = Recording::new(&recording.name, recording.seed);
        for _ in 0..limit {
            let signal = self.rng.next_ternary();
            result.add_step(signal);
            self.step_count += 1;
        }
        result
    }

    /// Generate a recording from scratch using the seed.
    pub fn generate(&mut self, name: impl Into<String>, seed: u64, steps: usize) -> Recording {
        self.rng.reset(seed);
        self.step_count = 0;
        let mut recording = Recording::new(name, seed);
        let limit = self.max_steps.min(steps);
        for _ in 0..limit {
            let signal = self.rng.next_ternary();
            recording.add_step(signal);
            self.step_count += 1;
        }
        recording
    }

    /// Get steps completed.
    pub fn steps_completed(&self) -> usize {
        self.step_count
    }

    /// Reset the engine with a new seed.
    pub fn reset(&mut self, seed: u64) {
        self.rng.reset(seed);
        self.step_count = 0;
    }
}

/// Comparison result between two recordings.
#[derive(Debug, Clone, PartialEq)]
pub struct ComparisonResult {
    pub name_a: String,
    pub name_b: String,
    pub total_steps: usize,
    pub matching_steps: usize,
    pub match_rate: f64,
    pub score_diff: i64,
}

/// Compare recordings for determinism verification.
#[derive(Debug)]
pub struct ReplayComparison;

impl ReplayComparison {
    /// Compare two recordings.
    pub fn compare(a: &Recording, b: &Recording) -> ComparisonResult {
        let total = a.steps.len().max(b.steps.len());
        let matching = a
            .steps
            .iter()
            .zip(b.steps.iter())
            .filter(|(sa, sb)| sa.signal == sb.signal)
            .count();

        let match_rate = if total > 0 {
            matching as f64 / total as f64
        } else {
            1.0
        };

        ComparisonResult {
            name_a: a.name.clone(),
            name_b: b.name.clone(),
            total_steps: total,
            matching_steps: matching,
            match_rate,
            score_diff: a.cumulative_score() - b.cumulative_score(),
        }
    }

    /// Check if two recordings are identical.
    pub fn are_identical(a: &Recording, b: &Recording) -> bool {
        if a.seed != b.seed {
            return false;
        }
        if a.steps.len() != b.steps.len() {
            return false;
        }
        a.steps.iter().zip(b.steps.iter()).all(|(sa, sb)| sa.signal == sb.signal)
    }

    /// Compare signal distributions.
    pub fn distribution_diff(a: &Recording, b: &Recording) -> (i32, i32, i32) {
        let count_signals = |rec: &Recording| -> (usize, usize, usize) {
            let mut pos = 0;
            let mut neg = 0;
            let mut neu = 0;
            for s in &rec.steps {
                match s.signal {
                    TernarySignal::Positive => pos += 1,
                    TernarySignal::Negative => neg += 1,
                    TernarySignal::Neutral => neu += 1,
                }
            }
            (pos, neg, neu)
        };
        let (pa, na, nua) = count_signals(a);
        let (pb, nb, nub) = count_signals(b);
        (
            pa as i32 - pb as i32,
            na as i32 - nb as i32,
            nua as i32 - nub as i32,
        )
    }
}

/// Serialize a recording to a simple string format.
pub fn serialize_recording(rec: &Recording) -> String {
    let signals: Vec<i8> = rec.steps.iter().map(|s| s.signal.value()).collect();
    format!("{}:{}:{}", rec.name, rec.seed, signals.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(","))
}

/// Deserialize a recording from string.
pub fn deserialize_recording(s: &str) -> Option<Recording> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() != 3 {
        return None;
    }
    let name = parts[0].to_string();
    let seed: u64 = parts[1].parse().ok()?;
    let signals_part = parts[2];

    let mut rec = Recording::new(name, seed);
    if signals_part.is_empty() {
        return Some(rec);
    }
    for token in signals_part.split(',') {
        let v: i8 = token.trim().parse().ok()?;
        let signal = TernarySignal::from_value(v)?;
        rec.add_step(signal);
    }
    Some(rec)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ternary_signal_value() {
        assert_eq!(TernarySignal::Positive.value(), 1);
        assert_eq!(TernarySignal::Negative.value(), -1);
        assert_eq!(TernarySignal::Neutral.value(), 0);
    }

    #[test]
    fn test_ternary_from_value() {
        assert_eq!(TernarySignal::from_value(1), Some(TernarySignal::Positive));
        assert_eq!(TernarySignal::from_value(-1), Some(TernarySignal::Negative));
        assert_eq!(TernarySignal::from_value(0), Some(TernarySignal::Neutral));
        assert_eq!(TernarySignal::from_value(5), None);
    }

    #[test]
    fn test_recording_new() {
        let rec = Recording::new("test", 42);
        assert_eq!(rec.name, "test");
        assert_eq!(rec.seed, 42);
        assert!(rec.is_empty());
        assert_eq!(rec.len(), 0);
    }

    #[test]
    fn test_recording_add_steps() {
        let mut rec = Recording::new("test", 42);
        rec.add_step(TernarySignal::Positive);
        rec.add_step(TernarySignal::Negative);
        rec.add_step(TernarySignal::Neutral);
        assert_eq!(rec.len(), 3);
        assert_eq!(rec.steps[0].index, 0);
        assert_eq!(rec.steps[1].index, 1);
        assert_eq!(rec.steps[2].index, 2);
    }

    #[test]
    fn test_recording_signals() {
        let mut rec = Recording::new("test", 42);
        rec.add_step(TernarySignal::Positive);
        rec.add_step(TernarySignal::Neutral);
        assert_eq!(rec.signals(), vec![TernarySignal::Positive, TernarySignal::Neutral]);
    }

    #[test]
    fn test_cumulative_score() {
        let mut rec = Recording::new("test", 42);
        rec.add_step(TernarySignal::Positive);
        rec.add_step(TernarySignal::Negative);
        rec.add_step(TernarySignal::Positive);
        assert_eq!(rec.cumulative_score(), 1);
    }

    #[test]
    fn test_seed_rng_determinism() {
        let mut rng1 = SeedRng::new(12345);
        let mut rng2 = SeedRng::new(12345);
        for _ in 0..100 {
            assert_eq!(rng1.next_u64(), rng2.next_u64());
        }
    }

    #[test]
    fn test_seed_rng_different_seeds() {
        let mut rng1 = SeedRng::new(1);
        let mut rng2 = SeedRng::new(2);
        assert_ne!(rng1.next_u64(), rng2.next_u64());
    }

    #[test]
    fn test_seed_rng_zero_seed() {
        let mut rng = SeedRng::new(0);
        assert_ne!(rng.state, 0);
        let val = rng.next_u64();
        assert_ne!(val, 0);
    }

    #[test]
    fn test_seed_rng_ternary_distribution() {
        let mut rng = SeedRng::new(999);
        let mut counts = [0usize; 3];
        for _ in 0..3000 {
            match rng.next_ternary() {
                TernarySignal::Positive => counts[0] += 1,
                TernarySignal::Negative => counts[1] += 1,
                TernarySignal::Neutral => counts[2] += 1,
            }
        }
        // Should be roughly uniform
        for count in counts {
            assert!(count > 800 && count < 1200, "count: {}", count);
        }
    }

    #[test]
    fn test_seed_manager_register() {
        let mut mgr = SeedManager::new();
        mgr.register("exp1", 42);
        mgr.register("exp2", 99);
        assert_eq!(mgr.get("exp1"), Some(42));
        assert_eq!(mgr.get("exp2"), Some(99));
        assert_eq!(mgr.get("exp3"), None);
    }

    #[test]
    fn test_seed_manager_generate() {
        let mut mgr = SeedManager::new();
        let (name1, seed1) = mgr.generate();
        let (name2, seed2) = mgr.generate();
        assert_ne!(name1, name2);
        assert_ne!(seed1, seed2);
        assert_eq!(mgr.len(), 2);
    }

    #[test]
    fn test_seed_manager_remove() {
        let mut mgr = SeedManager::new();
        mgr.register("exp1", 42);
        assert_eq!(mgr.remove("exp1"), Some(42));
        assert_eq!(mgr.get("exp1"), None);
        assert!(mgr.is_empty());
    }

    #[test]
    fn test_replay_engine_generate() {
        let mut engine = ReplayEngine::new(42);
        let rec = engine.generate("test", 42, 10);
        assert_eq!(rec.len(), 10);
        assert_eq!(rec.seed, 42);
    }

    #[test]
    fn test_replay_engine_determinism() {
        let mut engine1 = ReplayEngine::new(42);
        let mut engine2 = ReplayEngine::new(42);
        let rec1 = engine1.generate("a", 42, 100);
        let rec2 = engine2.generate("b", 42, 100);
        assert!(ReplayComparison::are_identical(&rec1, &rec2));
    }

    #[test]
    fn test_replay_engine_max_steps() {
        let mut engine = ReplayEngine::new(42).with_max_steps(5);
        let rec = engine.generate("test", 42, 100);
        assert_eq!(rec.len(), 5);
    }

    #[test]
    fn test_replay_engine_replay() {
        let mut engine = ReplayEngine::new(42);
        let original = engine.generate("orig", 42, 20);
        let replayed = engine.replay(&original);
        assert!(ReplayComparison::are_identical(&original, &replayed));
    }

    #[test]
    fn test_comparison_identical() {
        let mut engine = ReplayEngine::new(42);
        let a = engine.generate("a", 42, 50);
        let b = engine.generate("b", 42, 50);
        let result = ReplayComparison::compare(&a, &b);
        assert_eq!(result.match_rate, 1.0);
        assert_eq!(result.matching_steps, 50);
    }

    #[test]
    fn test_comparison_different() {
        let mut engine = ReplayEngine::new(42);
        let a = engine.generate("a", 42, 50);
        let b = engine.generate("b", 99, 50);
        let result = ReplayComparison::compare(&a, &b);
        assert!(result.match_rate < 1.0);
    }

    #[test]
    fn test_distribution_diff() {
        let mut rec_a = Recording::new("a", 1);
        rec_a.add_step(TernarySignal::Positive);
        rec_a.add_step(TernarySignal::Positive);

        let mut rec_b = Recording::new("b", 2);
        rec_b.add_step(TernarySignal::Negative);
        rec_b.add_step(TernarySignal::Neutral);

        let (dp, dn, dneu) = ReplayComparison::distribution_diff(&rec_a, &rec_b);
        assert_eq!(dp, 2);  // 2 - 0
        assert_eq!(dn, -1); // 0 - 1
        assert_eq!(dneu, -1); // 0 - 1
    }

    #[test]
    fn test_serialize_deserialize() {
        let mut rec = Recording::new("test_exp", 42);
        rec.add_step(TernarySignal::Positive);
        rec.add_step(TernarySignal::Negative);
        rec.add_step(TernarySignal::Neutral);

        let s = serialize_recording(&rec);
        let rec2 = deserialize_recording(&s).unwrap();
        assert_eq!(rec.name, rec2.name);
        assert_eq!(rec.seed, rec2.seed);
        assert_eq!(rec.signals(), rec2.signals());
    }

    #[test]
    fn test_serialize_roundtrip() {
        let rec = Recording::new("empty", 0);
        let s = serialize_recording(&rec);
        let rec2 = deserialize_recording(&s).unwrap();
        assert_eq!(rec2.name, "empty");
        assert!(rec2.is_empty());
    }
}
