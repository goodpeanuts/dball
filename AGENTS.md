# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the eframe app entry points (`main.rs`, `lib.rs`, `app/`, `app.rs`) and benches (`src/bench.rs`); `benches/` contains Criterion benchmarks.
- `crates/` hosts workspace members like `dball-client` and `dball-combora`.
- `assets/` stores application icons and static resources used by the native GUI.
- `migrations/` and `database/` contain Diesel assets; `api/` holds API-related config.
- `migrations`, `database`, and `api_invalid.toml` are useful references when touching persistence or API shape.

## Build, Test, and Development Commands
- Native app: `cargo run --release`.
- Prefer test/run aliases defined in `.cargo/config.toml` (e.g. `cargo tpc`, `cargo tpcq`, `cargo ctenv`, `cargo nt`) for consistent local workflows.
- CI-like suite: `./check.sh` runs cargo checks, `cargo fmt`, `cargo clippy -D warnings`, typos, and full tests.
- `cargo deny check -d` is temporarily disabled in `check.sh`; re-enable after Rust toolchain/cargo-deny upgrade (tracked in `subm/xxdoc/TODO.md`).
- After each code change, run `./check.sh` for quick regression coverage before handing off.
- Quick tests: `cargo test --workspace --all-targets --all-features`; doc tests via `cargo test --workspace --doc`; optional speedup with `cargo nextest`.

## Coding Style & Naming Conventions
- Rust 2024 edition; follow standard Rust casing (snake_case for functions/modules, PascalCase for types) and avoid `unsafe` (denied in workspace lints).
- Format with `cargo fmt`; lint with `cargo clippy --all-targets --all-features -D warnings` before pushing.
- Prefer explicit modules and small, focused files; keep GUI code in `src/app/` and shared logic in crates under `crates/`.

## Testing Guidelines
- Add targeted unit tests near the code under `src/` or crate-specific test modules; integration tests live alongside crates.
- For coverage, use `cargo llvm-cov --all-features --all-targets --workspace --html` (optionally `--open`).
- Benchmarks use Criterion (`cargo bench`); avoid running them in CI unless needed.

## Commit & Pull Request Guidelines
- Use Conventional Commits (see `cliff.toml`), e.g., `feat(ui): add scoreboard` or `fix(api): handle null token`.
- Keep PRs focused; include a short description, linked issues, and noteworthy decisions.
- Keep git hooks enabled with `pre-commit install`; before any `git add/commit/push`, run `pre-commit run --all-files` for quality gates.
- Attach test results (`./check.sh` or relevant subset). Include screenshots/GIFs for UI changes when feasible.
