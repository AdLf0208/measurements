// End of a running server tick: prune sessions of players who left.
fn observe_server_tick(game: &mut crate::game::Game<'_>, phase: ServerTickPhase) {
    crate::game::measurements_state::on_server_tick(game, phase);
}
