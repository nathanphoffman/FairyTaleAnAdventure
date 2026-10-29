# Project instructions for Claude Code

- **Do not make git commits** (and do not push). The user makes all commits. When a piece of work is
  finished and the tests pass, leave the changes in the working tree, list what changed in a few
  lines, and suggest a one-line commit message. Only commit if the user explicitly asks in that message.
- The C to Rust port is finished and no C is built. Current status and the plan are in
  `modern_engine_plan.md`, section "Where we are" (next: the modern renderer behind `Output`; the rules
  already only see `Output`, and `game::recorder::Recorder` is a screenless one for tests). The old HANDOFF/Roadmap notes in section 4a are history.
- `csrc/` is the reference and is not edited or built.
- Check a change with `cargo build`, `cargo clippy`, `cargo test`, `tests/regress.sh` and
  `tests/state_regress.sh` (run the last two in the background, one at a time, never rebuild while
  they run; every scenario must say `ok`).
