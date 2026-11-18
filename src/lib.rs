//! Measurements for Horizon: the tape measure reports the distance between two clicked blocks.
//!
//! Ported from Measurements 3.0.3 (`Mrbysco/Measurements`, MIT); the behaviour and the
//! upstream evidence are in `notes.md`. Upstream keeps one client-side list of measurement
//! boxes and draws them (`BoxHandler.java`, `MeasurementBox.java`); this port keeps a
//! session per player on the server and reports the Euclidean distance and the per-axis
//! deltas over the action bar, a deviation recorded in `notes.md`.
//!
//! [`runtime`] holds every number and no `mod_api` type; this file is only the glue: one
//! `impl Hooks`, one [`ItemPart`] on `measurements:tape_measure`, and the per-player session
//! map they share.

pub mod runtime;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex, MutexGuard};

use mod_api::{
    Hooks, HostSide, InteractionResult, ItemPart, PlayerLoggedOutEvent, PlayerRef,
    PlayerTickPostEvent, Registrar, RegistrationContext, ServerHost, ServerTickEvent,
    ServerTickPhase, Text, UseOnEvent,
};

use runtime::{Point, Session, is_tape_measure, readout};

/// The off hand's inventory slot in `PlayerRef::inventory_slots` numbering (`Inventory`'s
/// own: 0..36 the main inventory, 36..40 the armour, 40 the off hand).
const OFF_HAND_SLOT: u32 = 40;

/// One player's measurements and the dimension they were taken in.
struct PlayerMeasurements {
    /// The dimension the measurements were made in, as `namespace:path`. Upstream draws a
    /// box only in the dimension it was made in, so a click elsewhere starts clean.
    dimension: String,
    session: Session,
}

/// Every player's measurements, keyed by UUID. Upstream keeps one process-wide client list
/// (`BoxHandler.java:12-13`); the port runs on the server and keys it by player.
#[derive(Default)]
struct State {
    players: BTreeMap<u128, PlayerMeasurements>,
}

/// The state the hooks and the tape measure's Part share.
#[derive(Default)]
struct Mod {
    state: Arc<Mutex<State>>,
}

fn lock(state: &Mutex<State>) -> MutexGuard<'_, State> {
    state.lock().unwrap_or_else(|poison| poison.into_inner())
}

#[mod_api::horizon_mod]
impl Hooks for Mod {
    /// Once, at startup: attach the tape measure's Part, so the game asks this mod only for
    /// clicks made with that item.
    fn register(&self, registrar: &mut dyn Registrar, _context: &RegistrationContext) {
        let tape = registrar.item_key(runtime::TAPE_MEASURE);
        registrar.item_part(
            tape,
            TapeMeasure {
                state: Arc::clone(&self.state),
            },
        );
    }

    /// After a player's tick on both logical sides: their measurements go once the tape is
    /// in neither hand, as upstream's `isHolding` check does (`ClientClass.java:29`).
    fn player_tick_post(&self, host: HostSide<'_>, event: &mut PlayerTickPostEvent) {
        let HostSide::Server(host) = host else {
            return;
        };
        self.after_player_tick(host, event.player);
    }

    /// A player left: their measurements go with them, as upstream's logout clear does
    /// (`ClientClass.java:19-25`).
    fn player_logged_out(&self, _host: &mut dyn ServerHost, event: &mut PlayerLoggedOutEvent<'_>) {
        lock(&self.state).players.remove(&event.player);
    }

    /// The end of a running server tick: forget players who left. `player_logged_out` covers
    /// every leave this game tells of, so this is the second line of defence the port has
    /// always carried.
    fn server_tick(&self, host: &mut dyn ServerHost, event: &mut ServerTickEvent<'_>) {
        let ServerTickPhase::ServerTickEnd { world_running } = event.phase else {
            return;
        };
        if !world_running {
            return;
        }
        let online: BTreeSet<u128> = host
            .players()
            .server_players()
            .into_iter()
            .map(PlayerRef::uuid)
            .collect();
        lock(&self.state)
            .players
            .retain(|uuid, _| online.contains(uuid));
    }
}

impl Mod {
    /// Drops a player's measurements once the tape is in neither hand.
    fn after_player_tick(&self, host: &mut dyn ServerHost, player: PlayerRef) {
        if holds_tape(host, player) {
            return;
        }
        lock(&self.state).players.remove(&player.uuid());
    }
}

/// Whether `slot` is one the player can click with: the selected hotbar slot (the main hand)
/// or the off hand. `inventory_slots` does not mark the selected slot, so the port reads it.
fn is_held_slot(slot: u32, selected: Option<u32>) -> bool {
    slot == OFF_HAND_SLOT || Some(slot) == selected
}

/// Whether the player holds the tape in the main or the off hand. Upstream's `isHolding`
/// covers both hands (`ClientClass.java:29`). An unreadable inventory keeps the session: a
/// read that failed is not evidence the tape is gone.
fn holds_tape(host: &mut dyn ServerHost, player: PlayerRef) -> bool {
    let Some(slots) = player.inventory_slots(host) else {
        return true;
    };
    let selected = player
        .selected_slot(host)
        .and_then(|slot| u32::try_from(slot).ok());
    slots.iter().any(|(slot, stack)| {
        is_held_slot(*slot, selected) && is_tape_measure(&stack.item().id(host))
    })
}

/// The tape measure's Part: one `use_on` per right click with it on a block.
struct TapeMeasure {
    state: Arc<Mutex<State>>,
}

#[mod_api::horizon_part]
impl ItemPart for TapeMeasure {
    fn use_on(&self, host: HostSide<'_>, event: &mut UseOnEvent<'_>) -> Option<InteractionResult> {
        let HostSide::Server(host) = host else {
            // The readout is the server's alone: a client prediction would print it twice.
            return None;
        };
        let player = host.player(event.player)?;
        // The dimension is the session's; a click in another one starts clean.
        let dimension = player.dimension(host)?.to_string();
        let sneaking = player.entity().is_shift_key_down(host).unwrap_or_default();
        let at = Point::from_array(event.pos);
        let outcome = {
            // Only the pure call runs under the lock; the action bar below is game work.
            let mut state = lock(&self.state);
            let PlayerMeasurements {
                dimension: held,
                session,
            } = state
                .players
                .entry(player.uuid())
                .or_insert_with(|| PlayerMeasurements {
                    dimension: dimension.clone(),
                    session: Session::new(),
                });
            if *held != dimension {
                held.clone_from(&dimension);
                session.clear();
            }
            session.click(at, sneaking)
        };
        let _ = player.action_bar(host, Text::literal(readout(&outcome)));
        Some(InteractionResult::SuccessServer)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_selected_hotbar_slot_is_the_main_hand() {
        assert!(is_held_slot(3, Some(3)));
        assert!(is_held_slot(0, Some(0)));
    }

    #[test]
    fn the_off_hand_slot_40_is_a_hand() {
        assert!(is_held_slot(OFF_HAND_SLOT, Some(3)));
        assert!(is_held_slot(OFF_HAND_SLOT, None));
    }

    #[test]
    fn another_hotbar_slot_is_not_held() {
        assert!(!is_held_slot(4, Some(3)));
    }

    #[test]
    fn the_armour_and_the_other_inventory_slots_are_not_held() {
        // 36 is the feet slot; 41 and 42 are a body and a saddle slot.
        for slot in [36, 37, 38, 39, 41, 42] {
            assert!(!is_held_slot(slot, Some(3)), "slot {slot} is not a hand");
        }
    }
}
