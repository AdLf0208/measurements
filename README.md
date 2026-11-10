# measurements (Horizon port)

A Horizon port of [measurements](Mrbysco/Measurements) — MIT.

**Status: first slices.** What is below is what the port actually does today.

## Implemented so far

- Item** `measurements:tape_measure`: right-click a block to place point A,
- right-click again to place point B and finish the measurement. The action bar
- shows `Distance: <d> blocks (X: <dx>, Y: <dy>, Z: <dz>)`.
- Distance** is Euclidean over the coordinate differences; the distance is
- printed with one decimal (`5.0`, `7.1`, `0.0`) and the deltas as integers.
- Next click** after a finished measurement starts a new point A.
- Shift-right-click** removes the **latest** measurement (upstream's
- `undo`), reporting `Measurement undone (N left)` or `Measurement cleared`

## Not yet ported

- Delivery is the action bar, not an in-world box.** Upstream is a
- client-only renderer: `MeasurementBox` draws an AABB and three axis labels
- (`MeasurementBox.java:118-173`). It has no chat/action-bar readout and no
- Euclidean distance. The pin has no client render hook usable in the headless
- harness, so the port reports through `native_action_bar`, as the campaign
- brief chose. Deferred: the drawn box, its colours (line/text, `XYZRGB`),
- text size, line width, and the live crosshair preview.
- Added distance and integer deltas.** The port prints the Euclidean

## Install

Build the bundle with a native-mod capable launcher, or add the file from the
registry once it is listed. Pinned to Horizon `1d63e0a` and horizon-mixins
`5a64af93` for game 26.3.

## Build from source

```sh
cargo test
python3 tools/horizon.py check --horizon <horizon checkout>
python3 tools/horizon.py build --horizon <horizon checkout>
python3 tools/package.py
```

## Credits and licence

Original mod: Mrbysco/Measurements. Port: AdLf0208. Licence: MIT — see [LICENSE](LICENSE);
the original notices are preserved.
