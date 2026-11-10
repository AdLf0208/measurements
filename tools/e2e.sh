#!/usr/bin/env bash
# In-game check for the Measurements tape measure.
#
# Default run (creative, The End, two 5-block walls on the X and Z axes):
#   1. right-click a block with measurements:tape_measure -> action bar
#      "Point A: 100 64 100";
#   2. right-click the block 5 blocks away -> "Distance: 5.0 blocks
#      (X: 5, Y: 0, Z: 0)";
#   3. the next click starts a new A;
#   4. aim along the other axis -> "Distance: 5.0 blocks (X: 0, Y: 0, Z: 5)";
#   5. shift-right-click clears, and the next click starts a new A;
#   6. control: a diamond pickaxe click changes nothing, so the next tape
#      click starts A instead of finishing a measurement;
#   7. an off-hand tape keeps the session armed: point A with the main-hand
#      tape, move it to the off hand, empty the main hand, then click B -- the
#      readout must still be the distance, not a new point A (the old
#      main-hand-only reset dropped the session here);
#   8. logout: a LAN guest takes point A, disconnects, rejoins, and its first
#      click must start a new point A (the `player_logged_out` clear).
#
# COUNTERFACTUAL=1 swaps the tape for minecraft:stick: the identical clicks
# must produce no measurement readout. That is how the check proves it can
# fail -- the port item, not the scenario, does the work.
set -euo pipefail

CLONE=${CAMPAIGN_ROOT:-$(cd "$(dirname "$0")/../../.." && pwd)}
PORT=$CLONE/ports/measurements
# The control binary is shared at the campaign pin; the game is this port's build.
CTL=${CTL:-$CLONE/ports/_shared/bin/horizon-1d63e0a5/horizon-ctl}
GAME=${GAME:-$PORT/target/horizon-cargo/fast/horizon-client}
RES=${RES:-${HORIZON_RES:-${HORIZON_RES:-$HOME/.local/horizon-client/26.3}}}
SOCK=${SOCK:-/tmp/opencode/measurements-e2e.sock}
COUNTERFACTUAL=${COUNTERFACTUAL:-0}
# SELFTEST=1 keeps the default run but asserts a 6.0-block distance from a
# 5-block click, so the run must fail: the counterfactual proving the readout
# assertion is not vacuous.
#
# EMBEDDED=1: the game is launched with **only the native bundle** (no
# `--dev-mod`) and the content comes from the bundle's embedded copy
# (`mixins/measurements/data/measurements-content.hmod`, the AE2 pattern). The
# run also checks the file the helper wrote. CONTROL=1 (default with EMBEDDED)
# first launches the same binary with `.horizon-cache` unusable, where the tape
# measure id must be unknown.
SELFTEST=${SELFTEST:-0}
EMBEDDED=${EMBEDDED:-0}
CONTROL=${CONTROL:-$EMBEDDED}
WORLD=MeasurementsE2E
CONTROL_WORLD=MeasurementsControl

[ -x "$CTL" ] || { echo "missing horizon-ctl: $CTL" >&2; exit 1; }
[ -x "$GAME" ] || { echo "missing horizon-client: $GAME" >&2; exit 1; }
[ -d "$RES/assets" ] || { echo "missing prepared resources: $RES" >&2; exit 1; }
[ -f "$PORT/content/horizon-mod.toml" ] || { echo "missing content mod: $PORT/content" >&2; exit 1; }

if [ "$COUNTERFACTUAL" = 1 ]; then
    TAPE=minecraft:stick
else
    TAPE=measurements:tape_measure
fi

SCRATCH=$(mktemp -d /tmp/opencode/measurements-home.XXXXXX)
export HORIZON_HOME=$SCRATCH/game
export HORIZON_LAUNCHER_HOME=$SCRATCH/launcher
export XDG_DATA_HOME=$SCRATCH/data
export XDG_CONFIG_HOME=$SCRATCH/config
export HORIZON_NO_AUDIO=1
mkdir -p "$(dirname "$SOCK")"
rm -f "$SOCK"
# The game's game folder defaults to the working directory: run from scratch so
# saves/options/caches never land in the port tree.
cd "$SCRATCH"

ctl() { "$CTL" --socket "$SOCK" "$@"; }
fail() { echo "FAIL: $*" >&2; exit 1; }
actionbar() { ctl messages 5 | jq -r '.actionbar // ""'; }
tp_to() {
    ctl command "execute in minecraft:the_end run tp @s $1 $2 $3" >/dev/null
    ctl wait 4
}
click() {
    ctl look-at-block "$1" "$2" "$3" >/dev/null || fail "cannot aim at $1 $2 $3"
    ctl wait 3
    aimed=$(ctl looking-at | jq -c '.block.pos // empty')
    [ "$aimed" = "[$1,$2,$3]" ] || fail "aimed at $aimed, expected [$1,$2,$3]"
    ctl use-item
    ctl wait 5
}
shift_click() {
    ctl key shift down
    ctl wait 2
    click "$1" "$2" "$3"
    ctl key shift up
    ctl wait 3
}
expect_action() {
    local want=$1 got
    got=$(actionbar)
    [ "$got" = "$want" ] || fail "action bar was '$got', expected '$want'"
    echo "actionbar: $got"
}
expect_not_measurement() {
    local got
    got=$(actionbar)
    case "$got" in
        "Point A:"*|"Distance:"*|"Measurement cleared"*)
            fail "the counterfactual produced a readout: '$got'" ;;
    esac
    echo "actionbar (no measurement): '$got'"
}

cleanup() {
    local status=$?
    ctl quit >/dev/null 2>&1 || true
    sleep 1
    rm -rf "$SCRATCH" || true
    # Last line of every run: `check-e2e-logs.py` reads it as the verdict.
    echo "EXIT:$status"
}
trap cleanup EXIT

launch() { # launch [WORLD]
    local world=${1:-$WORLD}
    rm -f "$SOCK"
    cd "$SCRATCH"
    local args=(--game "$GAME" --world "$world" --seed 20261104 --creative --headless --wait
                --resources "$RES")
    if [ "$EMBEDDED" = 1 ]; then
        echo "  launch: no --dev-mod; the bundle carries its own content" >&2
    else
        args+=(--dev-mod "$PORT/content")
    fi
    "$CTL" --socket "$SOCK" launch "${args[@]}" >/dev/null
    ctl wait-ready 120
}

# The control for EMBEDDED=1: the same binary (every hook is in it) with the
# embedded content made unavailable — `.horizon-cache` is a plain file, so the
# helper can neither compare nor write its content file and falls back to
# `Continue`. The tape measure id must then be unknown, and a click must not
# produce a readout.
control_leg() {
    echo "== control: the same game with no embedded content file =="
    rm -rf "$SCRATCH/.horizon-cache"
    : > "$SCRATCH/.horizon-cache"
    launch "$CONTROL_WORLD"
    ctl command "execute in minecraft:the_end run tp @s 100.5 100 102.5" >/dev/null
    ctl wait 20
    ctl command "execute in minecraft:the_end run fill 96 63 96 106 63 106 minecraft:stone" >/dev/null
    ctl command "execute in minecraft:the_end run fill 96 64 96 106 70 106 minecraft:air" >/dev/null
    ctl command "execute in minecraft:the_end run tp @s 100.5 64 102.5" >/dev/null
    ctl wait 6
    ctl command "item replace entity @s hotbar.0 with measurements:tape_measure 1" >/dev/null 2>&1 || true
    ctl wait 4
    ctl inventory | jq -e '[.hotbar[], .main[] | select(.item == "measurements:tape_measure")] | length == 0' >/dev/null \
        || fail "the control game has a tape measure, so the content did not come from the embedded file"
    ctl select 1 >/dev/null 2>&1 || true
    ctl command "give @s minecraft:stick 1" >/dev/null 2>&1 || true
    ctl wait 3
    ctl look-at-block 100 64 100 >/dev/null 2>&1 || true
    ctl use-item >/dev/null 2>&1 || true
    ctl wait 4
    case "$(actionbar)" in
        "Point A:"*|"Distance:"*|"Measurement cleared"*)
            fail "the control game produced a measurement readout: '$(actionbar)'" ;;
    esac
    [ -f "$SCRATCH/.horizon-cache" ] || fail "the control's .horizon-cache was replaced: the helper wrote its content"
    echo "  without the embedded file: no tape item, no readout"
    ctl quit >/dev/null 2>&1 || true
    rm -f "$SCRATCH/.horizon-cache"
}

# EMBEDDED=1: the bundle must have written its content out at startup, byte for
# byte the delivered content mod.
embedded_content_check() {
    local cache=$SCRATCH/.horizon-cache/measurements-embedded-content.hmod
    [ -f "$cache" ] || fail "the bundle did not write its embedded content to $cache"
    cmp -s "$cache" "$PORT/content/measurements.hmod" \
        || fail "the embedded content differs from content/measurements.hmod"
    echo "  embedded content: $(wc -c < "$cache" | tr -d ' ') B at $cache, identical to content/measurements.hmod"
}

echo "== launch (creative, The End, $([ "$EMBEDDED" = 1 ] && echo 'native bundle only (embedded content)' || echo 'content --dev-mod content/')) =="
if [ "$CONTROL" = 1 ]; then control_leg; fi
launch
if [ "$EMBEDDED" = 1 ]; then embedded_content_check; fi

echo "== teleport to The End, then build the arena (chunks must load first) =="
ctl command "execute in minecraft:the_end run tp @s 100.5 100 102.5"
ctl wait 20
dimension=$(ctl state | jq -r '.dimension // ""')
echo "dimension: $dimension"
[ "$dimension" = "minecraft:the_end" ] || fail "not in The End: $dimension"
ctl command "execute in minecraft:the_end run fill 96 63 96 106 63 106 minecraft:stone"
ctl command "execute in minecraft:the_end run fill 96 64 96 106 70 106 minecraft:air"
ctl command "execute in minecraft:the_end run fill 100 64 100 105 64 100 minecraft:stone"
ctl command "execute in minecraft:the_end run fill 100 64 100 100 64 105 minecraft:stone"
ctl command "execute in minecraft:the_end run tp @s 100.5 64 102.5"
ctl wait 10
ctl command "clear @s"
ctl give "$TAPE" 1 >/dev/null || fail "the content mod did not provide $TAPE"
ctl give minecraft:diamond_pickaxe 1 >/dev/null || fail "cannot give a pickaxe"
ctl wait 4
ctl select "$TAPE" >/dev/null || fail "cannot select $TAPE"
ctl wait 3
held=$(ctl inventory | jq -r '.held.item // "none"')
echo "held: $held"
[ "$held" = "$TAPE" ] || fail "the held item is $held, expected $TAPE"

echo "== X axis: point A at 100,64,100 =="
tp_to 100.5 64 102.5
click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Point A: 100 64 100"
fi

echo "== point B five blocks away on X =="
tp_to 105.5 64 102.5
click 105 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
elif [ "$SELFTEST" = 1 ]; then
    expect_action "Distance: 6.0 blocks (X: 6, Y: 0, Z: 0)"
else
    expect_action "Distance: 5.0 blocks (X: 5, Y: 0, Z: 0)"
fi

if [ "$COUNTERFACTUAL" != 1 ]; then
echo "== a tape in the off hand keeps the session armed (both hands) =="
tp_to 100.5 64 102.5
click 100 64 100
expect_action "Point A: 100 64 100"
ctl command "execute in minecraft:the_end run item replace entity @s weapon.offhand with measurements:tape_measure" >/dev/null
ctl command "execute in minecraft:the_end run item replace entity @s weapon.mainhand with minecraft:air" >/dev/null
ctl wait 8
# At the 1d63e0a pin the window's `state` mirror and `inventory`'s slot 504 can
# lag the server-side off hand after `item replace`, so the phase asserts the
# behaviour that names it: with the main hand emptied, a click must still finish
# the measurement, which is only possible when the port reads the off-hand tape.
echo "off hand (inventory 504): $(ctl inventory | jq -r '.offhand.item // "none"'), state mirror: $(ctl state | jq -r '.offhand.item // "none"'), main hand: $(ctl state | jq -r '.held.item // "none"')"
tp_to 105.5 64 102.5
click 105 64 100
expect_action "Distance: 5.0 blocks (X: 5, Y: 0, Z: 0)"
# Leave exactly one finished measurement for the counts the next phases assert.
shift_click 105 64 100
expect_action "Measurement undone (1 left)"
ctl command "execute in minecraft:the_end run item replace entity @s weapon.mainhand with measurements:tape_measure" >/dev/null
ctl command "execute in minecraft:the_end run item replace entity @s weapon.offhand with minecraft:air" >/dev/null
ctl wait 4
ctl select "$TAPE" >/dev/null || fail "cannot select $TAPE after the off-hand phase"
ctl wait 3
fi

echo "== a further click starts the next measurement on the Z axis =="
tp_to 100.5 64 102.5
click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Point A: 100 64 100"
fi
tp_to 102.5 64 105.5
click 100 64 105
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Distance: 5.0 blocks (X: 0, Y: 0, Z: 5)"
fi

echo "== shift-right-click undoes the latest, then clears the rest =="
tp_to 100.5 64 102.5
shift_click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Measurement undone (1 left)"
fi
shift_click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Measurement cleared"
fi
echo "== with nothing left, the next click starts a new A =="
tp_to 100.5 64 102.5
click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Point A: 100 64 100"
fi

echo "== shift-clear again, then a different item must do nothing =="
shift_click 100 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Measurement cleared"
fi
ctl select minecraft:diamond_pickaxe >/dev/null || fail "cannot select the pickaxe"
ctl wait 3
tp_to 100.5 64 102.5
click 100 64 100
pickaxe_action=$(actionbar)
case "$pickaxe_action" in
    "Point A:"*|"Distance:"*) fail "the pickaxe produced a measurement readout: '$pickaxe_action'" ;;
esac
echo "actionbar after the pickaxe click: '$pickaxe_action'"
ctl select "$TAPE" >/dev/null || fail "cannot select the tape again"
ctl wait 3
# If the pickaxe click had started a measurement, this click would finish it
# and print a distance; it must print a fresh point A instead.
tp_to 105.5 64 102.5
click 105 64 100
if [ "$COUNTERFACTUAL" = 1 ]; then
    expect_not_measurement
else
    expect_action "Point A: 105 64 100"
fi

if [ "$COUNTERFACTUAL" != 1 ]; then
echo "== logout: a player who leaves loses the session at once (player_logged_out) =="
# The pin fires `player_logged_out` for a LAN guest (and for the guests still
# aboard at a server stop); the single-player host leaves through the
# `server_tick` prune instead. So a guest joins, takes point A, logs out and
# rejoins: the first click after the rejoin must start a new measurement
# rather than finish the one from before the logout.
GUEST_ROOT=$SCRATCH/guest
GUEST_SOCK=$SCRATCH/guest.sock
mkdir -p "$GUEST_ROOT"
guest_ctl() {
    ( cd "$GUEST_ROOT" && env HORIZON_HOME="$GUEST_ROOT/game" \
        HORIZON_LAUNCHER_HOME="$GUEST_ROOT/launcher" \
        XDG_DATA_HOME="$GUEST_ROOT/data" XDG_CONFIG_HOME="$GUEST_ROOT/config" \
        HORIZON_NO_AUDIO=1 "$CTL" --socket "$GUEST_SOCK" "$@" )
}
guest_actionbar() { guest_ctl messages 5 | jq -r '.actionbar // ""'; }
guest_click() {
    guest_ctl look-at-block "$1" "$2" "$3" >/dev/null || fail "the guest cannot aim at $1 $2 $3"
    guest_ctl wait 3
    guest_ctl use-item
    guest_ctl wait 5
}
port=$(ctl lan open 0 commands | jq -r '.port')
[ -n "$port" ] && [ "$port" != null ] || fail "LAN did not report a port"
echo "LAN port: $port"
rm -f "$GUEST_SOCK"
guest_args=(--game "$GAME" --headless --name Guest --resources "$RES")
if [ "$EMBEDDED" != 1 ]; then guest_args+=(--dev-mod "$PORT/content"); fi
guest_ctl launch "${guest_args[@]}" >/dev/null
guest_ctl connect "127.0.0.1:$port" Guest
guest_ctl wait-for loaded --timeout 300
guest_ctl wait 60
guest_ctl state | jq -e '.player != null' >/dev/null \
    || fail "the guest did not join: $(guest_ctl state | head -c 300)"
guest_ctl command "execute in minecraft:the_end run tp @s 100.5 64 102.5" >/dev/null
guest_ctl command "give @s measurements:tape_measure 1" >/dev/null
guest_ctl wait 10
guest_ctl select measurements:tape_measure >/dev/null || fail "the guest cannot select the tape"
guest_ctl wait 3
guest_click 100 64 100
got=$(guest_actionbar)
[ "$got" = "Point A: 100 64 100" ] || fail "the guest's first click was '$got'"
echo "guest point A: '$got'"
guest_ctl disconnect >/dev/null || true
# Back on the title screen: `wait` refuses there ("no world open"), so give the
# client a moment in wall time and let the poll below wait for the rejoin.
sleep 5
guest_ctl connect "127.0.0.1:$port" Guest
# At the 1d63e0a pin `wait-for loaded` needs an open world and a reconnecting
# guest is still on its title screen, so poll the state that matters: the guest
# has a player body again. (The first join above still uses wait-for loaded.)
for _ in $(seq 1 60); do
    guest_ctl state 2>/dev/null | jq -e '.player != null' >/dev/null 2>&1 && break
    # While the rejoin is still loading the guest is on its title screen, where
    # `wait` refuses ("no world open"); the poll must not abort the run.
    guest_ctl wait 10 >/dev/null 2>&1 || true
done
guest_ctl state 2>/dev/null | jq -e '.player != null' >/dev/null \
    || fail "the guest did not rejoin: $(guest_ctl state 2>&1 | head -c 300)"
guest_ctl wait 60
guest_ctl select measurements:tape_measure >/dev/null \
    || fail "the guest cannot select the tape after the rejoin"
guest_ctl wait 3
guest_click 105 64 100
got=$(guest_actionbar)
case "$got" in
    "Point A: 105 64 100")
        echo "after the logout+rejoin the first click started fresh: '$got'" ;;
    *) fail "the session survived the guest's logout: '$got'" ;;
esac
guest_ctl quit >/dev/null 2>&1 || true
fi

if [ "$COUNTERFACTUAL" = 1 ]; then
    echo "COUNTERFACTUAL PASS: a plain stick produced no measurement readout in any of the same clicks"
else
    echo "PASS: A->B read 5.0 on X, 5.0 on Z, shift-click cleared, the off-hand tape kept the session armed, a guest's logout dropped its session, and the pickaxe control did nothing"
fi
