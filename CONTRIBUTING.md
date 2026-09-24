# Contributing

The project is in its workspace stage. Read [the plan](docs/PLAN.md) and choose an explicitly assigned task from [the task index](docs/TASKS.md). Each task is intended to produce one independently verifiable change.

## Working on a task

1. Read its dependencies, boundaries, and acceptance criteria.
2. Implement only that outcome. Keep the core independent of Bevy and use English for project artifacts.
3. Add focused verification with the behavior, including property tests where specified.
4. Run the project's documented checks and the task-specific checks. Record exact commands, results, and any limitations in the task card.
5. Review the diff against the task and architecture. Set `ready_for_fukit` only when every required check passes.

The [README](README.md) lists the workspace build and check commands. Add focused tests as behavior is introduced, and run the task's required platform checks.

## Review and commits

Keep each change coherent and describe its concrete behavior in English. Suggested commit titles in task cards are defaults, not substitutes for describing the final implementation.

The project owner uses Jujutsu and the personal `fukit` skill for plan-compliance review, code review, checks, fixes, commit, and targeted push. That workflow requires an existing jj repository and an established remote/bookmark. Loading its instructions is not authorization to publish.

Other contributors do not need that personal skill. They should provide the same acceptance evidence and follow the repository's contribution destination once established. Do not add unrelated changes, hide skipped checks, or rewrite published history as part of a task.

## Dependencies, assets, and language

Record the origin and license of dependencies and assets when adding them. Use redistributable materials and preserve attribution obligations. Keep UI and lesson text separate from code so translations can be added later.

The project is hosted at [kilatev/bredboard](https://github.com/kilatev/bredboard) under the [MIT License](LICENSE). Preserve its copyright notice and review third-party obligations separately.

Use repository-local contributor identity so personal and corporate projects can coexist. Set your own values with `git config --local user.name`, `git config --local user.email`, `jj config set --repo user.name`, and `jj config set --repo user.email`. Do not change global identity settings as part of this project's setup.
