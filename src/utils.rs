use ggez::graphics;

#[inline]
pub(crate) fn screen_size(ctx: &mut ggez::Context) -> (f32, f32) {
    let screen_rect = graphics::screen_coordinates(ctx);
    (screen_rect.w, screen_rect.h)
}
