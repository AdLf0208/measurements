# Measurements — a Horizon port

**Measurements** is a Minecraft mod. Allows measuring distance using a tape measure. This repository is a **port of it to [Horizon](https://github.com/horizon-rust/horizon)**, the Rust engine for Minecraft Java 26.3. It carries the complete source of the port plus the content files it ships, and installs as a single `.hmod` package.

Upstream: [Mrbysco/Measurements](https://github.com/Mrbysco/Measurements) — licence **MIT**.

| | |
|---|---|
| Upstream mod | Measurements ([source](https://github.com/Mrbysco/Measurements)) |
| Ported to | Horizon 0b14f8a, Minecraft Java 26.3, mod API `0.2` |
| Package | `measurements.hmod` (install it with the Horizon launcher) |
| Licence | **MIT** — the upstream mod's licence; see [Licence and credit](#licence-and-credit) |

## What this port implements

- **Item** `measurements:tape_measure`: right-click a block to place point A,
- **Distance** is Euclidean over the coordinate differences; the distance is
- **Next click** after a finished measurement starts a new point A.
- **Shift-right-click** removes the **latest** measurement (upstream's
- **Same block twice** reports `Distance: 0.0 blocks (X: 0, Y: 0, Z: 0)`.
- **Reset**: the session is dropped when the tape is no longer in the main
- **Reviewer:** truncated description, split bullets
- **Reviewer:** relative source link

## Not yet ported

- **Delivery is the action bar, not an in-world box.** Upstream is a
- **Added distance and integer deltas.** The port prints the Euclidean
- **Per-player server sessions.** Upstream keeps one process-wide client list
- **Shift-click removes the latest measurement, not necessarily all.** This
- **The "still holding" reset checks the main hand.** Upstream's
- **Recipe not hand-crafted in game.** `horizon-mod check` accepts the recipe
- Not carried: config (line/text colours, text size, line width), in-world
- **No in-game run since the migration.** The pre-SDK e2e (default + `COUNTERFACTUAL=1`, both

## Installing

1. Install the [Horizon launcher](https://github.com/horizon-rust/horizon/releases) (0.2.70 or newer).
2. Add `measurements.hmod` to the instance's mods.
3. The package is pinned to a specific Horizon build; the launcher checks that pin.

## Licence and credit

Measurements was created by its upstream authors; the code, models and assets in the original mod
remain theirs and are used here under **MIT**. The Rust in this repository is a clean-room
rewrite for Horizon, not a translation of the original source.

- `LICENSE` — the upstream licence text.
- `LICENSE-NOTICE.md` — what this port carries and where it came from.
- The upstream project: https://github.com/Mrbysco/Measurements

## Technical details

```
horizon-mod.toml   package manifest (id, version, api, [native] pin)
Cargo.toml         the crate; depends on `horizon-mod-api` from the fetched SDK
src/               lib.rs (the one `impl Hooks`), runtime.rs (pure logic + tests), glue
assets/ data/      the content the package ships
tools/             content generators and the e2e scenario
```

- Build: `cargo horizon-mod build` (packages; it does not compile against the game).
- Test: `cargo test`, `cargo clippy --all-targets -- -D warnings`.
- In-game: `ports/tools/launcher-game.sh run measurements` in the campaign tree, or install the
  `.hmod` and play; the harness scenario is `tools/e2e.sh`.

This repository is generated from the campaign's port at `ports/measurements/`; the port's own
`notes.md` is the full record (decisions, traps, evidence).
