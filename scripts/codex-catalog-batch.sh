#!/usr/bin/env bash
# Two-wave codex orchestrator for the 212-scheme catalog.
#
# Wave 1 ("models"): one codex agent at a time closes one missing component
#   model family (diode, actuator, ic/device, ...). Runs sequentially because
#   these touch shared crates/core files; parallelizing this wave risks two
#   agents inventing incompatible models for the same primitive.
# Wave 2 ("schemes"): once a model exists, fan out N codex agents in parallel
#   git worktrees, one per catalog scheme that is now unblocked. Each agent's
#   branch is merged back sequentially so conflicts surface one at a time
#   instead of silently colliding.
#
# Usage:
#   ./codex-catalog-batch.sh models  [--parallel N] [token ...]
#   ./codex-catalog-batch.sh schemes [--parallel N] [--limit N]
#
# Requires: codex CLI, git, awk. Run from the repo root (bredboard).

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT"

LEDGER="docs/catalog/CATALOG-AUDIT-LEDGER.md"
BASE_BRANCH="$(git rev-parse --abbrev-ref HEAD)"
WORKTREE_ROOT="${WORKTREE_ROOT:-$REPO_ROOT/.codex-batch-worktrees}"
LOG_DIR="${LOG_DIR:-$REPO_ROOT/.codex-batch-logs}"
SHARED_TARGET_DIR="${SHARED_TARGET_DIR:-$REPO_ROOT/.cargo-target-shared}"
mkdir -p "$WORKTREE_ROOT" "$LOG_DIR" "$SHARED_TARGET_DIR"

MODE="${1:-}"
shift || true

PARALLEL=1
LIMIT=0
POSITIONAL=()
while [[ $# -gt 0 ]]; do
  case "$1" in
    --parallel) PARALLEL="$2"; shift 2 ;;
    --limit) LIMIT="$2"; shift 2 ;;
    *) POSITIONAL+=("$1"); shift ;;
  esac
done

die() { echo "error: $*" >&2; exit 1; }

# Row layout in $LEDGER:
# 1:pad 2:key 3:section 4:name 5:svg 6:bom 7:disposition 8:readiness 9:design 10:safety 11:build 12:impl 13:manual
ledger_rows() {
  awk -F'|' 'NR>2 && $2 ~ /^ *C[0-9A-Z-]+ *$/ {
    for (i=1;i<=NF;i++) { gsub(/^ +| +$/, "", $i) }
    print
  }' OFS='|' "$LEDGER"
}

run_agent() {
  local key="$1" prompt="$2"
  local safe_key="${key//\//-}"
  local branch="wt/catalog-batch/${safe_key,,}"
  local wt_dir="$WORKTREE_ROOT/$safe_key"
  local log_file="$LOG_DIR/${safe_key}.log"

  rm -rf "$wt_dir"
  git worktree add -q -b "$branch" "$wt_dir" "$BASE_BRANCH" 2>>"$log_file" \
    || { git branch -D "$branch" 2>/dev/null || true; git worktree add -q -b "$branch" "$wt_dir" "$BASE_BRANCH"; }

  echo "[$key] agent starting (log: $log_file)"
  if CARGO_TARGET_DIR="$SHARED_TARGET_DIR" codex exec \
      -C "$wt_dir" \
      --sandbox workspace-write \
      --skip-git-repo-check \
      -o "$wt_dir/.codex-last-message.txt" \
      "$prompt" >>"$log_file" 2>&1; then
    echo "[$key] agent finished"
  else
    echo "[$key] agent FAILED, see $log_file" >&2
  fi

  (
    cd "$wt_dir"
    if [[ -n "$(git status --porcelain)" ]]; then
      git add -A
      git commit -q -m "catalog: ${key} batch pass" || true
    fi
  )
}

merge_branch() {
  local key="$1" safe_key="${1//\//-}"
  local branch="wt/catalog-batch/${safe_key,,}"
  if git merge --no-ff -q -m "merge: ${key} batch pass" "$branch"; then
    echo "[$key] merged into $BASE_BRANCH"
  else
    git merge --abort || true
    echo "[$key] MERGE CONFLICT — left worktree at $WORKTREE_ROOT/$safe_key on branch $branch for manual resolution" >&2
    return 1
  fi

  if ! CARGO_TARGET_DIR="$SHARED_TARGET_DIR" cargo check --workspace --all-targets --locked >>"$LOG_DIR/${safe_key}.merge-check.log" 2>&1; then
    echo "[$key] POST-MERGE cargo check FAILED after merging — see $LOG_DIR/${safe_key}.merge-check.log; leaving merge commit in place for manual fix (consider 'git revert')" >&2
    return 1
  fi

  git worktree remove -f "$WORKTREE_ROOT/$safe_key" 2>/dev/null || true
}

run_batch() {
  # args: key1 prompt1 key2 prompt2 ...
  local pids=() keys=()
  while [[ $# -gt 0 ]]; do
    local key="$1" prompt="$2"; shift 2
    run_agent "$key" "$prompt" &
    pids+=("$!")
    keys+=("$key")
    if [[ "${#pids[@]}" -ge "$PARALLEL" ]]; then
      wait "${pids[@]}"
      pids=()
    fi
  done
  [[ "${#pids[@]}" -gt 0 ]] && wait "${pids[@]}"

  for key in "${keys[@]}"; do
    merge_branch "$key" || true
  done
}

# ---------------------------------------------------------------------------
# Wave 1: models
# ---------------------------------------------------------------------------
models_wave() {
  local tokens=("${POSITIONAL[@]}")
  if [[ "${#tokens[@]}" -eq 0 ]]; then
    tokens=($(ledger_rows | awk -F'|' '$7=="blocked_component"{print $8}' \
      | grep -oP 'need:\K[^,]*(,[^,]*)*' | tr ',' '\n' | sed 's/×.*//' \
      | sort | uniq -c | sort -rn | awk '{print $2}'))
  fi
  [[ "$PARALLEL" -gt 1 ]] && echo "note: forcing --parallel 1 for the models wave (shared core files)" >&2
  PARALLEL=1

  for token in "${tokens[@]}"; do
    local rows
    rows="$(ledger_rows | awk -F'|' -v t="$token" '$7=="blocked_component" && $8 ~ t {print $2": "$4}')"
    [[ -z "$rows" ]] && { echo "skip $token: no matching rows"; continue; }
    local prompt
    prompt="Read AGENTS.md and docs/roadmap/CATALOG-EXECUTION-PLAN.md first.
The catalog ledger docs/catalog/CATALOG-AUDIT-LEDGER.md lists these entries as
blocked_component on a missing '$token' model/support:
$rows

Design and implement one coherent '$token' electrical/domain model in the Rust
core (crates/core), following the architecture rules in AGENTS.md (no Bevy
dependency in core, no scripted/faked electrical outcomes, connectivity
derived from the existing contact/pin/wire model, not a second netlist).
Add property/unit tests with reproducible seeds. Do NOT implement the
individual catalog fixtures yet — only the shared model/contract they need.
When done, update the 'Component/model readiness' column for the affected
rows in docs/catalog/CATALOG-AUDIT-LEDGER.md (remove the 'need:$token' marker
once satisfied) and leave their disposition as blocked_component if a fixture
still needs to be built, or note remaining gaps."
    run_batch "$token" "$prompt"
  done
}

# ---------------------------------------------------------------------------
# Wave 2: schemes
# ---------------------------------------------------------------------------
schemes_wave() {
  local eligible
  eligible="$(ledger_rows | awk -F'|' '$7=="blocked_component" && $8 !~ /need:/ {print $2}')"
  [[ -z "$eligible" ]] && { echo "no schemes are currently unblocked (no blocked_component row without a 'need:' marker)"; return 0; }
  [[ "$LIMIT" -gt 0 ]] && eligible="$(echo "$eligible" | head -n "$LIMIT")"

  local args=()
  while read -r key; do
    [[ -z "$key" ]] && continue
    local row prompt
    row="$(ledger_rows | awk -F'|' -v k="$key" '$2==k')"
    prompt="Read AGENTS.md first. Implement catalog scheme $key end to end:
ledger row: $row
Inspect its source schematic/BOM (see the row's file paths), build the
fixture under fixtures/projects/, register it in
crates/app/src/exercise_catalog.rs (and crates/app/src/main.rs if needed),
and verify with the project's stated cargo/test commands. Record any design,
safety, or buildability finding using the codebook in
docs/catalog/CATALOG-AUDIT-LEDGER.md instead of guessing past it. Update this
scheme's row in docs/catalog/CATALOG-AUDIT-LEDGER.md (disposition, evidence
columns) to reflect the real outcome — fixture_ready only if automated
evidence actually passed."
    args+=("$key" "$prompt")
  done <<< "$eligible"

  run_batch "${args[@]}"
}

case "$MODE" in
  models) models_wave ;;
  schemes) schemes_wave ;;
  *) die "usage: $0 {models|schemes} [--parallel N] [--limit N] [tokens...]" ;;
esac
