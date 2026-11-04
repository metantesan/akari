use super::*;

impl<M: Clone> Ui<M> {
    pub(super) fn interactive(n: &Node<M>) -> bool {
        matches!(
            n.element.kind,
            Kind::Button(..) | Kind::IconButton(..) | Kind::Tab { .. } | Kind::TextField { .. }
        )
    }
    pub fn hit_test(&self, x: f32, y: f32) -> Option<Key> {
        self.nodes
            .iter()
            .rev()
            .find(|n| Self::interactive(n) && n.rect.contains(x, y) && n.clip.contains(x, y))
            .map(|n| n.element.key)
    }
    pub fn focused_bounds(&self) -> Option<Rect> {
        self.nodes
            .iter()
            .find(|n| Some(n.element.key) == self.focused)
            .map(|n| n.rect)
    }
    pub fn text_focused(&self) -> bool {
        self.nodes.iter().any(|n| {
            Some(n.element.key) == self.focused && matches!(n.element.kind, Kind::TextField { .. })
        })
    }
    pub fn event(&mut self, event: Event) -> Vec<M> {
        let mut messages = vec![];
        match event {
            Event::PointerMove(x, y) => self.hovered = self.hit_test(x, y),
            Event::PointerDown(x, y) => {
                self.pressed = self.hit_test(x, y);
                self.focused = self.pressed;
                if let Some(key) = self.focused
                    && let Some(n) = self.nodes.iter().find(|n| n.element.key == key)
                    && let Kind::TextField { value, .. } = &n.element.kind
                {
                    let s = self.states.get_mut(&key).expect("value must be present");
                    s.cursor = value.len();
                    s.cursor_after = false;
                    s.anchor = s.cursor;
                    s.anchor_after = s.cursor_after;
                }
                self.reveal_focus();
            }
            Event::PointerUp(x, y) => {
                if let Some(key) = self.pressed.take()
                    && self.hit_test(x, y) == Some(key)
                    && let Some(n) = self.nodes.iter().find(|n| n.element.key == key)
                    && let Kind::Button(_, m)
                    | Kind::IconButton(_, m)
                    | Kind::Tab { on_select: m, .. } = &n.element.kind
                {
                    messages.push(m.clone());
                }
            }
            Event::Cancel | Event::FocusLost => {
                self.pressed = None;
                for s in self.states.values_mut() {
                    s.composition.clear();
                }
            }
            Event::Scroll(x, y, delta) => {
                let scroll_keys: Vec<_> = self
                    .nodes
                    .iter()
                    .rev()
                    .filter(|node| {
                        matches!(node.element.kind, Kind::Scroll) && node.clip.contains(x, y)
                    })
                    .map(|node| node.element.key)
                    .collect();
                let mut remaining = delta;
                for key in scroll_keys {
                    let Some(state) = self.states.get_mut(&key) else {
                        continue;
                    };
                    let old = state.scroll;
                    state.scroll = (old + remaining).clamp(0., state.scroll_max);
                    remaining -= state.scroll - old;
                    if remaining.abs() < 0.01 {
                        break;
                    }
                }
            }
            Event::Tab(reverse) => {
                let keys: Vec<_> = self
                    .nodes
                    .iter()
                    .filter(|n| Self::interactive(n))
                    .map(|n| n.element.key)
                    .collect();
                if !keys.is_empty() {
                    let index = self.focused.and_then(|k| keys.iter().position(|v| *v == k));
                    let i = match index {
                        Some(i) if reverse => (i + keys.len() - 1) % keys.len(),
                        Some(i) => (i + 1) % keys.len(),
                        None if reverse => keys.len() - 1,
                        None => 0,
                    };
                    self.focused = Some(keys[i]);
                    self.reveal_focus();
                }
            }
            Event::Activate => {
                if let Some(n) = self
                    .nodes
                    .iter()
                    .find(|n| Some(n.element.key) == self.focused)
                {
                    match &n.element.kind {
                        Kind::Button(_, m) => messages.push(m.clone()),
                        Kind::IconButton(_, m) => messages.push(m.clone()),
                        Kind::Tab { on_select, .. } => messages.push(on_select.clone()),
                        Kind::TextField {
                            on_submit: Some(m), ..
                        } => messages.push(m.clone()),
                        _ => {}
                    }
                }
            }
            Event::Composition(value) => {
                if let Some(s) = self.focused.and_then(|k| self.states.get_mut(&k)) {
                    s.composition = value;
                }
            }
            Event::Text(_) | Event::Edit(..) => {
                if let Some(n) = self
                    .nodes
                    .iter()
                    .find(|n| Some(n.element.key) == self.focused)
                    && let Kind::TextField {
                        value, on_change, ..
                    } = &n.element.kind
                {
                    let s = self
                        .states
                        .get_mut(&n.element.key)
                        .expect("value must be present");
                    let mut text = value.clone();
                    s.cursor = boundary(&text, s.cursor);
                    s.anchor = boundary(&text, s.anchor);
                    let (start, end) = (s.cursor.min(s.anchor), s.cursor.max(s.anchor));
                    match event {
                        Event::Text(insert) => {
                            text.replace_range(start..end, &insert.replace(['\n', '\r'], ""));
                            s.cursor = start + insert.replace(['\n', '\r'], "").len();
                            s.cursor_after = false;
                            s.anchor = s.cursor;
                            s.anchor_after = s.cursor_after;
                            s.composition.clear();
                        }
                        Event::Edit(key, shift) => {
                            let prev = |p| {
                                text.grapheme_indices(true)
                                    .map(|(i, _)| i)
                                    .rfind(|i| *i < p)
                                    .unwrap_or(0)
                            };
                            let next = |p| {
                                text.grapheme_indices(true)
                                    .map(|(i, _)| i)
                                    .find(|i| *i > p)
                                    .unwrap_or(text.len())
                            };
                            match key {
                                EditKey::SelectAll => {
                                    s.anchor = 0;
                                    s.anchor_after = false;
                                    s.cursor = text.len();
                                    s.cursor_after = false;
                                }
                                EditKey::Backspace | EditKey::Delete => {
                                    let (a, b) = if start != end {
                                        (start, end)
                                    } else if matches!(key, EditKey::Backspace) {
                                        (prev(start), end)
                                    } else {
                                        (start, next(end))
                                    };
                                    text.replace_range(a..b, "");
                                    s.cursor = a;
                                    s.cursor_after = false;
                                    s.anchor = a;
                                    s.anchor_after = s.cursor_after;
                                }
                                _ => {
                                    s.cursor = match key {
                                        EditKey::Left => {
                                            if !shift && start != end {
                                                start
                                            } else {
                                                prev(s.cursor)
                                            }
                                        }
                                        EditKey::Right => {
                                            if !shift && start != end {
                                                end
                                            } else {
                                                next(s.cursor)
                                            }
                                        }
                                        EditKey::Home => 0,
                                        EditKey::End => text.len(),
                                        _ => s.cursor,
                                    };
                                    s.cursor_after = false;
                                    if !shift {
                                        s.anchor = s.cursor;
                                        s.anchor_after = s.cursor_after;
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                    if text != *value {
                        messages.push(on_change(text));
                    }
                }
            }
        }
        messages
    }
    fn reveal_focus(&mut self) {
        let Some(bounds) = self.focused_bounds() else {
            return;
        };
        let Some(index) = self
            .nodes
            .iter()
            .position(|n| Some(n.element.key) == self.focused)
        else {
            return;
        };
        for n in self.nodes[..index].iter().rev() {
            if matches!(n.element.kind, Kind::Scroll)
                && n.rect.x <= bounds.x
                && n.rect.x + n.rect.w >= bounds.x + bounds.w
            {
                let delta = if bounds.y < n.rect.y {
                    bounds.y - n.rect.y
                } else {
                    (bounds.y + bounds.h - n.rect.y - n.rect.h).max(0.)
                };
                self.states
                    .get_mut(&n.element.key)
                    .expect("value must be present")
                    .scroll = (self.states[&n.element.key].scroll + delta).max(0.);
                break;
            }
        }
    }
}
