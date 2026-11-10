// A player left: their measurements go with them (API 57).
fn observe_player_logged_out(player: u128) {
    crate::game::measurements_state::on_player_logged_out(player);
}
