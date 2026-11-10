// A player's server tick: forget measurements once the tape is in neither hand.
fn observe_player_tick(game: &mut crate::game::Game<'_>) {
    crate::game::measurements_state::on_player_tick(game);
}
