# Working on this native Horizon mod

This port is in the **SDK format** (API 0.2, 2026-11-18). One flat package: `horizon-mod.toml`,
`Cargo.toml`, `src/lib.rs` with the one `impl mod_api::Hooks`, `src/runtime.rs` and the
`assets/`/`data/` content. The retired `.hnmod` layout (mixins, compatibility.json,
horizon-mixin.toml, per-hook helper files) is archived in
`../../_shared/evidence/measurements/retired-hnmod-0.1.0/`; do not reintroduce it.

- `src/runtime.rs` is the carried pure logic and must stay **byte-identical** to the archived
  copy (`sha256 7c0a2dc8…c23ef700`); its tests run with `cargo test` alone.
- Glue belongs in `src/lib.rs`: typed `Hooks` methods and Parts, no game-internal reach.
- Read the campaign root `AGENTS.md` and `../../_shared/AGENT_MOD_STRUCTURE.md` §0 before
  editing.

Builds and tests go through the campaign build slots (never a bare `cargo` for a campaign
step), so the box is not overloaded:

```sh
cd ../..    # the campaign root
BUILD_LOCAL=1 ports/tools/build-slot.sh run --job sdk-measurements --port ports/measurements \
    --step check -- bash -c 'cd ports/measurements && cargo horizon-mod check'
ports/tools/build-slot.sh run --job sdk-measurements --port ports/measurements \
    --step test -- bash -c 'cd ports/measurements && cargo test'
```

Validate before handing back: `cargo fmt --all -- --check`, `cargo clippy --all-targets -- -D
warnings`, `cargo test`, `cargo horizon-mod build`, `cargo horizon-mod check
measurements.hmod`. Keep `notes.md`'s verification table honest and do not claim an in-game
behaviour without a `horizon-ctl` transcript.
