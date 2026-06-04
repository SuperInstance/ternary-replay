# ternary-replay: Deterministic recording and replay of ternary experiments

Record sequences of ternary signals (positive/negative/neutral), serialize them, replay them from a seed, and compare recordings for bit-exact reproducibility.

## Why This Exists

When you're running agent experiments with ternary decisions, you need reproducibility. "Why did the agent choose X at step 47?" requires being able to replay the exact same sequence. This crate provides seeded deterministic RNG, recording data structures, replay engines, comparison tools, and a simple serialization format so you can store, retrieve, and verify experiment runs.

## Core Concepts

- **TernarySignal** — A value that is `Positive` (+1), `Negative` (−1), or `Neutral` (0). The fundamental unit of an experiment step.
- **Recording** — A named, seeded sequence of steps. Each step has an index, a signal, and optional metadata (key-value string pairs).
- **SeedRng** — A deterministic pseudo-random number generator using xorshift64. Same seed always produces the same sequence. Given the same seed, two `SeedRng` instances will produce identical ternary signals forever.
- **SeedManager** — A name-to-seed registry. Generate, register, look up, and remove seeds by name.
- **ReplayEngine** — Given a seed, regenerates the exact same ternary signal sequence. Can also replay an existing recording by resetting to its seed.
- **ReplayComparison** — Compare two recordings step-by-step: match rate, cumulative score difference, signal distribution differences.
- **Serialization** — Recordings serialize to `"name:seed:v1,v2,v3,..."` format and deserialize back. Values are comma-separated i8 representations of signals.

## Quick Start

```toml
# Cargo.toml
[dependencies]
ternary-replay = "0.1"
```

```rust
use ternary_replay::*;

// Generate a recording from a seed
let mut engine = ReplayEngine::new(42);
let recording = engine.generate("experiment_alpha", 42, 100);
println!("Generated {} steps, score: {}", recording.len(), recording.cumulative_score());

// Serialize and deserialize
let encoded = serialize_recording(&recording);
let decoded = deserialize_recording(&encoded).unwrap();
assert!(ReplayComparison::are_identical(&recording, &decoded));

// Replay from the same seed produces identical results
let replayed = engine.replay(&recording);
assert!(ReplayComparison::are_identical(&recording, &replayed));

// Compare two different runs
let other = engine.generate("experiment_beta", 99, 100);
let comparison = ReplayComparison::compare(&recording, &other);
println!("Match rate: {:.1}%", comparison.match_rate * 100.0);
```

## API Overview

| Type / Function | What it is |
|---|---|
| `TernarySignal` | Enum: `Positive`, `Negative`, `Neutral` |
| `Step` | One experiment step: index, signal, metadata |
| `Recording` | Named, seeded sequence of steps |
| `SeedRng` | Deterministic xorshift64 RNG producing ternary signals |
| `SeedManager` | Name-to-seed registry |
| `ReplayEngine` | Generates/replays recordings from seeds |
| `ReplayComparison` | Static methods: `compare`, `are_identical`, `distribution_diff` |
| `ComparisonResult` | Match rate, score difference, step counts |
| `serialize_recording` | Recording → `"name:seed:values"` string |
| `deserialize_recording` | String → `Option<Recording>` |

## How It Works

**SeedRng.** Uses xorshift64: state is a single `u64`. Each call to `next_u64` applies three shift-xor operations. `next_ternary` takes the result modulo 3 to map to one of the three signals. Seed 0 is remapped to 1 internally (xorshift requires nonzero state).

**Recording.** Steps are appended in order with auto-incrementing indices. `cumulative_score` sums the i8 values of all signals. `signals()` extracts just the signal sequence as a flat vector.

**ReplayEngine.** `generate` creates a new recording by resetting the RNG to the given seed and drawing `steps` ternary signals. `replay` takes an existing recording, resets to its seed, and regenerates up to `max_steps` (default: unlimited). If `max_steps` is set via `with_max_steps()`, only that many steps are generated.

**Comparison.** `ReplayComparison::compare` does a pairwise comparison of signals at each step index, computing match rate (matching / total) and cumulative score difference. `are_identical` checks seed equality, length equality, and signal equality. `distribution_diff` returns the count differences for each signal type as `(positive_diff, negative_diff, neutral_diff)`.

**Serialization.** Format: `"<name>:<seed>:<v1>,<v2>,<v3>,..."` where each value is −1, 0, or 1. Empty recordings serialize as `"name:seed:"`. Deserialization splits on `:`, then parses the comma-separated values.

## Known Limitations

- **xorshift64 is not cryptographically secure.** Do not use `SeedRng` for security-sensitive purposes. The sequence is predictable given any output.
- **Replay regenerates from seed, not from stored steps.** If you modify a recording's steps manually and then replay it, the replay will produce the original seed-based sequence, not your modified version. Replay is seed-driven, not snapshot-driven.
- **No compression.** Serialization is plain text. A 1,000,000-step recording is roughly 3–4 MB. For large recordings, apply external compression.
- **Metadata is lost in serialization.** `serialize_recording` only preserves name, seed, and signal values. Step metadata (`HashMap<String, String>`) is not included in the serialized format.
- **Step-level metadata comparison not supported.** `ReplayComparison` compares signals only, not metadata. Two recordings with different metadata but identical signals are considered "identical."

## Use Cases

- **Experiment reproducibility.** Record an agent's ternary decisions, serialize the recording, and share the seed + name. Anyone can replay and verify the results.
- **Regression testing.** Store a known-good recording. After code changes, replay from the same seed and compare. If `are_identical` returns false, something changed.
- **A/B testing.** Run two agents with different seeds, compare match rates and score differences. `distribution_diff` tells you if one agent favors positive signals over the other.

## Ecosystem Context

Used by `ternary-scheduling` (replay scheduled task outcomes), `ternary-metrics` (replay performance data), and `ternary-validation` (replay strategies against validation rules). No dependencies on other ternary crates.

## License

MIT
