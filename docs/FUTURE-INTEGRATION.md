# Future Integration: ternary-replay

## Current State
Provides deterministic replay of agent experiments from seeds: `Recording` captures named experiment runs with `Step` sequences (index, signal, metadata), `RecordingStore` manages multiple recordings, and `ExperimentComparison` compares recordings by signal alignment and divergence metrics.

## Integration Opportunities

### With ternary-cell (Tick History Reconstruction)
Every ternary-cell tick generates a `Step` (index = tick number, signal = ternary outcome, metadata = cell ID, surprise, energy). A `Recording` captures the full tick history of a cell grid. Replay reconstructs the grid state at any historical tick by re-executing the recorded signal sequence deterministically — essential for debugging why a cell grid converged to an unexpected state.

### With ternary-diff (Compressed History Storage)
ternary-diff compresses state sequences into diffs. ternary-replay stores full step sequences. Together: the recording stores diffs (not full states), and replay applies diffs sequentially to reconstruct any point in history. `ExperimentComparison` compares diff-compressed recordings efficiently — compute diff-of-diffs to find where experiments diverge.

### With ternary-causality (Causal Replay)
ternary-causality builds causal DAGs from event sequences. ternary-replay provides those event sequences. A replay becomes a causal investigation: "replay experiment X, but intervene at step 42 (ternary-causality's `Intervention`), and show what would have happened differently." This is counterfactual reasoning over recorded histories.

## Potential in Mature Systems
In room-as-codespace, every room (Codespace) records its tick history as a ternary-replay `Recording`. PLATO maintains a global recording store across all rooms. When a room crashes or produces anomalous results, PLATO replays the recording to diagnose the issue. Recordings also enable "time travel" — rewind a room to a prior state and fork a new experiment from that point, using ternary-diff's three-way merge to combine the fork with the original timeline.

## Cross-Pollination Ideas
- **ternary-curriculum**: Replay successful learning trajectories as curriculum examples — new agents learn from recordings of expert agents.
- **ternary-transfer**: Transfer recordings between rooms — replay a source room's history in a target room to transfer learned behavior.
- **ternary-ensemble**: Replay multiple agents' recordings simultaneously to compare strategies in an ensemble evaluation.

## Dependencies for Next Steps
- Define `CellRecording` type wrapping `Recording` with cell-specific metadata
- Add recording serialization to ternary-protocol for cross-room replay
- Implement diff-based recording compression using ternary-diff
- Build counterfactual replay engine combining with ternary-causality interventions
