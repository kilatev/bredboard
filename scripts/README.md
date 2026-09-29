# codex-catalog-batch.sh

Two-wave Codex CLI orchestrator for closing out the 212-scheme catalog
(`docs/catalog/CATALOG-AUDIT-LEDGER.md`). Runs `codex exec` agents in
isolated git worktrees and merges their branches back one at a time.

Requires: `codex` CLI on PATH, `git`, `awk`. Run from anywhere inside the
repo — it `cd`s to the repo root itself.

## Why two waves

Most unfinished catalog rows aren't independent per-scheme work. They're
blocked on a small number of missing shared component models (diode,
actuator, ic/device, ...) in `crates/core`. Fanning agents out per-scheme
before those exist risks several agents inventing incompatible models for
the same primitive in shared files.

- **Wave 1 (`models`)**: one agent at a time, each closes one missing
  component-model family. Always runs at concurrency 1 (core files are
  shared) even if you pass `--parallel`.
- **Wave 2 (`schemes`)**: once a model exists, fan out N agents in parallel,
  one per catalog scheme that's now unblocked (fixture + registration work,
  much lower conflict risk).

## Usage

```bash
# Wave 1 — close missing component models, most-referenced first.
# No arguments = auto-detect tokens from the ledger's "need:" markers.
./scripts/codex-catalog-batch.sh models

# Or target specific tokens only:
./scripts/codex-catalog-batch.sh models diode actuator

# Wave 2 — implement schemes that are now unblocked (no more "need:" marker).
# Re-run this repeatedly as each models pass unblocks more schemes.
./scripts/codex-catalog-batch.sh schemes --parallel 5 --limit 20
```

Flags:
- `--parallel N` — max concurrent agents (schemes wave only; forced to 1 in
  the models wave).
- `--limit N` — cap how many eligible schemes to take in one `schemes` run.

## What it does per agent

1. Creates a git worktree on branch `wt/catalog-batch/<key>` off the branch
   you're currently on.
2. Runs `codex exec -C <worktree> --sandbox workspace-write` with a prompt
   telling the agent to read `AGENTS.md` first, then do the model or scheme
   work and update the ledger row itself.
3. Commits any changes in the worktree (local commit only — never pushes).
4. After the batch finishes, merges each branch back one at a time with
   `git merge --no-ff`.
5. On a merge conflict: aborts that merge, leaves the worktree and branch
   in place for manual resolution, and keeps going with the rest of the
   batch instead of stopping everything.

Logs land in `.codex-batch-logs/<key>.log`; worktrees live under
`.codex-batch-worktrees/<key>` until merged (or left behind on conflict).
Both directories are working state, not repo content — add them to
`.gitignore` if you keep them around, or delete them once merged.

## First run

Don't start with `--parallel 5` on the full backlog. Run `models` first
(it's the risky, architecture-deciding wave), then try `schemes --limit 1`
or `--limit 2` and read the log before scaling up `--parallel`.

## Caveats

- The script never pushes and never touches the base branch's history
  except via local merges — review `git log` before pushing further.
- It trusts each agent to update its own ledger row honestly (disposition,
  evidence columns). Spot-check a few rows after a batch; nothing here
  re-verifies the agent's claims against `docs/roadmap/CATALOG-EXECUTION-PLAN.md`'s
  "no scripted outcomes" rule for you.
- Ledger parsing assumes the current table shape (13 `|`-delimited columns,
  audit key in column 2). If the ledger format changes, `ledger_rows()` in
  the script needs updating.
