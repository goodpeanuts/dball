# Repository Guidelines

## Project Structure & Module Organization
- `src/` holds the eframe app entry points (`main.rs`, `lib.rs`, `app/`, `app.rs`) and benches (`src/bench.rs`); `benches/` contains Criterion benchmarks.
- `crates/` hosts workspace members like `dball-client` and `dball-combora`.
- `assets/` and `index.html` support the WASM build; `assets/sw.js` controls caching behavior.
- `migrations/` and `database/` contain Diesel assets; `api/` holds API-related config; `Trunk.toml` configures web builds.
- `migrations`, `database`, and `api_invalid.toml` are useful references when touching persistence or API shape.

## Build, Test, and Development Commands
- Native app: `cargo run --release`.
- Web dev: `trunk serve` (after `rustup target add wasm32-unknown-unknown`) builds and serves at `http://127.0.0.1:8080`.
- Web release: `trunk build --release` outputs `dist/`.
- CI-like suite: `./check.sh` runs cargo checks (including wasm), `cargo fmt`, `cargo clippy -D warnings`, Trunk build, cargo-deny, typos, and full tests.
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
- Attach test results (`./check.sh` or relevant subset). Include screenshots/GIFs for UI changes when feasible.

## Web & Caching Tips
- During web dev, open `http://127.0.0.1:8080/index.html#dev` to bypass service-worker caching from `assets/sw.js`.
- When shipping new assets, ensure `assets/sw.js` caches the correct `{crate}_bg.wasm` and JS bundle names if they change.
