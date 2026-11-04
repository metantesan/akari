use super::*;
use unicode_segmentation::UnicodeSegmentation;

impl Renderer {
    fn glyph(&mut self, key: CacheKey) -> Option<Glyph> {
        if let Some(g) = self.glyphs.get(&key) {
            return Some(*g);
        }
        let image = self.swash.get_image(&mut self.fonts, key).clone()?;
        let p = image.placement;
        if p.width == 0 || p.height == 0 {
            return None;
        }
        let color = matches!(image.content, SwashContent::Color);
        let data = if color {
            image.data
        } else if matches!(image.content, SwashContent::Mask) {
            image
                .data
                .iter()
                .flat_map(|a| [255, 255, 255, *a])
                .collect()
        } else {
            image
                .data
                .chunks_exact(3)
                .flat_map(|a| [255, 255, 255, a[0].max(a[1]).max(a[2])])
                .collect()
        };
        let uv = self.allocate(p.width, p.height)?;
        self.upload(uv, p.width, p.height, &data);
        let g = Glyph {
            uv,
            left: p.left,
            top: p.top,
            width: p.width,
            height: p.height,
            color,
        };
        self.glyphs.insert(key, g);
        Some(g)
    }
    fn buffer(&mut self, text: &str, size: f32, width: f32, align: TextAlign) -> Buffer {
        let mut b = Buffer::new(&mut self.fonts, Metrics::new(size, size * 1.4));
        b.set_size(Some(width.max(1.)), None);
        b.set_text(
            text,
            &Attrs::new(),
            Shaping::Advanced,
            Some(match align {
                TextAlign::Left => cosmic_text::Align::Left,
                TextAlign::Center => cosmic_text::Align::Center,
                TextAlign::Right => cosmic_text::Align::Right,
            }),
        );
        b.shape_until_scroll(&mut self.fonts, false);
        b
    }
    pub(super) fn text(
        &mut self,
        items: &mut Vec<Instance>,
        rect: Rect,
        clip: Rect,
        text: &str,
        style: (f32, Color, TextAlign),
    ) {
        let (size, color, align) = style;
        let b = self.buffer(text, size, rect.w, align);
        let runs: Vec<_> = b.layout_runs().collect();
        let layout_top = runs.first().map_or(0., |run| run.line_top);
        let layout_bottom = runs
            .last()
            .map_or(size, |run| run.line_top + run.line_height);
        let block_height = layout_bottom - layout_top;
        let y_shift = match align {
            TextAlign::Left | TextAlign::Right => 0.,
            TextAlign::Center => ((rect.h - block_height) * 0.5).max(0.),
        };
        for run in runs {
            for glyph in run.glyphs {
                let p = glyph.physical((rect.x, rect.y + y_shift + run.line_y - layout_top), 1.);
                if let Some(g) = self.glyph(p.cache_key) {
                    let r = Rect::new(
                        (p.x + g.left) as f32,
                        (p.y - g.top) as f32,
                        g.width as f32,
                        g.height as f32,
                    );
                    let mut i =
                        Instance::new(r, clip, if g.color { [1.; 4] } else { color }, 1., 0., 0.);
                    i.uv = g.uv;
                    items.push(i);
                }
            }
        }
    }
    pub(super) fn text_field(
        &mut self,
        items: &mut Vec<Instance>,
        field: &Draw,
        scale: f32,
        accent: Color,
    ) {
        let Draw::TextField {
            rect,
            clip,
            text,
            size,
            color,
            align,
            state,
            focused,
        } = field
        else {
            return;
        };
        let rect = Rect::new(
            rect.x * scale,
            rect.y * scale,
            rect.w * scale,
            rect.h * scale,
        );
        let clip = Rect::new(
            clip.x * scale,
            clip.y * scale,
            clip.w * scale,
            clip.h * scale,
        );
        let size = size * scale;
        let composing = !state.composition.is_empty();
        let (composition_start, composition_end, caret_byte) = if composing {
            let start = state.anchor.min(state.cursor);
            let end = start + state.composition.len();
            let preedit_cursor = state
                .composition_cursor
                .map(|(_, cursor_end)| cursor_end)
                .unwrap_or(state.composition.len());
            (start, end, start + preedit_cursor)
        } else {
            (0, 0, state.cursor)
        };
        let caret_visible = !composing || state.composition_cursor.is_some();
        // The UI draw builder inserts preedit text into this display string;
        // the retained app value is still unchanged until IME commit.
        let mut buffer = self.buffer(text, size, rect.w, *align);
        buffer.set_wrap(Wrap::None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let runs: Vec<_> = buffer.layout_runs().collect();
        let text_cursor = Cursor::new_with_affinity(
            0,
            caret_byte,
            Affinity::from_after(if composing { false } else { state.cursor_after }),
        );
        let caret = runs
            .iter()
            .find_map(|run| run.cursor_position(&text_cursor))
            .unwrap_or(0.);
        let caret_x = caret + 1.5;
        let scroll_x = if *focused {
            (caret_x - (rect.w - 4.).max(1.)).max(0.)
        } else {
            0.
        };
        let layout_top = runs.first().map_or(0., |run| run.line_top);
        let layout_bottom = runs
            .last()
            .map_or(size, |run| run.line_top + run.line_height);
        let block_height = layout_bottom - layout_top;
        let y_shift = ((rect.h - block_height) * 0.5).max(0.);
        let start = if composing {
            0
        } else {
            state.anchor.min(state.cursor)
        };
        let end = if composing {
            0
        } else {
            state.anchor.max(state.cursor)
        };
        let mut composition_bounds: Option<(f32, f32)> = None;
        for run in runs {
            for glyph in run.glyphs {
                if composing && glyph.end > composition_start && glyph.start < composition_end {
                    composition_bounds = Some(match composition_bounds {
                        Some((left, right)) => (left.min(glyph.x), right.max(glyph.x + glyph.w)),
                        None => (glyph.x, glyph.x + glyph.w),
                    });
                }
                if glyph.end > start && glyph.start < end {
                    items.push(Instance::new(
                        Rect::new(rect.x + glyph.x - scroll_x, rect.y, glyph.w, rect.h),
                        clip,
                        [accent[0], accent[1], accent[2], 0.22],
                        0.,
                        1.,
                        0.,
                    ));
                }
                let p = glyph.physical(
                    (
                        rect.x - scroll_x,
                        rect.y + y_shift + run.line_y - layout_top,
                    ),
                    1.,
                );
                if let Some(g) = self.glyph(p.cache_key) {
                    let glyph_rect = Rect::new(
                        (p.x + g.left) as f32,
                        (p.y - g.top) as f32,
                        g.width as f32,
                        g.height as f32,
                    );
                    let mut instance = Instance::new(
                        glyph_rect,
                        clip,
                        if g.color { [1.; 4] } else { *color },
                        1.,
                        0.,
                        0.,
                    );
                    instance.uv = g.uv;
                    items.push(instance);
                }
            }
        }
        if let Some((left, right)) = composition_bounds {
            items.push(Instance::new(
                Rect::new(
                    rect.x + left - scroll_x,
                    rect.y + rect.h - 2.,
                    (right - left).max(1.),
                    1.,
                ),
                clip,
                accent,
                0.,
                0.,
                0.,
            ));
        }
        if *focused && caret_visible {
            items.push(Instance::new(
                Rect::new(
                    rect.x + caret_x - scroll_x,
                    rect.y + y_shift,
                    1.5,
                    block_height,
                ),
                clip,
                accent,
                0.,
                0.,
                0.,
            ));
        }
    }
    /// Maps pointer coordinates through the actual shaped layout, including bidi runs.
    pub fn text_hit(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        x: f32,
        y: f32,
        cursor: (usize, bool),
    ) -> (usize, bool) {
        self.text_hit_aligned(text, size, width, [x, y], cursor, natural_text_align(text))
    }

    pub fn text_hit_aligned(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        position: [f32; 2],
        cursor: (usize, bool),
        align: TextAlign,
    ) -> (usize, bool) {
        let [x, y] = position;
        let mut buffer = self.buffer(text, size, width, align);
        buffer.set_wrap(Wrap::None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let current = Cursor::new_with_affinity(0, cursor.0, Affinity::from_after(cursor.1));
        let caret = buffer
            .cursor_position(&current)
            .map_or(0., |position| position.0);
        let horizontal_scroll = ((caret + 1.5) - (width - 4.).max(1.)).max(0.);
        let hit = buffer
            .hit(x + horizontal_scroll, y)
            .unwrap_or_else(|| Cursor::new_with_affinity(0, text.len(), Affinity::Before));
        (hit.index, hit.affinity.after())
    }
    pub fn text_move(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        cursor: usize,
        cursor_after: bool,
        right: bool,
    ) -> Option<(usize, bool)> {
        self.text_move_aligned(
            text,
            size,
            width,
            (cursor, cursor_after),
            right,
            natural_text_align(text),
        )
    }

    pub fn text_move_aligned(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        cursor: (usize, bool),
        right: bool,
        align: TextAlign,
    ) -> Option<(usize, bool)> {
        let mut buffer = self.buffer(text, size, width, align);
        buffer.set_wrap(Wrap::None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let current = Cursor::new_with_affinity(0, cursor.0, Affinity::from_after(cursor.1));
        let mut stops = Vec::new();
        for (index, _) in text.grapheme_indices(true) {
            for affinity in [Affinity::Before, Affinity::After] {
                let stop = Cursor::new_with_affinity(0, index, affinity);
                if let Some((x, _)) = buffer.cursor_position(&stop) {
                    stops.push((stop, x));
                }
            }
        }
        for affinity in [Affinity::Before, Affinity::After] {
            let stop = Cursor::new_with_affinity(0, text.len(), affinity);
            if let Some((x, _)) = buffer.cursor_position(&stop) {
                stops.push((stop, x));
            }
        }
        stops.sort_by(|a, b| a.1.total_cmp(&b.1).then_with(|| a.0.index.cmp(&b.0.index)));
        let position = stops.iter().position(|(stop, _)| *stop == current)?;
        let next = if right {
            stops.get(position + 1)
        } else {
            position.checked_sub(1).and_then(|i| stops.get(i))
        }?;
        Some((next.0.index, next.0.affinity.after()))
    }
    pub fn text_cursor_x(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        cursor: usize,
        cursor_after: bool,
    ) -> Option<f32> {
        self.text_cursor_x_aligned(
            text,
            size,
            width,
            cursor,
            cursor_after,
            natural_text_align(text),
        )
    }

    pub fn text_cursor_x_aligned(
        &mut self,
        text: &str,
        size: f32,
        width: f32,
        cursor: usize,
        cursor_after: bool,
        align: TextAlign,
    ) -> Option<f32> {
        let mut buffer = self.buffer(text, size, width, align);
        buffer.set_wrap(Wrap::None);
        buffer.shape_until_scroll(&mut self.fonts, false);
        let cursor = Cursor::new_with_affinity(0, cursor, Affinity::from_after(cursor_after));
        buffer.cursor_position(&cursor).map(|position| position.0)
    }
}
