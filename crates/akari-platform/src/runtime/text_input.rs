use super::*;

impl<A: Application> Runtime<A> {
    pub(super) fn move_text_cursor(&mut self, right: bool, shift: bool) {
        let Some((key, text, state, rect, align)) = self.ui.editing_aligned() else {
            return;
        };
        let Some(graphics) = &mut self.graphics else {
            return;
        };
        let current = (state.cursor, state.cursor_after);
        let mut target = graphics
            .renderer
            .text_move_aligned(&text, 16., rect.w, current, right, align);
        if !shift && state.anchor != state.cursor {
            let cursor_x = graphics.renderer.text_cursor_x_aligned(
                &text,
                16.,
                rect.w,
                state.cursor,
                state.cursor_after,
                align,
            );
            let anchor_x = graphics.renderer.text_cursor_x_aligned(
                &text,
                16.,
                rect.w,
                state.anchor,
                state.anchor_after,
                align,
            );
            if let (Some(cursor_x), Some(anchor_x)) = (cursor_x, anchor_x) {
                target = if (right && cursor_x > anchor_x) || (!right && cursor_x < anchor_x) {
                    Some((state.cursor, state.cursor_after))
                } else {
                    Some((state.anchor, state.anchor_after))
                };
            }
        }
        if let Some((cursor, cursor_after)) = target
            && let Some(input) = self.ui.states.get_mut(&key)
        {
            input.cursor = cursor;
            input.cursor_after = cursor_after;
            if !shift {
                input.anchor = cursor;
                input.anchor_after = cursor_after;
            }
        }
        self.rebuild();
    }

    pub(super) fn position_cursor(&mut self, x: f32, y: f32) {
        let Some((key, text, state, rect, align)) = self.ui.editing_aligned() else {
            return;
        };
        let Some(graphics) = &mut self.graphics else {
            return;
        };
        let (cursor, cursor_after) = graphics.renderer.text_hit_aligned(
            &text,
            16.,
            rect.w,
            [x - rect.x, y - rect.y],
            (state.cursor, state.cursor_after),
            align,
        );
        if let Some(input) = self.ui.states.get_mut(&key) {
            input.cursor = cursor;
            input.cursor_after = cursor_after;
            input.anchor = cursor;
            input.anchor_after = cursor_after;
        }
        self.rebuild();
    }
}
