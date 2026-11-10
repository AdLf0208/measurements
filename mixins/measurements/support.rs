// Game-side glue for the Measurements tape measure: per-player sessions and
// the action-bar readout.
//
// Injected into `crate::game` as `measurements_state`; it needs the game, so
// it is not part of the standalone library (only `runtime.rs` is).
//
// Upstream keeps one client-side list and clears it when the tape leaves the
// player's hands (`ClientClass.java:27-32`). The port keeps a session per
// player because it runs on the server, clears it when the player no longer
// holds the tape, and prunes sessions of players who left (there is no
// player-quit hook at the pin).

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Mutex, MutexGuard};

use horizon_client::items::ItemStack;
use horizon_client::text::Text;

use crate::game::measurements::{Point, Session, is_tape_measure, readout};

/// One player's measurements and the dimension they were taken in.
struct PlayerMeasurements {
    dimension: String,
    session: Session,
}

#[derive(Default)]
struct State {
    players: BTreeMap<u128, PlayerMeasurements>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn state() -> MutexGuard<'static, Option<State>> {
    STATE.lock().unwrap_or_else(|poison| poison.into_inner())
}

/// A right click on a block with the tape measure: the first click places
/// point A, the second finishes point B and reports the distance, and a
/// shift-click removes the latest measurement. API 13.
#[allow(private_interfaces)]
pub fn on_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    if !is_tape_measure(&stack.item) {
        return;
    }
    let Some(player) = game.native_acting_player() else {
        return;
    };
    *claimed = true;
    let sneaking = game
        .player
        .as_ref()
        .is_some_and(|player| player.shift_key_down);
    let at = Point::from_array(pos);
    let outcome = {
        let mut guard = state();
        let state = guard.get_or_insert_with(State::default);
        let entry = state
            .players
            .entry(player.uuid)
            .or_insert_with(|| PlayerMeasurements {
                dimension: player.dimension.clone(),
                session: Session::new(),
            });
        if entry.dimension != player.dimension {
            // Upstream draws a box only in the dimension it was made in; a
            // click elsewhere starts clean rather than joining two dimensions.
            entry.dimension.clone_from(&player.dimension);
            entry.session.clear();
        }
        entry.session.click(at, sneaking)
    };
    let line = readout(&outcome);
    game.native_action_bar(player.uuid, Text::literal(line));
}

/// A player's server tick: drop the session once the tape is in neither hand,
/// as upstream's `isHolding` check does (`ClientClass.java:29`, both hands).
/// API 31.
#[allow(private_interfaces)]
pub fn on_player_tick(game: &mut crate::game::Game<'_>) {
    let Some(player) = game.native_acting_player() else {
        return;
    };
    if tape_in_either_hand(game) {
        return;
    }
    let mut guard = state();
    if let Some(state) = guard.as_mut() {
        state.players.remove(&player.uuid);
    }
}

/// Whether the acting player holds the tape in the main or the off hand. The
/// port runs on the server, where a click can be made with either hand, so the
/// reset must look at both (`LivingEntity.getItemInHand`).
fn tape_in_either_hand(game: &crate::game::Game<'_>) -> bool {
    let main = game.stack_in(game.selected);
    let off = game.stack_in(crate::game::shield::OFFHAND_SLOT);
    main.into_iter()
        .chain(off)
        .any(|stack| is_tape_measure(&stack.item))
}

/// A player left: drop their session at once, as upstream's logout clear does
/// (`ClientClass.java:19-25`). API 57 pairs every leave, server stop included.
pub fn on_player_logged_out(player: u128) {
    let mut guard = state();
    if let Some(state) = guard.as_mut() {
        state.players.remove(&player);
    }
}

/// At the end of a running server tick, forget players who left (the pin has
/// no player-quit hook, so this is one tick late). API 4.
#[allow(private_interfaces)]
pub fn on_server_tick(
    game: &mut crate::game::Game<'_>,
    phase: crate::game::native_lifecycle::ServerTickPhase,
) {
    let crate::game::native_lifecycle::ServerTickPhase::ServerTickEnd { world_running } = phase
    else {
        return;
    };
    if !world_running {
        return;
    }
    let online: BTreeSet<u128> = game
        .native_server_players()
        .into_iter()
        .map(|player| player.uuid)
        .collect();
    let mut guard = state();
    if let Some(state) = guard.as_mut() {
        state.players.retain(|uuid, _| online.contains(uuid));
    }
}
