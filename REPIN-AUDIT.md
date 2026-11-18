# Measurements — re-pin audit (Horizon `83ad384f`)

> **Historical (2026-11-18).** This audit was written against the retired `.hnmod`/mixin
> format at pin `83ad384f`. The port is now a flat SDK `.hmod` at `0b14f8a`; the hook names
> below describe the old container and do not exist in the current source. See
> `notes.md` §"Migration to the SDK format".

Read-only audit. **No port code, content, manifest or campaign document changed.**

Nothing was built or run: every claim below is a source read of the two pins.

| | Pin | Checkout |
|---|---|---|
| New | Horizon `83ad384f48d0742746d518737511882680aab5b7` (427 hooks / 159 helpers), mixins `e8cd0aa38cffef49cb2e088ef716baa107148c02` | `ports/_shared/pins/horizon-83ad384f`, `.../horizon-mixins-e8cd0aa` |
| Old (shipped) | Horizon `c9d474dd9116e84c243f8992d75c11314df56f06` (108 hooks), mixins `d9c83d7…` | `ports/artifacts/target/mixins/original/32051c454db029bf4671` (staged c9d474d export used by the ports) |

Sources read: `notes.md` §Deviations and gaps, `ARCHITECTURE.md` item 18,
`ports/_shared/repin-83ad384f.md`; then `native-hooks.json` and the Rust
sources of both pins. All paths below are relative to the **new** pin unless
marked “c9d474d”.

## Summary

- **Deferred features audited: 7. Newly realizable: 7. Still blocked: 0.**
  (Plus one verification gap — the recipe — that is not a feature.)
- The pin was **not** the reason for any of the seven: three already had
  usable hooks at `c9d474d`, four are genuinely new.
- **Highest-value next step:** the in-world box. `native_world_drawn`
  (`crates/horizon-client-render/src/gfx/native_render.rs:152`) hands a mod the
  wgpu device/queue/encoder, the world depth and `view_proj`/`camera`
  (`WorldDrawn`, same file `:49-88`), so a client half can draw the AABB and the
  crosshair preview itself. Verify with `--offscreen` + `ctl screenshot`
  (`docs/horizon-ctl-commands.md:10,24`), not with the headless e2e.

## Deferred matrix

| # | Deferred feature (source) | Pin capability | Contract | At `c9d474d` | Verdict | Exact next step |
|---|---|---|---|---|---|---|
| 1 | In-world box: AABB + three axis labels (`notes.md` §Deferred 1; ARCHITECTURE item 18) | `native_world_drawn` (`native_render.rs:152`), `WorldDrawn.encoder` `:52`, `.view_proj` `:66`, `.jittered` `:67`, `.camera` `:69`, `.depth` `:62`; `native_render_needs` `:165` | listen | **yes** (`WorldDrawn` already had wgpu at c9d474d) | **realizable now** (raw wgpu: own pipeline + WGSL; no box/line helper exists) | A `world_drawn` helper module owning one pipeline/vertex buffer; draw 12 world-space line segments from the per-player endpoint list, `depth`-tested; needs the port's session state on the *client* half (single player: same `Game`; guests: sync via `net_from_client`/`net_from_server`) |
| 2 | Live crosshair preview of the unfinished box (`notes.md` §Deferred 1) | `native_view_hit` (`crates/horizon-client/src/game/native_claimed_items.rs:142`, gives the block hit + distance), `native_raycast` `:137` | helper | yes | **realizable now** | Same renderer as #1, fed by `native_view_hit` in the unfinished state |
| 3 | Colours (`XYZRGB` line/text), text size, line width (`notes.md` §Deferred 1, 7) | Own pipeline ⇒ colour/width are draw arguments; text is drawn as billboarded quads or via `native_world_drawn`'s colour target | listen | yes | **realizable now** | Part of #1: pass ARGB per segment/vertex; no pin surface needed beyond #1 |
| 4 | Client config (colours, text size, line width; `notes.md` §Deferred 7) | `native_register_configs` (`crates/horizon-client/src/game/native_hooks.rs:4242`), reconstruct/load/unload hooks, `config_dir` helper — **API 397, new** | collect | **no** (API 397 absent at c9d474d) | **realizable now** | Declare a `Client` config (`config/measurements-client.toml`), parse in `config_loading`, feed #1/#3 |
| 5 | Client `LineRenderType` (`notes.md` §Deferred 7) | no separate surface needed: with `world_drawn` the port owns its blend/depth state outright | — | n/a | **realizable now** | Choose blend/depth in the pipeline of #1 (no vanilla render-type hook exists, and none is needed) |
| 6 | Off-hand tape is not the reset check (`notes.md` §Deferred 5) | `native_inventory(uuid, Player)` = 41 live slots incl. off hand (`crates/horizon-client/src/game/native_inventories.rs:164`, **API 132, new**); `native_living_hand_item(living, Hand::Off)` (`crates/horizon-client/src/game/native_living_refs.rs:105`, **API 382, new**) | helper | **no** | **realizable now** | Call `native_inventory` from `server_tick` (a player's own hook is swapped in and returns `Err`), check slots 36–40, drop the session |
| 7 | No quit hook ⇒ prune one tick later (`notes.md` §Session, ARCHITECTURE item 18) | `native_player_logged_out` (`native_hooks.rs:6958`); hooks 57 pairs every leave, server stop included (`docs/native-mods.md:797`) | listen | **yes** (present at c9d474d) | **realizable now** | Replace the prune with a `player_logged_out` injection (keep the `server_tick` prune as a safety net) |

Not a feature: the recipe was never hand-crafted (`notes.md` §Deferred 6); that is
a harness probe, unaffected by the pin.

## The four spot checks

| Check | Result |
|---|---|
| **Block-state rotate/mirror (for `TRANSFORM`)** | Not relevant to this port; for the pin-wide answer see `../morered/REPIN-AUDIT.md`. There is still no rotate/mirror hook (no hook name in `native-hooks.json` contains `rotate`, `mirror` or `structure`). |
| **Client world-space rendering** | **Present and writable, already at `c9d474d`.** `native_world_drawn(WorldDrawn)` gives `device`/`queue`/`encoder`, the colour target, the reversed depth, `view_proj`/`jittered`/`camera`, entity quads and `block_sprites` (`native_render.rs:49-88,152`). It is a *raw* hook: no `draw_box`/`draw_line` helper, and `native_visuals` rims (`native_hooks.rs:2272`, `NativeBlockRim` `crates/horizon-client/src/mod_api.rs:225`) only trace a block's selection shape at up to 8 positions — not an arbitrary AABB. `--headless` has no renderer (`crates/horizon-client/src/game.rs:508-512`), so visual proof needs `--offscreen`. |
| **Heightmap** | Not needed by this port. `native_surface_heights(dimension, columns, NativeHeightmap)` exists at the new pin (`crates/horizon-client/src/game/native_world_columns.rs:22`) with `MotionBlocking` (`native_world_edit.rs:7-12`) — **API 127, new**; it is *not* the same thing as `native_level_height` (= world min-Y/height, `native_level_refs.rs:84`). |
| **Fluid / entity hooks** | Fluids: `native_block_entity_fluids` (`native_hooks.rs:3050`) + tank helpers were already at `c9d474d`; what is new is mod fluid **ids** (`machines/native_fluids.rs:37-53`) and `native_item_fluid_tank` (`native_hooks.rs:3081`) — see MGU. Entities: `native_summon`/`kill`/`set_health` existed; new are `experience_orb_pickup`, `entity_change_block`, `dragon_egg_form` (none needed here). |

## Corrections to the campaign re-pin note

`ports/_shared/repin-83ad384f.md` lists “client world-space box renderer → the
measurements box stays blocked” under *Still absent*. That is wrong on the
facts: `native_world_drawn` with `encoder` + `view_proj` was already in the
shipped pin (`c9d474d` export, `crates/horizon-client-render/src/gfx/native_render.rs`,
same `WorldDrawn` fields). The real blocker was never the pin; it is that a
headless e2e cannot see pixels and the box needs a hand-written wgpu pass. The
audit therefore records it as *realizable now (high effort)*, not *blocked*.

## Evidence (commands)

```
python3 -c '…json…'   # hooks/helpers of both native-hooks.json; name-set diff
grep -rn "fn native_world_drawn\|struct WorldDrawn"      ports/_shared/pins/horizon-83ad384f/crates/horizon-client-render/src/gfx/native_render.rs
grep -rn "fn native_view_hit\|fn native_raycast"          …/game/native_claimed_items.rs
grep -rn "fn native_inventory"                            …/game/native_inventories.rs
grep -rn "fn native_living_hand_item"                     …/game/native_living_refs.rs
grep -rn "fn native_player_logged_out\|fn native_register_configs" …/game/native_hooks.rs
grep -rn "headless"                                       …/game.rs:508-512 (no renderer)
grep -n "screenshot\|offscreen"                          …/docs/horizon-ctl-commands.md:10,24
```

Old-pin confirmation (staged `c9d474d` export):
`grep -n "WorldDrawn" ports/artifacts/target/mixins/original/32051c454db029bf4671/crates/horizon-client-render/src/gfx/native_render.rs`
→ `encoder`, `view_proj`, `camera` already present.
