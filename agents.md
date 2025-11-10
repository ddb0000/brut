# Agents Playbook

## Purpose
Coordinate the small set of human (or automated) contributors who keep brut moving. The product is a single-binary terminal editor written in Rust (`src/main.rs`), so every role shares the same source of truth and must be explicit when handing off work.

## Shared Context
- Core dependencies: crossterm for terminal I/O and ropey for the text buffer.
- Primary workflows: launch with or without a filename, edit characters, navigate with arrows, press Esc to trigger the save prompt, and write changes to disk.
- Definition of done: behavioral parity with the PRD plus updated docs/tests when behavior changes.

## Agents

### Product (PRD Steward)
- **Inputs**: roadmap goals, incident reports from operators, usability feedback, and repository issues.
- **Responsibilities**: keep `prd.md` current, break initiatives into prioritized user stories, define acceptance criteria for navigation, editing, persistence, and UX polish, and call out non-goals early.
- **Outputs**: ordered backlog items with clear success metrics and any constraints (performance ceiling, portability, etc.).

### Engineering (Rust Implementation)
- **Inputs**: prioritized stories plus the current code in `src/main.rs` and `Cargo.toml`.
- **Responsibilities**: implement features with idiomatic Rust, preserve raw-mode safety (always leave the terminal in a clean state), keep the rope invariant valid, and document any non-obvious state (cursor tracking, save prompt buffering).
- **Practices**: use `cargo fmt`/`cargo clippy` before proposing changes, prefer small focused functions, and guard terminal mutations behind helper routines where practical.
- **Outputs**: tested patches, updated docs (README, usage snippets), and notes for QA about observable changes.

### QA (Manual + Automation)
- **Inputs**: release candidates or feature branches plus acceptance criteria from Product.
- **Responsibilities**: exercise the manual matrix (launch without args, edit and save existing file, cancel save prompt, confirm save prompt) on at least one platform per change, watch for cursor glitches and buffer corruption, and log reproduction steps for any regressions.
- **Outputs**: pass/fail reports, risk callouts (e.g., areas not covered), and verification notes that can be attached to release tags.

### Docs & Release Engineering
- **Inputs**: merged features, QA sign-off, and metrics.
- **Responsibilities**: update onboarding/usage docs, summarize new capabilities in release notes, tag versions (goal: `v0.1.0` milestone), and ensure binaries are reproducible.
- **Outputs**: changelog entries, updated README snippets, and published tags or binary artifacts.

## Handoff Rhythm
1. Product refines backlog against the PRD and posts acceptance criteria.
2. Engineering estimates and implements, coordinating with Product on any scope tradeoffs.
3. QA validates the implemented behavior against criteria and the regression matrix.
4. Docs/Release capture the delta, publish notes, and cut tags.
5. Retrospectives feed back into Product for the next iteration.

## Communication Guardrails
- Keep status lightweight: short async updates after each phase (planning, coding, testing, release).
- Flag risks immediately (performance regressions, terminal portability, save-prompt UX) so Product can redirect scope.
- Record any additions to the manual test matrix directly in `prd.md` to keep all roles aligned.
