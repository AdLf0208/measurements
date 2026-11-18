# Measurements — port notes

Ported from **Measurements 3.0.3** (`Mrbysco/Measurements`, **MIT**;
`gradle.properties` `version=3.0.3`, `minecraft_version=1.21.1`,
`neo_version=21.1.145`) for Minecraft Java 1.21.1 NeoForge, rewritten against
Horizon **26.3** at pin `c9d474d` / horizon-mixins `d9c83d7`.

**LOCAL-ONLY** until the maintainer approves publishing (ARCHITECTURE.md §8).
The licence is MIT, so derivatives are allowed; publication still needs the
maintainer's approval.

Status: **migrated to the SDK format (0.2.0) on 2026-11-18** — one flat
`measurements.hmod` at Horizon `0b14f8a` / mixins `e59da90`, `runtime.rs` byte-identical,
`cargo horizon-mod check` green for the native and `wasm32` legs, and 17 tests passing. The
in-game evidence below (the `measurements:tape_measure` item, a fail-capable The End e2e, the
default PASS and the stick counterfactual) is from the **retired `.hnmod` bundle** and has not
been re-run against this package: no launcher instance exists on this machine. See
§"Migration to the SDK format" for the mapping and the gaps.

## Behaviour (slice 1)

- **Item** `measurements:tape_measure`: right-click a block to place point A,
  right-click again to place point B and finish the measurement. The action bar
  shows `Distance: <d> blocks (X: <dx>, Y: <dy>, Z: <dz>)`.
- **Distance** is Euclidean over the coordinate differences; the distance is
  printed with one decimal (`5.0`, `7.1`, `0.0`) and the deltas as integers.
- **Next click** after a finished measurement starts a new point A.
- **Shift-right-click** removes the **latest** measurement (upstream's
  `undo`), reporting `Measurement undone (N left)` or `Measurement cleared`
  when none remain; the next click starts a new A.
- **Same block twice** reports `Distance: 0.0 blocks (X: 0, Y: 0, Z: 0)`.
- **Reset**: the session is dropped when the tape is no longer in the main
  hand (`player_tick`), when the player changes dimension, and when a player
  leaves (pruned at `server_tick` End; the pin has no quit hook).

## Upstream semantics (file:line in the 3.0.3 checkout)

The analysis clone was `Mrbysco/Measurements` @
`ebdb995e7ceefdb1f7e09a8e803560f27b009e1f` (2025-09-02, branch `multi/1.21`,
no release tags in the shallow clone). Files read:
`item/TapeItem.java`, `registration/MeasurementRegistry.java`,
`client/BoxHandler.java`, `client/MeasurementBox.java`,
`client/ClientClass.java`, `client/LineRenderType.java`,
`platform/NeoForgePlatformHelper.java`, `platform/FabricPlatformHelper.java`,
`config/TextColor.java`, the item model/lang and the generated recipe.

| Behaviour | Evidence |
|---|---|
| One item `measurements:tape_measure`, plain `Item.Properties()` | `MeasurementRegistry.java:21` |
| `useOn` adds the clicked pos to `BoxHandler` for a valid user | `TapeItem.java:14-23` |
| "Valid user" is client-side only (not FakePlayer) | `NeoForgePlatformHelper.java:13-15`; `FabricPlatformHelper.java:11-13` |
| One global static `List<MeasurementBox>`; not per player, not a component | `BoxHandler.java:12-13` |
| Shift-click = `undo()` (remove the latest box), not clear-all | `BoxHandler.java:15-19`, `43-47` |
| First click starts a box; second sets the end and finishes it; a click after a finished box starts a new one | `BoxHandler.java:21-37` |
| An unfinished box follows the crosshair each client tick | `ClientClass.java:27-47` |
| Clear on login/logout and whenever the tape is not held (both hands) | `ClientClass.java:19-25`, `27-32` |
| Box AABB is `min(a,b) .. max(a,b)+1`, so each side is the inclusive count `|Δ|+1` | `MeasurementBox.java:68-78` |
| Labels are the three integer axis sizes only; no Euclidean distance or chat/action-bar text | `MeasurementBox.java:118-122`, `170-172` |
| A box renders only in its own dimension | `MeasurementBox.java:85-86`, `46-49` |
| Recipe: gray wool / `#c:ingots/iron` / yellow wool, ` G / GIY / GY` | `neoforge/src/generated/resources/data/measurements/recipe/tape_measure.json` |
| Item name "Tape Measure" | `assets/measurements/lang/en_us.json:3` |

The detailed extraction is preserved in
[`../_shared/upstream/measurements/FINDINGS.md`](../_shared/upstream/measurements/FINDINGS.md)
(the Java clone itself was deleted after analysis).

## Hooks (pin `c9d474d`)

| Hook | Helper | What it does |
|---|---|---|
| `use_item_on` (v1, HEAD, API 13) | `use_item_on.rs` | With the tape: click A / finish B / shift-remove the latest, then send the action-bar line. |
| `player_tick` (v1, HEAD, API 31) | `player_tick.rs` | Drop the session while the tape is not in the main hand. |
| `server_tick` (v1, HEAD, API 4) | `server_tick.rs` | At `ServerTickEnd`, prune sessions of players who left. |

The readout uses `native_action_bar(uuid, Text)`; the acting player comes from
`native_acting_player()`. Embedded tests run as `game::measurements::tests`.

## Content (`.hmod`)

No ATM10 jar was carried for this mod (`ports/_shared/carry-work/` has none),
so the content half was assembled from the **MIT upstream repository's own**
assets and data rather than a vanilla placeholder:

- `data/measurements/horizon/item/tape_measure.json`: `template`
  `minecraft:stick` (a plain item; no block item, default stack 64 like
  upstream's plain `Item.Properties()`), `max_stack_size` not overridden.
- `assets/measurements/items/tape_measure.json` + `models/item/tape_measure.json`
  (parent `minecraft:item/generated`, layer0 `measurements:item/tape_measure2`)
  and the real 16×16 upstream texture `tape_measure2.png` (2264 B). Nothing was
  invented.
- `assets/measurements/lang/{en_us,ru_ru,tr_tr}.json` (upstream, with
  `itemGroup.measurements` added to `en_us`).
- `data/measurements/recipe/tape_measure.json` (upstream shape; NeoForge
  `#c:ingots/iron` kept and supplied by
  `data/c/tags/item/ingots/iron.json` → `minecraft:iron_ingot`).
- `data/measurements/horizon/creative_tab/tape_measure.json` lists the item.

`horizon-mod check`: `measurements 3.0.3 (mod, API 0.2, both)`, **4497 bytes**,
SHA-256 `9e48744c5ce8b529bcef6e017da46eb6caef5dd5f58d4ca73b0c41ffa2b5f82a`,
11 files, unsigned, "no code: content only". The item is `/give`-able and the
e2e held it (`held: measurements:tape_measure`); the Measurements creative tab
is declared.

## Deviations and gaps

1. **Delivery is the action bar, not an in-world box.** Upstream is a
   client-only renderer: `MeasurementBox` draws an AABB and three axis labels
   (`MeasurementBox.java:118-173`). It has no chat/action-bar readout and no
   Euclidean distance. The pin has no client render hook usable in the headless
   harness, so the port reports through `native_action_bar`, as the campaign
   brief chose. Deferred: the drawn box, its colours (line/text, `XYZRGB`),
   text size, line width, and the live crosshair preview.
2. **Added distance and integer deltas.** The port prints the Euclidean
   distance (one decimal) and coordinate differences (`|Δ|`). Upstream's labels
   are inclusive block counts (`|Δ|+1`), so the same 0→5 measurement reads 5 in
   the port and 6 in upstream's label. Recorded and intentional per the brief.
3. **Per-player server sessions.** Upstream keeps one process-wide client list
   (`BoxHandler.java:12-13`). The port keys sessions by player UUID, clears on a
   dimension change, and prunes a session one tick after a player leaves (no
   quit hook at the pin). The login/logout clear is therefore replaced by the
   prune.
4. **Shift-click removes the latest measurement, not necessarily all.** This
   is upstream's `undo` (`BoxHandler.java:43-47`). With two stored
   measurements the first shift reports `Measurement undone (1 left)` and a
   second clears; with one it reports `Measurement cleared` directly. The e2e
   exercises both.
5. **The "still holding" reset checks the main hand.** Upstream's
   `isHolding` covers both hands (`ClientClass.java:29`). An off-hand-only tape
   can start a measurement through `use_item_on` but is cleared on the next
   player tick; recorded gap.
6. **Recipe not hand-crafted in game.** `horizon-mod check` accepts the recipe
   and the supplied `#c:ingots/iron` tag; the harness did not craft it (the
   campaign's `/recipe give` probe was inconclusive for vanilla ids too).
7. Not carried: config (line/text colours, text size, line width), in-world
   rendering/preview, and the client-side `LineRenderType`. No `/measure`
   command fallback was needed: the pin registered the item and
   `use_item_on` delivered the readout (proved by the e2e).

## Verification

### Runtime (standalone crate)

| Check | Command | Result |
|---|---|---|
| fmt | `cargo fmt --all --check` | exit 0 |
| clippy | `cargo clippy --all-targets --locked --offline -- -D warnings` | `cargo_exit: 0`, **0 warnings** |
| tests | `cargo test --locked --offline` | **13 passed**, 0 failed |

### Staged pin (`stage → check → test → build`)

`HORIZON_MIXINS=…/horizon-mixins python3 tools/horizon.py <step> --horizon …/horizon`
with `CARGO_BUILD_JOBS=2` (one cargo at a time).

| Step | Result |
|---|---|
| stage | exit 0, `stage_changed: true`, revision `c9d474dd…` (3 injections, 2 modules) |
| check | `cargo_exit: 0`, **0 warnings** |
| test | `cargo_exit: 0`, **13 passed; 0 failed; 531 filtered out**, **0 warnings** |
| build | `cargo_exit: 0`, **0 warnings**, binary `target/horizon-cargo/fast/horizon-client` |

### Content

| Check | Command | Result |
|---|---|---|
| build | `horizon-mod build` (from `content/`) | exit 0, `target/horizon-mod/measurements.hmod` |
| check | `horizon-mod check target/horizon-mod/measurements.hmod` | exit 0, no code, 4497 B, SHA-256 `9e48744c…b5f82a`, 11 files |

### In-game e2e (`tools/e2e.sh`)

Creative, The End, floor at y=63, walls at y=64 along X (`100..105`) and Z
(`100..105`), player moved in front of each target; `--dev-mod content`.
Logs: driver `/tmp/opencode/measurements-e2e-driver.log`, counterfactual
`/tmp/opencode/measurements-e2e-cf-driver.log`. (The game itself writes
`/tmp/opencode/measurements-e2e.log` beside the socket, so the driver log uses
a different path.)

Default run — **exit 0**:

| Action | Action bar |
|---|---|
| click (100,64,100) with the tape | `Point A: 100 64 100` |
| click (105,64,100) | `Distance: 5.0 blocks (X: 5, Y: 0, Z: 0)` |
| click (100,64,100) again | `Point A: 100 64 100` (new measurement) |
| click (100,64,105) | `Distance: 5.0 blocks (X: 0, Y: 0, Z: 5)` |
| shift-click | `Measurement undone (1 left)` |
| shift-click | `Measurement cleared` |
| click (100,64,100) | `Point A: 100 64 100` |
| shift-click | `Measurement cleared` |
| diamond-pickaxe click (control) | unchanged (`Measurement cleared`) |
| tape click (105,64,100) | `Point A: 105 64 100` (not a distance, so the pickaxe did nothing) |

`PASS: A->B read 5.0 on X, 5.0 on Z, shift-click cleared, and the pickaxe
control did nothing`

Counterfactual run — `COUNTERFACTUAL=1 tools/e2e.sh`, **exit 0**: the identical
clicks with `minecraft:stick` produced an empty action bar every time, so the
port item is what acts.

`COUNTERFACTUAL PASS: a plain stick produced no measurement readout in any of
the same clicks`

## Skills used

- superpowers `test-driven-development` (13 failing tests observed first, then
  the implementation), `systematic-debugging` (e2e arena/chunk-load and
  shift-undo diagnosis), `verification-before-completion` (every claim above
  comes from the recorded command output).
- Beautiful Rust `rust-best-practices` (read from
  `ports/_shared/skills/BeautifulRust/skills/rust-best-practices/SKILL.md`):
  fmt/clippy `-D warnings`/tests clean, typed `Point`/`Delta`/`Outcome`,
  `Option` for the unfinished end, no bool parameters, no unused API; the two
  test-only accessors are `#[cfg(test)]` so the game build stays warning-free.

## Re-pin to Horizon `83ad384f` (in progress)

Re-pinned from `c9d474dd…` (108 hooks) to **`83ad384f48d0742746d518737511882680aab5b7`**
(427 hooks / 159 helpers), horizon-mixins `e8cd0aa38cffef49cb2e088ef716baa107148c02`,
`mixin_schema` 6, `crates/horizon-client/src/game.rs` sha256
`996432b9f7f9461fa69c90ff798daf6b89fe3795f379d7c6f401916633296463`.
`compatibility.json` and the bundle's `[compatibility] horizon_revision` match;
`tools/horizon.py` runs the embedded tests as **`--lib`** (the pin's
`src/main.rs` no longer declares `mod game;`).

Feature implemented in this pass:

**Both hands** for the "still holding" reset, and an immediate clear on quit:
`player_tick` now checks the main hand *and* the off hand
(`Game::stack_in(shield::OFFHAND_SLOT)`, the same slots `held()` uses), and a
new `player_logged_out` hook (API 57, `mixins/measurements/player_logged_out.rs`)
drops the session instead of waiting a tick for the `server_tick` prune. The e2e
gained an off-hand phase: point A with the main-hand tape, move the tape to the
off hand and empty the main hand, wait, then point B and read the distance (the
old main-hand check would have cleared the session and printed a fresh
`Point A`).

Verification status (recorded honestly, nothing claimed without output):

- `cargo fmt --all --check`: exit 0.
- `cargo clippy --all-targets --locked --offline -- -D warnings`: exit 0.
- standalone `cargo test --locked --offline`: see the counts in the note below.
- `python3 tools/horizon.py stage --horizon ports/_shared/pins/horizon-83ad384f`:
  exit 0, `stage_changed: true`, revision `83ad384f…` (the staged tree contains
  the new hooks/helpers).
- `check` / `build` / `horizon-ctl` / embedded `test` / `tools/e2e.sh`: **queued
  behind the campaign build lock** (`flock /tmp/opencode/campaign-build.lock`,
  8+ waiters from other ports) when this note was written; their result is to
  be appended before the row in `ports/ledger.csv` is advanced.

See [`REPIN-AUDIT.md`](REPIN-AUDIT.md) for the deferred-feature audit at this pin.

Class/tier per `ports/ledger.csv` and `horizon/docs/modpacks.md`: **content/logic** — approach: content + use/tick/quit hooks; both hands and the quit hook stay in the logic tier.

## Re-pin to Horizon `1d63e0a` and artifact rebuild (2026-11-09)

Mode A migration: the port pinned `83ad384f` and shipped only the `.hmod`; the
`.hnmod` half is new. The stamped hook catalog is byte-identical between the two
pins, so every injection, module and binding is unchanged; only the pin identity
moved.

| | Before | After |
|---|---|---|
| Horizon | `83ad384f48d0742746d518737511882680aab5b7` | **`1d63e0a5a37a7a708eb0b32b49bf0dfeb28ed8a9`** |
| horizon-mixins | `e8cd0aa38cffef49cb2e088ef716baa107148c02` | **`5a64af938ec86b00d53d16a408aabc0038fb3e57`** |
| `game.rs` SHA-256 | `996432b9f7f9461fa69c90ff798daf6b89fe3795f379d7c6f401916633296463` | `64813cf4db0b9c3983d42908609bb633b9fbbc705f0c890396f940a6c7cef80f` |

### Ladder (through `ports/tools/build-slot.sh`, `CARGO_INCREMENTAL=0`)

| Step | Result |
|---|---|
| `cargo fmt` + `--check` | exit 0 |
| `cargo clippy --all-targets -- -D warnings` | exit 0, **0 warnings** |
| `cargo test --locked --offline` | **13 passed**, 0 failed |
| `horizon.py stage` | exit 0 at `1d63e0a` |
| `horizon.py check` | `cargo_exit: 0`, **0 warnings** (73 s) |
| `horizon.py test` | **13 passed**, 1240 filtered |
| `horizon.py build` (fast) | `cargo_exit: 0` (506 s) |

### Delivered artifacts (reproducible)

| File | Size | SHA-256 | Entries |
|---|---:|---|---:|
| `content/measurements.hmod` | 4,497 B | `9e48744c5ce8b529bcef6e017da46eb6caef5dd5f58d4ca73b0c41ffa2b5f82a` | 11 |
| `measurements.hnmod` | 7,357 B | `32b74e16cf079dea8e7fc875e4c4f21686561ac78587ae5f9ec38939d0f23d25` | 8 |

The `.hmod` is built and checked by the pin's own `cargo-horizon-mod`
(`no code: content only`); the `.hnmod` is `tools/package.py`'s deterministic
archive. A second `tools/package.sh` run reproduced both hashes.

### In-game e2e (fast build, shared `horizon-ctl`)

First run at this pin failed the off-hand phase: *"the off hand holds none,
expected measurements:tape_measure"*. Root cause: the phase read the window's
`ctl state` off-hand mirror, which lags the server-side slot after
`item replace entity @s weapon.offhand` at this pin; the port reads the off hand
server-side. The phase now asserts the **behaviour** that names the off hand —
with the main hand emptied, a click must still finish the measurement — and
prints both accessors. `SELFTEST=1` flips the first distance readout so the
negative leg fails by design. Re-run pending a slot.

Skills: superpowers/using-superpowers,verification-before-completion,systematic-debugging; BeautifulRust/rust-best-practices

> **2026-11-10 (after this record).** A parallel campaign session converged this
> port to the registry's one-file rule by embedding the carried content in the
> `.hnmod` (`mixins/<id>/embedded_content.rs` + `mixins/<id>/data/<id>-content.hmod`).
> The `.hnmod` size/SHA-256 above is therefore from this job's split build and is
> **superseded** by that port's own later record; the pin migration, the ladder
> results and the e2e findings in this section describe the code that was tested
> here and remain the basis for the harness fixes listed.

> **2026-11-10 — embedded content; one `.hnmod` is the deliverable.**
> The built content mod (`content/measurements.hmod`) now travels inside the
> bundle at `mixins/measurements/data/measurements-content.hmod`, is compiled in
> with `horizon_client::native_data!("measurements", "measurements-content.hmod")`,
> written once to `.horizon-cache/measurements-embedded-content.hmod` and
> appended to the dev-mod list by the `embedded-content` `content_root`
> injection (HEAD, cancellable; the AE2 pattern, proved first on
> `charginggadgets`). `tools/package.py` packs `data/` and refuses an embedded
> file that differs from `content/measurements.hmod`. One installed `.hnmod`
> installs the tape measure and the code; the split pair is no longer a registry
> blocker.
>
> Artifact: `measurements.hnmod` 11,965 B, SHA-256
> `76814bdc08b21006123cbaae014e16333082e0449e429303c736afec3630af36`,
> 10 entries (including `mixins/measurements/data/measurements-content.hmod`,
> 4,497 B); `cargo horizon-mod check measurements.hnmod --kind native-mod`
> passes. `content/measurements.hmod` stays in the port as the input of that
> copy, not as a second deliverable.
>
> In-game proof with no `--dev-mod`:
> `ports/_shared/evidence/measurements/e2e-embedded.log` first runs the control
> (a plain `.horizon-cache` file: no tape item, a click produces no readout, so
> the content provably comes from the bundle), then the full scenario — a LAN
> guest rejoins with the same embedded bundle and its logout still drops the
> session — plus a byte-compare of the written file against
> `content/measurements.hmod`; it ends `PASS` + `EXIT:0`.
> `e2e-embedded-selftest.log` ends `FAIL` + `EXIT:1`.
> `check-e2e-logs.py measurements --strict` and `upstream_ready.py
> measurements` are both green. Harness fix this leg needed: after
> `disconnect` the guest is on its title screen, where `wait` refuses ("no world
> open") — the rejoin leg now sleeps in wall time and polls (errors swallowed)
> instead of aborting; the guest launch omits `--dev-mod` in embedded mode. No
> assertion was changed.

## Reviewer repairs (2026-11-11) — PR review items and what changed


Each item below is quoted from `ports/_shared/horizon-mods/reviews/pr-*-measurements.md` (the review is the specification), with what this repair did about it.

- **Reviewer:** truncated description, split bullets
  **Repair:** Rewritten: complete sentences, the action-bar deviation, the Euclidean-vs-inclusive-count difference and the main-hand reset are stated.
- **Reviewer:** relative source link
  **Repair:** Already absolute (`https://github.com/Mrbysco/Measurements`).
- **Reviewer:** licence notice
  **Repair:** `LICENSE` + `LICENSE-NOTICE.md` in the `.hnmod`; `data/measurements/license/` in the carried `.hmod`; copyright `2021 Mrbysco`.
- **Reviewer:** registry artifact delivery
  **Repair:** **Staging (orchestrator, not ours)**: the artifact is not in registry release/staging as `<sha256>.<ext>`; the orchestrator answers it with `horizon-mods#686` (no write access to the registry's releases). Every repackaged artifact below has a fresh sha256 for that re-push, and the author release must be re-uploaded from the port repository.

Skills: superpowers/using-superpowers,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices

## Migration to the SDK format (2026-11-18)

Horizon's mod-API merge (#16941, launcher ≥ 0.2.70) retired the `.hnmod` container and the
registry closed every `.hnmod` submission, this port's #708 among them. The port is now **one
flat `.hmod`** built against the SDK nightly **`0b14f8acfd97fa2c750e42795249dcc981178ca4`**
(mixins `e59da9094f0c2fdda6c627fe9eff22dec6a906f0`, launcher 0.2.78, API `0.2`). The pure
logic is unchanged: `src/runtime.rs` is the retired `mixins/measurements/runtime.rs`
**byte-identical** (`sha256 7c0a2dc80bc78dfcacbd4a2f0bd28c03d6ff71c5070b73037599e015c23ef700`),
tests and all, and they were green before any glue was written.

### Files

| Created | What it is |
|---|---|
| `horizon-mod.toml` | the only manifest: `id = "measurements"`, `version = "0.2.0"`, `api = "0.2"`, `side = "both"`, `capabilities = ["registries"]`, `[native] game = "26.3"`, `horizon = "0b14f8a…"`, `mixins = "e59da90…"` |
| `Cargo.toml`, `Cargo.lock` | package `measurements` 0.2.0, edition 2024, the one dependency `horizon-mod-api` from `.horizon/sdk/` |
| `src/lib.rs` | the one `#[mod_api::horizon_mod] impl Hooks`, the tape measure's `ItemPart`, the per-player session map |
| `src/runtime.rs` | the carried pure model, byte-identical to the retired copy |
| `assets/`, `data/` | the carried content, moved out of `content/` unchanged (item definition, model, MIT texture, three lang files, recipe, creative tab, the `c:ingots/iron` tag the recipe names) |
| `measurements.hmod` | the built package (below) |

Deleted, archived under
`../_shared/evidence/measurements/retired-hnmod-0.1.0/`: `Cargo.toml` (whose `[lib] path`
pointed into `mixins/`), `Cargo.lock`, `compatibility.json`, `rust-toolchain.toml`,
`mixins/measurements/` (`horizon-mixin.toml`, `runtime.rs`, `support.rs`, `use_item_on.rs`,
`player_tick.rs`, `server_tick.rs`, `player_logged_out.rs`, `embedded_content.rs`, and the
embedded `data/measurements-content.hmod`), `content/` (its own manifest + assets + data + the
built `.hmod`), `measurements.hnmod`, and `tools/` (`horizon.py`, `package.py`, `package.sh`,
`stamp_license.py`, `audit/`, `e2e.sh`). `AGENTS.md` and `README.md` were rewritten for the
new layout.

### Hook mapping

| Retired hook (`horizon-mixin.toml`) | This SDK | Difference |
|---|---|---|
| `use_item_on` v1 HEAD, item-scoped (`stack: &ItemStack`), helper set `claimed = true` | **`ItemPart::use_on`**, a Part attached to `measurements:tape_measure` (`#[mod_api::horizon_part]`), `UseOnEvent { stack, pos, face, hand, hit, player, player_entity_id }`, returns `Option<InteractionResult>` | the part is asked only for that item, so the id check is the attachment; the port answers `Some(InteractionResult::SuccessServer)` where it used to claim |
| `player_tick` v1 HEAD (the acting player's server tick) | **`Hooks::player_tick_post`**, `PlayerTickPostEvent { player: PlayerRef }`, `side = "both"` | it now names the ticked player instead of the acting one (better), and fires on the client too — the glue returns on `HostSide::Client` |
| `player_logged_out` v1 (`player: u128`) | **`Hooks::player_logged_out`** v2, `PlayerLoggedOutEvent { player: u128, name: &str }` | same value; the name is new and unused |
| `server_tick` v1 HEAD with `phase` | **`Hooks::server_tick`**, `ServerTickEvent { context, phase }` | same `ServerTickEnd { world_running }` shape; kept as the second line of defence beside the logout hook |
| `embedded-content` (a `content_root` HEAD injection, the AE2 pattern) | **not needed** | the flat `.hmod` carries `assets/` and `data/` itself, so the whole write-out-and-append workaround retires with the container — the registry's one-file-per-project obstacle is gone too |
| `native_action_bar(uuid, Text)` | **`PlayerRef::action_bar(host, Text)`** on `ServerHost` | same action bar, now a typed handle call; returns `Result<bool, TextFormError>`, which the port ignores like the old helper did |
| `native_acting_player()` | **`host.player(uuid)`** (a `ServerHost` method) or `host.players().acting_player()` | the click's own `player` field is used, so no acting-player dependency |
| `game.stack_in(selected)` and `crate::game::shield::OFFHAND_SLOT` | **`PlayerRef::inventory_slots(host)`** (numbered `Inventory` slots) + **`selected_slot(host)`**, off hand **40** | see trap 4: the place API numbers the off hand **-106** |
| `game.player.shift_key_down` | **`EntityRef::is_shift_key_down(host)`** ("the server's shift-key flag") | same input |
| `native_server_players()` (`{ uuid, … }` structs) | **`host.players().server_players()`** (`Vec<PlayerRef>`, `PlayerRef::uuid()`) | same list |

**Client/HUD, the risk area of this port.** The port uses **no client hook at all**: the
readout is a server `action_bar` call, and the one both-sides hook returns early on
`HostSide::Client`, so nothing is drawn or predicted twice. Nothing the old bundle did was
lost on the client side, because the old bundle used no client hook either. The drawn box
remains a recorded deviation (`Deviations and gaps` 1), **not** a new gap; the SDK does now
expose a client draw surface (`gui_layer_pre`, `render_gui_pre/post`, `ClientHost`), which
gives a later slice the headroom the pin lacked.

### Manifest mapping

| Retired | Now |
|---|---|
| `horizon-native.json`: `game`, `horizon`, `mixins`, `id`, `version`, `schema` | `horizon-mod.toml`: top-level `id`/`name`/`version`/`api`/`side`/`capabilities`, and `[native] game`/`horizon`/`mixins` |
| `[mod] id/version/priority` and `[compatibility] horizon_revision` in `horizon-mixin.toml` | the same top-level `id`/`version` and `[native] horizon`; the mixin priority has no counterpart (no injections are planned) |
| `[[module]] runtime` + `[[module]] state` | `src/runtime.rs` (`pub mod runtime`) + `src/lib.rs` in one crate |
| `[[injection]] use-item-on` / `player-tick` / `server-tick` / `player-logged-out` | four `impl Hooks`/Part methods (above) |
| `[[injection]] embedded-content` | deleted |
| `content/horizon-mod.toml` (`version = "3.0.3"`, "port-owned content mod") | merged into the one manifest, at the **port's** version `0.2.0` — the registry version is the port's, not the original mod's |

### Commands (all through `ports/tools/build-slot.sh`, `BUILD_LOCAL=1` for the reason in trap 8)

```
cargo fmt --all -- --check                                  exit 0
cargo clippy --all-targets -- -D warnings                   exit 0, 0 warnings
cargo test                                                  17 passed, 0 failed
cargo horizon-mod check                                     exit 0
cargo horizon-mod build                                     exit 0 → target/horizon-mod/measurements.hmod
cargo horizon-mod check measurements.hmod                   exit 0
cargo horizon-mod fits  measurements.hmod                   exit 0
```

`cargo horizon-mod check` (no file) compiles the crate for the native and `wasm32` targets and
reports the surface it touches:

```
Measurements uses 111 items of the API (2 Part kinds, 1 capability, 13 fields or variants, 2
functions, 4 hooks, 6 hook arguments, 2 macros, 33 methods, 14 traits, 34 types). Made for
0b14f8a.
```

`cargo horizon-mod check measurements.hmod`:

```
measurements 0.2.0 (native-mod, format native-1/0b14f8acfd97fa2c750e42795249dcc981178ca4/e59da9094f0c2fdda6c627fe9eff22dec6a906f0, for 26.3)
  19536 bytes, SHA-256 a673c67ab5f2495bcd10be286063cfc4967a4a949d5b4a735f8e6c7116900cbd, unsigned
  native source: unsandboxed; not executed by the registry check. Compile and test against the exact pinned Horizon and Mixins revisions before review.
```

`cargo horizon-mod fits measurements.hmod`:

```
Measurements:
0b14f8a  made for
```

The package is 19 entries (`horizon-mod.toml`, `Cargo.lock`, `Cargo.toml`, `LICENSE`,
`LICENSE-NOTICE.md`, `README.md`, six `assets/measurements/…`, four `data/…`, `src/lib.rs`,
`src/runtime.rs`, `uses.json`) and rebuilding it reproduces the same SHA-256.

### Traps (the wave will hit these)

1. **`use_item_on` is not `use_on` at this SDK.** `use_item_on` is now the **block** Part hook
   (`BlockBehaviour.useItemOn`); the item's `ItemStack.useOn` — what the retired catalog called
   `use_item_on` — is `use_on` on `ItemPart`. A mechanical rename compiles and never fires.
2. **An `ItemPart` needs `#[mod_api::horizon_part]`.** Without the attribute the implementation
   fails `E0046: not all trait items implemented, missing: item_subscribed`.
3. **`host.players().player(uuid)` does not exist.** `Players` has `server_players`,
   `player_by_name` and `acting_player`; the by-UUID lookup is `ServerHost::player(uuid)`
   (`E0599` otherwise).
4. **The off hand is `40` in one API and `-106` in another.** `PlayerRef::inventory_slots` and
   `ENTITY_ITEMS` number it `40` (`Inventory`'s own), while `ItemPlace::PlayerSlot` uses `-106`.
   Mixing them reads the wrong slot with no error.
5. **`player_tick_pre/post` are both-sides hooks with a `HostSide`.** A server-only body must
   return on `HostSide::Client` or the client tick drives the server-side session map.
6. **`cargo horizon-mod build` packages without compiling.** It reported `Built …hmod` from a
   crate that had never been type-checked (no crate artifacts, no `Cargo.lock` in the first
   archive). The compile proof is `cargo check --all-targets` / `cargo test`, and
   `cargo horizon-mod check` with no file.
7. **`cargo horizon-mod check` (no file) needs `wasm32-unknown-unknown`.** Without it the run
   ends `can't find crate for 'core' … the wasm32-unknown-unknown target may not be installed`;
   `rustup target add wasm32-unknown-unknown` fixes it (installed on this box on 2026-11-18).
8. **The slot runner forwards `check`/`test`/`build` to the second machine, which does not have
   the port yet.** A fresh SDK port failed there with
   `bash: line 1: cd: /home/deepseek/atm10/measurements: No such file or directory`; run the
   migration ladders with `BUILD_LOCAL=1` until the remote sync covers `.horizon/` and the new
   layout. The remote failure is a sync problem, not a code problem: the same command is green
   locally.
9. **There is no in-game e2e for an SDK mod without a launcher instance.**
   `cargo horizon-mod run` answers `code mods run through the launcher's native build:
   horizon-launcher native-build . --trust-native-code`, and this machine has neither the
   launcher nor an instance.

### Verification after the migration

| Check | Command | Result |
|---|---|---|
| runtime byte-identity | `cmp src/runtime.rs` (archived copy) | identical, `sha256 7c0a2dc8…c23ef700` |
| format | `cargo fmt --all -- --check` | exit 0 (`evidence/measurements/sdk-fmt.log`) |
| lints | `cargo clippy --all-targets -- -D warnings` | exit 0, 0 warnings (`sdk-clippy.log`) |
| tests | `cargo test` | **17 passed**, 0 failed — 13 carried runtime + 4 glue (`sdk-test.log`) |
| API/side gate | `cargo horizon-mod check` | exit 0, 111 items, native + wasm32 legs |
| package | `cargo horizon-mod build` | exit 0, 19 entries (`sdk-build.log`) |
| package check | `cargo horizon-mod check measurements.hmod` | exit 0, 19,536 B, SHA-256 `a673c67a…00cbd` (`sdk-check-package.log`) |
| fits | `cargo horizon-mod fits measurements.hmod` | exit 0, `0b14f8a  made for` (`sdk-fits.log`) |
| glue test is not vacuous | removed the off-hand clause of `is_held_slot` and re-ran | `the_off_hand_slot_40_is_a_hand` **failed** (16 passed, 1 failed); restored, green |

### Gaps after the migration

1. **No in-game run since the migration.** The pre-SDK e2e (default + `COUNTERFACTUAL=1`, both
   `PASS` with `EXIT:0`) describes the retired bundle; see `## Verification` above and
   `evidence/measurements/e2e*.log`. It cannot be re-run until a launcher instance exists —
   `horizon-launcher native-build` is the only way to get a game binary with this mod in it.
   Every behaviour claim in this note therefore still rests on the retired bundle's run, not on
   this package.
2. **The item Part's attachment is unverified in game.** `r.item_key("measurements:tape_measure")`
   is documented as "requested by registry id, validated after all declarations"; the item is
   declared by this package's own `data/measurements/horizon/item/tape_measure.json`. It
   compiles and passes `cargo horizon-mod check`, but no run has shown a click reaching the part.
3. **The client box is still not drawn** (`Deviations and gaps` 1). Unchanged by the migration;
   the SDK's client draw hooks make it possible for a later slice.

### Changelog for the resubmission

> Supersedes #708, closed because Horizon retired `.hnmod` (#17992). Changes: the port is now
> one flat SDK package with `api = "0.2"` and `[native]` pinned to `0b14f8a` (mixins `e59da90`,
> game 26.3); the `use_item_on`, `player_tick`, `player_logged_out` and `server_tick` injections
> and the embedded-content `content_root` injection are gone, rewritten as typed
> `impl mod_api::Hooks` methods and one `ItemPart::use_on` on `measurements:tape_measure`; the
> tape measure's assets and data now travel in the package itself instead of an embedded
> `.hmod`; `runtime.rs` and its 13 tests are unchanged (byte-identical). Version 0.2.0.

Skills: superpowers/test-driven-development,systematic-debugging,verification-before-completion; BeautifulRust/rust-best-practices
