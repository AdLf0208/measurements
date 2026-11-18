# Measurements for Horizon

A clean-room Rust port of **Measurements 3.0.3** (`Mrbysco/Measurements`,
MIT) — the tape measure: right-click point A, right-click point B, read the
Euclidean distance and per-axis deltas in the action bar; shift-right-click
removes the latest measurement.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).

## Layout (the SDK format, API 0.2)

```
horizon-mod.toml           the only manifest and the only identity (api 0.2,
                           [native] pinned to Horizon 0b14f8a / mixins e59da90)
Cargo.toml, Cargo.lock     the mod's crate; horizon-mod-api comes from .horizon/sdk
src/lib.rs                 the one `#[mod_api::horizon_mod] impl Hooks`, the tape
                           measure's ItemPart, and the per-player session map
src/runtime.rs             pure model: points, deltas, session, readout (no
                           mod_api types) — carried unchanged from the .hnmod port
assets/measurements/       item definition, model, MIT texture, lang
data/                      the tape measure item, recipe, creative tab, and the
                           `c:ingots/iron` tag the recipe names
measurements.hmod          the built package (the deliverable)
notes.md                   semantics, the migration recipe, verification, gaps
```

## Commands

```sh
# the crate, against the SDK fetched into .horizon/sdk
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test

# the package: build, then validate what the registry will read
cargo horizon-mod build
cargo horizon-mod check target/horizon-mod/measurements.hmod
cargo horizon-mod fits  target/horizon-mod/measurements.hmod
```

The port's ladders run through the campaign's build slots
(`ports/tools/build-slot.sh run --job … --port ports/measurements --step …`),
so a build never shares the box with another job.

## In game

The launcher builds the game against the mod
(`horizon-launcher native-build . --instance <instance> --trust-native-code`);
the recorded pre-SDK scenario (a headless world driven by `horizon-ctl`) is kept
in `notes.md` and in `../_shared/evidence/measurements/`. It has not been
re-run since the migration: no launcher instance exists on this machine.

## Minecraft 26.3

All asset lookups and runs use `26.3.zip` in the campaign root and the prepared
root `/home/adlf/Desktop/horizon/.local/horizon-client/26.3`; never a 1.21.1
install.
