use super::*;

impl<M: Clone> Ui<M> {
    pub fn rebuild(&mut self, mut root: Element<M>, size: [f32; 2]) {
        let mut reserved = HashSet::new();
        collect_widget_keys(&root, &mut reserved);
        let mut used = HashSet::new();
        let mut next_fallback = Key::MAX;
        let mut duplicates = Vec::new();
        repair_duplicate_widget_keys(
            &mut root,
            &reserved,
            &mut used,
            &mut next_fallback,
            &mut duplicates,
        );
        self.duplicate_keys = duplicates;
        let previous_focus_order: Vec<_> = self
            .nodes
            .iter()
            .filter(|node| Self::interactive(node))
            .map(|node| node.element.key)
            .collect();
        let text_keys: HashSet<_> = self
            .nodes
            .iter()
            .filter(|n| matches!(n.element.kind, Kind::TextField { .. }))
            .map(|n| n.element.key)
            .collect();
        self.nodes.clear();
        let bounds = Rect::new(0., 0., size[0], size[1]);
        self.layout(root, bounds, bounds);
        for n in &self.nodes {
            if let Kind::TextField { value, .. } = &n.element.kind {
                let state = self
                    .states
                    .get_mut(&n.element.key)
                    .expect("value must be present");
                if !text_keys.contains(&n.element.key) {
                    state.cursor = value.len();
                    state.anchor = state.cursor;
                }
                state.cursor = boundary(value, state.cursor);
                state.anchor = boundary(value, state.anchor);
            }
        }
        let live: HashSet<_> = self.nodes.iter().map(|n| n.element.key).collect();
        self.states.retain(|k, _| live.contains(k));
        if self.focused.is_some_and(|k| !live.contains(&k)) {
            let focus_order: Vec<_> = self
                .nodes
                .iter()
                .filter(|node| Self::interactive(node))
                .map(|node| node.element.key)
                .collect();
            let old_index = self
                .focused
                .and_then(|focused| previous_focus_order.iter().position(|key| *key == focused));
            self.focused = old_index
                .and_then(|index| {
                    previous_focus_order[index + 1..]
                        .iter()
                        .find(|key| live.contains(key))
                        .or_else(|| {
                            previous_focus_order[..index]
                                .iter()
                                .rev()
                                .find(|key| live.contains(key))
                        })
                        .copied()
                })
                .or_else(|| focus_order.first().copied());
        }
    }
    fn layout(&mut self, e: Element<M>, rect: Rect, clip: Rect) {
        let clip = clip.intersect(rect);
        let inner = rect.inset(e.padding);
        let scroll = matches!(e.kind, Kind::Scroll);
        let row = matches!(e.kind, Kind::Row);
        let linear = row || matches!(e.kind, Kind::Column | Kind::Scroll);
        let offset = self.states.entry(e.key).or_default().scroll;
        self.nodes.push(Node {
            element: e.clone(),
            rect,
            clip,
        });
        if linear {
            let axis = if row { 0 } else { 1 };
            let avail = if row { inner.w } else { inner.h };
            let gap = e.gap * e.children.len().saturating_sub(1) as f32;
            let fixed: f32 = e
                .children
                .iter()
                .map(|c| match if row { c.width } else { c.height } {
                    Length::Fixed(v) => v,
                    Length::Fill => 0.,
                })
                .sum();
            let fills = e
                .children
                .iter()
                .filter(|c| matches!(if row { c.width } else { c.height }, Length::Fill))
                .count();
            let share = ((avail - fixed - gap).max(0.)) / fills.max(1) as f32;
            let total = fixed + gap + share * fills as f32;
            let max_scroll = (total - avail).max(0.);
            let offset = if scroll {
                offset.clamp(0., max_scroll)
            } else {
                0.
            };
            self.states
                .get_mut(&e.key)
                .expect("value must be present")
                .scroll = offset;
            self.states
                .get_mut(&e.key)
                .expect("value must be present")
                .scroll_max = max_scroll;
            let rtl = row && e.direction == Direction::RightToLeft;
            let mut pos = if rtl {
                inner.x + inner.w
            } else if row {
                inner.x
            } else {
                inner.y - offset
            };
            for child in e.children {
                let major = match if row { child.width } else { child.height } {
                    Length::Fixed(v) => v,
                    Length::Fill => share,
                };
                let cross = match if row { child.height } else { child.width } {
                    Length::Fixed(v) => v,
                    Length::Fill => {
                        if row {
                            inner.h
                        } else {
                            inner.w
                        }
                    }
                };
                let size = Constraints {
                    min: child.min,
                    max: [
                        child.max[0].min(inner.w),
                        if scroll {
                            child.max[1]
                        } else {
                            child.max[1].min(inner.h)
                        },
                    ],
                }
                .constrain(if row { [major, cross] } else { [cross, major] });
                let cross_avail = if row { inner.h } else { inner.w };
                let shift = match e.align {
                    Align::Start => 0.,
                    Align::Center => (cross_avail - size[1 - axis]) / 2.,
                    Align::End => cross_avail - size[1 - axis],
                }
                .max(0.);
                let r = if rtl {
                    pos -= size[0];
                    Rect::new(pos, inner.y + shift, size[0], size[1])
                } else if row {
                    Rect::new(pos, inner.y + shift, size[0], size[1])
                } else {
                    Rect::new(inner.x + shift, pos, size[0], size[1])
                };
                if rtl {
                    pos -= e.gap;
                } else {
                    pos += size[axis] + e.gap;
                }
                self.layout(child, r, clip);
            }
        } else {
            for child in e.children {
                let size = Constraints {
                    min: child.min,
                    max: [child.max[0].min(inner.w), child.max[1].min(inner.h)],
                }
                .constrain([
                    match child.width {
                        Length::Fill => inner.w,
                        Length::Fixed(v) => v,
                    },
                    match child.height {
                        Length::Fill => inner.h,
                        Length::Fixed(v) => v,
                    },
                ]);
                self.layout(child, Rect::new(inner.x, inner.y, size[0], size[1]), clip);
            }
        }
    }
}
