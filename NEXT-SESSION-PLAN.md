# Handoff: optimize + resume catalog "schemes" batch wave

Context ran out in the previous session. This file is the handoff plan.
Delete it once the work below is done and committed (it's not meant to be
permanent project documentation).

## Where things stand

- All model waves (other/diode/module/board/ic-device) are done and merged
  into `main`. No `need:*` tokens remain in the ledger.
- A **trial run** of the schemes wave (`--parallel 5 --limit 10`) completed
  and all 10 branches are merged into `main` (commits `ba82862`..`ed6b8fc`).
  Verification passed: `cargo fmt --all --check`, `cargo clippy --workspace
  --all-targets --locked -- -D warnings`, `cargo test --workspace --locked`
  (54 app + 141 core + 3 tools), native + WASM builds.
- Current true ledger disposition counts (recomputed from the table, not the
  prose summary): `fixture_ready=72, blocked_component=74, blocked_design=1,
  out_of_scope=0, physical_scope=65` (sums to 212).
- **73 schemes remain eligible** for the schemes wave (`blocked_component`
  with no `need:` marker). Verify with:
  ```
  awk -F'|' 'NR>2 && $2 ~ /^ *C[0-9A-Z-]+ *$/ { for (i=1;i<=NF;i++) gsub(/^ +| +$/,"",$i); print }' OFS='|' docs/catalog/CATALOG-AUDIT-LEDGER.md \
    | awk -F'|' '$7=="blocked_component" && $8 !~ /need:/ {c++} END{print c}'
  ```
- `.codex-batch-logs/` and `.codex-batch-worktrees/` are still on disk
  (untracked, gitignored is NOT set up for them — check before assuming
  they're safe to delete; they may be large, `du -sh` first). Leftover
  per-worktree `target/` dirs were ~15-17GB each during the trial.

## Problems found in the trial run (fix before the full run)

1. **`.codex-last-message.txt` scratch file** written by `codex exec -o` into
   each worktree got committed (`git add -A` in `run_agent()`) and caused a
   guaranteed conflict on every merge (same path, different content, every
   branch). **Already partially fixed**: it's now in `.gitignore` on `main`
   (commit `7156107`), so future worktrees branched from current `main`
   should no longer stage it via `git add -A`. Confirm this actually holds
   before the full run (branch a throwaway worktree, run `codex exec -o
   .codex-last-message.txt ...` or just touch the file + `git add -A` +
   check it's not staged).

2. **Real conflicts in shared files** — `crates/app/src/exercise_catalog.rs`,
   `crates/app/src/main.rs`, `docs/catalog/CATALOG-AUDIT-LEDGER.md`, and
   `docs/tasks/C04-catalog-implementation.md` all get touched by every agent
   (each inserts a new const/enum variant/match arm/list entry, often at the
   same textual insertion point as siblings). With `--parallel 5` this means
   9 of 10 merges conflicted in the trial. These are additive conflicts
   (never mutually exclusive), but resolving them ate a large fork with ~280k
   tokens and ~277 tool calls for just 7 branches. At 73 remaining schemes
   this will NOT scale as manual/delegated conflict resolution.

   **Fix options to implement** (pick based on judgment, or combine):
   a. Reduce merge conflict surface: after each agent finishes in its
      worktree but before merging, `git rebase main` (or `git merge main`)
      the worktree branch onto latest main INSIDE `run_agent()`, resolving
      trivial conflicts there (still additive) — this moves conflict
      resolution earlier and per-branch instead of batched, but doesn't
      reduce total conflict count.
   b. Serialize the *merge* step strictly with `PARALLEL=1` equivalent
      behavior it already has (`merge_branch` is already sequential in
      `run_batch`) — this is already true, the conflicts are inherent to
      insertion-point collisions, not to parallelism of the merge step
      itself. The real fix is smaller batches: lower `--limit` per
      invocation (e.g. run 5-10 schemes at a time, verify, repeat) rather
      one massive 73-scheme run, so a conflict-resolution pass (manual or
      forked) stays small each time.
   c. Consider tweaking the agent prompt (in `schemes_wave()` in
      `scripts/codex-catalog-batch.sh`) to have each agent append its new
      list/match-arm entries in a more merge-friendly way — e.g. explicitly
      instruct: "add new entries in alphabetical/key order matching
      surrounding entries" (reduces semantic conflicts, but textual
      conflicts on the same insertion line are still likely with parallel
      agents — this only helps human/agent readability during resolution,
      not conflict rate).
   d. Add a cheap post-merge sanity check inside `merge_branch()` (after a
      successful, non-conflicted merge) — e.g. `cargo check --workspace
      --locked` — so structural mistakes (like the stale `Circuit::all()`
      array size / stale count assertion found in the trial) are caught
      immediately per-branch instead of accumulating across 7+ merges before
      the final full verification catches them. This was the actual bug
      that slipped through: clippy didn't catch it, only `cargo test` did.

3. **Build/test cost per worktree** — each worktree recompiles Bevy from
   scratch (~15-17GB target dir, ~15-20+ min just to build). With 12 cores /
   23GB RAM, `--parallel 5` caused CPU/RAM contention (only ~2GB free RAM
   during the trial) and one batch of 5 took ~65+ minutes instead of the
   ~18 minutes the first batch took.

   **Fix**: share `CARGO_TARGET_DIR` across all worktree builds. In
   `run_agent()` in `scripts/codex-catalog-batch.sh`, before invoking
   `codex exec`, export:
   ```bash
   export CARGO_TARGET_DIR="$REPO_ROOT/.cargo-target-shared"
   ```
   (or set it via a `.cargo/config.toml` `[build] target-dir` that both the
   base repo and worktrees pick up — env var is simpler and scoped to the
   script). This lets crates.io deps (bevy et al.) build once and be reused
   by every worktree; only `crates/core`/`crates/app` recompile per agent.
   Caveat: cargo takes a lock on the target dir per invocation, so fully
   concurrent builds of the *same* crate will serialize on that specific
   compile unit — expected and fine, still much faster than N independent
   full Bevy builds.

   Optional additional speedups (lower priority, evaluate cost/benefit):
   - Install `sccache`, set `RUSTC_WRAPPER=sccache` — complements shared
     target dir, useful if the shared-target-dir approach has issues.
   - Install `mold` or `lld` as the linker (via `.cargo/config.toml`
     `[target.x86_64-unknown-linux-gnu] rustflags = ["-C",
     "link-arg=-fuse-ld=mold"]`) — cuts link time for the many test
     binaries.
   - Lower `--parallel` from 5 to 3 to match available RAM/CPU headroom
     once the shared target dir is in place (re-evaluate after measuring;
     shared cache may make 5 viable again since redundant compilation goes
     away).

4. Clean up disk usage from the trial: `.codex-batch-worktrees/*/target`
   directories (many GB each) should be removed once their branches are
   merged (the script's `merge_branch()` already does `git worktree remove
   -f`, which should delete the worktree's `target/` too — confirm this
   actually reclaimed the disk space, `du -sh` before assuming it's fine).

## Plan for this session

1. Read `scripts/codex-catalog-batch.sh` and confirm current state matches
   this file's description (things may have moved since this was written).
2. Implement the shared `CARGO_TARGET_DIR` change (item 3) — highest
   value/effort ratio.
3. Implement the post-merge `cargo check --workspace --locked` sanity step
   inside `merge_branch()` (item 2d) — cheap, catches structural breakage
   immediately per-branch instead of at the end.
4. Decide on `--parallel` and `--limit` for the full run based on judgment
   (start conservative, e.g. `--parallel 3 --limit 10`, verify clean, then
   scale up) rather than attempting all 73 in one shot — the trial showed
   conflict-resolution cost scales with batch size, so smaller batches with
   verification between them de-risk this even if it takes more invocations.
5. Run the schemes wave in the background (this is a long-running,
   multi-hour task — treat it as such: kick off a batch, verify + merge,
   commit, repeat). Push to `origin/main` only when the user asks (per
   AGENTS.md and this project's established workflow — do not push
   automatically).
6. Before running: `git status --short` must be clean except the untracked
   `.codex-batch-logs/`/`.codex-batch-worktrees/` dirs (or intentionally
   cleaned up first) — do not run the batch script on top of uncommitted
   work.
7. Delete this file (`NEXT-SESSION-PLAN.md`) once its contents are acted on
   or superseded, so it doesn't linger as stale documentation.
