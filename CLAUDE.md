# AGENTS.md

You must follow this project's rules for all changes.
All code produced must remain compliant with these rules.

## Architectural principles

- Apply the Single Responsibility Principle (SRP).
  Each module, struct, function, or component must have a clear, well-defined responsibility.

- Apply the Dependency Inversion Principle (DIP).
  High-level modules must not depend directly on low-level modules.
  Both must depend on abstractions.

## Code organization

- Follow Clean Code principles.

- Each source-code file should preferably contain no more than approximately 200 lines.
  If it exceeds this limit, consider splitting it into smaller, cohesive modules.

- Each change, adjustment, bug fix, and so on must be made on a separate branch.

- Do not add comments to code without my permission.

- Functions must be small and have a single responsibility.

- Avoid deeply nested conditionals.

- Prefer:
  - early returns;
  - `match`;
  - `if let`;
  - `let else`;
  - decomposition into smaller functions;
  - types and enums to represent states explicitly.

- Avoid code duplication.
  When behavior is genuinely shared, extract an appropriate abstraction.

- Do not create abstractions prematurely.
  An abstraction should exist because it solves a real design problem,
  not merely to anticipate a possible future need.

## Subagent strategy

- The primary agent is responsible for planning, architectural decisions, implementation, and final integration.
- For complex tasks, identify independent activities that can be delegated to subagents.
- Prioritize delegating code exploration, technical investigation, test analysis, and review.
- Use up to two subagents simultaneously, according to the Codex configuration.
- Avoid delegation for simple or sequential tasks when coordination costs outweigh the benefits.
- Investigation subagents must not modify files.
- Consolidate findings before making architectural decisions.
- All agents must follow this AGENTS.md.
- Preserve the approved scope and work autonomously through completion.

## Conduct

You may be proactive within the task's scope, but never beyond it. If, during a task, you notice code that needs refactoring or a fix that was not requested, do not make that change. Only mention it in the final report so I can decide whether to include it in a future task.

## Rust

- Prefer idiomatic Rust.
- Use the type system to represent rules and states whenever possible.
- Prefer enums over boolean flags when there are multiple possible states.
- Avoid operations that assume success without checking for errors.
- For `Option<T>`, avoid `unwrap()` and `expect()` when absence is a normal case; handle it first.
- Avoid `panic!`, `unreachable!`, and `assert!` for expected runtime situations.
- Prefer explicit error handling.

- Use `Option<T>` when a value may not exist.

- Do not silence warnings without a clear justification.

## Testing

- Follow TDD principles.
- Every relevant behavior should have tests.
- Algorithms and domain rules should be testable.
- Bug fixes should, whenever possible, include a test that reproduces the issue before the fix.
- Deterministic domain functions should have unit tests.

## Quality

Before considering a change complete:

1. The project must compile without errors.
2. No new warnings should be introduced without justification.
3. Existing tests must continue to pass.
4. New behavior should have tests when applicable.
5. The code must remain simple, readable, and consistent with the architecture.

## Package versioning

Package versions must follow the Semantic Versioning 2.0.0 (SemVer) specification, using the `MAJOR.MINOR.PATCH` format.

**MAJOR:** increment when a change is incompatible with previous versions, regardless of the number or significance of features introduced.

**MINOR:** increment when new features or improvements are introduced without breaking backward compatibility.

**PATCH:** increment exclusively for backward-compatible bug fixes.

When incrementing MAJOR, reset MINOR and PATCH to zero. When incrementing MINOR, reset PATCH to zero.

Versions before `1.0.0` are considered early development and do not guarantee public API stability.

Choose the version based on the nature and impact of the changes, never merely their number or complexity.
