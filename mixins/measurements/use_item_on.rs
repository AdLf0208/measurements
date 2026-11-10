// Runs where `ItemStack.useOn` starts: with the tape measure, place point A,
// finish point B, or (sneaking) remove the latest measurement.
fn observe_use_item_on(
    game: &mut crate::game::Game<'_>,
    stack: &horizon_client::items::ItemStack,
    pos: [i32; 3],
    claimed: &mut bool,
) {
    crate::game::measurements_state::on_use_item_on(game, stack, pos, claimed);
}
