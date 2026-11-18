# Validation

## 2026-11-09 — Horizon `1d63e0a`, both artifacts

Horizon `1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9` + horizon-mixins
`5a64af938ec86b00d53d16a408aabc0038fb3e57`, every step through
`ports/tools/build-slot.sh` with `CARGO_INCREMENTAL=0`.

| Check | Result |
|---|---|
| `cargo fmt` + `--check`, `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings |
| `cargo test --locked --offline` | 13 passed |
| `horizon.py stage` / `check` / `test` / `build` | exit 0; 0 warnings; embedded 13 passed |
| `tools/package.sh` | `.hmod` 4,497 B `9e48744c…b5f82a` (11), `.hnmod` 7,357 B `32b74e16…0f23d25` (8) |
| `tools/e2e.sh` | off-hand phase rewritten to assert the readout (the `state` mirror lags the server slot at this pin); re-run pending |

Evidence: `ports/_shared/evidence/measurements/`.
`upstream_ready.py measurements` still blocks on the one-file-per-project rule.

## 2026-11-18 — the SDK format (Horizon `0b14f8a`, mixins `e59da90`)

The retired `.hnmod` container is replaced by one flat SDK package; `src/runtime.rs` is the
retired `runtime.rs` byte-identical (`sha256 7c0a2dc8…c23ef700`) and its tests are unchanged.
Every step ran through `ports/tools/build-slot.sh` with `BUILD_LOCAL=1` (the remote runner has
no synced copy of this port yet).

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings |
| `cargo test` | **17 passed**, 0 failed (13 carried runtime + 4 glue) |
| `cargo horizon-mod check` (no file) | exit 0 — 111 API items, 4 hooks, native + `wasm32` legs |
| `cargo horizon-mod build` | exit 0 → `target/horizon-mod/measurements.hmod` |
| `cargo horizon-mod check measurements.hmod` | exit 0 — `measurements 0.2.0 (native-mod, format native-1/0b14f8a…/e59da90…, for 26.3)`, **19,536 B**, SHA-256 `a673c67ab5f2495bcd10be286063cfc4967a4a949d5b4a735f8e6c7116900cbd`, 19 entries |
| `cargo horizon-mod fits measurements.hmod` | exit 0 — `0b14f8a  made for` |
| rebuild determinism | a second `cargo horizon-mod build` produced the same SHA-256 |
| in game | **not run** — `cargo horizon-mod run` needs `horizon-launcher native-build`, and there is no launcher instance on this machine |

Evidence: `ports/_shared/evidence/measurements/sdk-{fmt,clippy,test,check,build,check-package,fits}.log`.
The retired format, its tools and its `.hnmod` are archived in
`ports/_shared/evidence/measurements/retired-hnmod-0.1.0/`.
