use super::*;
use crate::geometry::{TEXT_FIELD_HORIZONTAL_INSET, directional_align};

impl<M: Clone> Ui<M> {
    pub fn semantics(&self) -> Vec<Semantic> {
        self.nodes
            .iter()
            .map(|n| {
                let (default_role, default_label, actions) = match &n.element.kind {
                    Kind::Text(t, _) => (Role::Text, t.clone(), vec![]),
                    Kind::Button(t, _) => (Role::Button, t.clone(), vec!["activate"]),
                    Kind::IconButton(icon, _) => {
                        (Role::Button, icon.name.to_owned(), vec!["activate"])
                    }
                    Kind::Tab { label, .. } => (Role::Tab, label.clone(), vec!["activate"]),
                    Kind::TextField {
                        value, placeholder, ..
                    } => (
                        Role::TextField,
                        if value.is_empty() {
                            placeholder.clone()
                        } else {
                            value.clone()
                        },
                        vec!["focus", "set_value"],
                    ),
                    Kind::Scroll => (Role::ScrollView, String::new(), vec!["scroll"]),
                    Kind::Image(_) => (Role::Image, String::new(), vec![]),
                    Kind::Icon(icon) => (Role::Image, icon.name.to_owned(), vec![]),
                    _ => (Role::Group, String::new(), vec![]),
                };
                Semantic {
                    key: n.element.key,
                    role: n.element.semantic_role.unwrap_or(default_role),
                    label: n.element.label.clone().unwrap_or(default_label),
                    bounds: n.rect.intersect(n.clip),
                    focused: self.focused == Some(n.element.key),
                    selected: n.element.selected,
                    checked: n.element.checked,
                    value: match &n.element.kind {
                        Kind::ProgressBar(value) => Some(*value),
                        _ => None,
                    },
                    actions,
                }
            })
            .collect()
    }
    pub fn draw(&self) -> Vec<Draw> {
        let mut out = vec![];
        for n in &self.nodes {
            let e = &n.element;
            let r = n.rect;
            let c = n.clip;
            if c.w <= 0. || c.h <= 0. {
                continue;
            }
            let focused = self.focused == Some(e.key);
            let color = if self.pressed == Some(e.key) {
                self.theme.accent
            } else if self.hovered == Some(e.key) {
                self.theme.surface.map(|v| {
                    if self.theme.background[0] > 0.5 {
                        (v - 0.045).max(0.)
                    } else {
                        (v + 0.055).min(1.)
                    }
                })
            } else {
                self.theme.surface
            };
            if let Surface::Glass(style) = e.surface {
                out.push(Draw::Glass {
                    rect: r,
                    clip: c,
                    style,
                });
            }
            if matches!(e.surface, Surface::Card) {
                out.push(Draw::Quad {
                    rect: r,
                    clip: c,
                    color: self.theme.surface,
                    radius: 14.,
                    border: 0.,
                });
                out.push(Draw::Quad {
                    rect: r,
                    clip: c,
                    color: self.theme.outline,
                    radius: 14.,
                    border: 1.,
                });
            }
            if focused && !matches!(e.kind, Kind::TextField { .. }) {
                out.push(Draw::Quad {
                    rect: r,
                    clip: c,
                    color: self.theme.accent,
                    radius: 12.,
                    border: 2.,
                });
            }
            match &e.kind {
                Kind::Button(label, _) => {
                    if matches!(e.surface, Surface::Plain) {
                        out.push(Draw::Quad {
                            rect: r.inset(3.),
                            clip: c,
                            color,
                            radius: 10.,
                            border: 0.,
                        });
                    }
                    out.push(Draw::Text {
                        rect: r.inset(8.),
                        clip: c,
                        text: label.clone(),
                        size: 16.,
                        color: self.theme.text,
                        align: directional_align(e.direction, e.text_align),
                    });
                }
                Kind::IconButton(icon, _) => {
                    if matches!(e.surface, Surface::Plain) {
                        out.push(Draw::Quad {
                            rect: r.inset(3.),
                            clip: c,
                            color,
                            radius: 10.,
                            border: 0.,
                        });
                    }
                    let side = r.w.min(r.h).min(24.);
                    out.push(Draw::Icon {
                        rect: Rect::new(
                            r.x + (r.w - side) * 0.5,
                            r.y + (r.h - side) * 0.5,
                            side,
                            side,
                        ),
                        clip: c,
                        icon: *icon,
                        color: self.theme.text,
                    });
                }
                Kind::Tab {
                    label,
                    icon,
                    selected,
                    ..
                } => {
                    let center_x = r.x + r.w * 0.5;
                    if *selected {
                        out.push(Draw::Quad {
                            rect: r.inset(4.),
                            clip: c,
                            color: [
                                self.theme.accent[0],
                                self.theme.accent[1],
                                self.theme.accent[2],
                                0.12,
                            ],
                            radius: 14.,
                            border: 0.,
                        });
                        out.push(Draw::Quad {
                            rect: Rect::new(center_x - 14., r.y + r.h - 4., 28., 2.),
                            clip: c,
                            color: self.theme.accent,
                            radius: 2.,
                            border: 0.,
                        });
                    }
                    out.push(Draw::Icon {
                        rect: Rect::new(center_x - 10., r.y + 8., 20., 20.),
                        clip: c,
                        icon: *icon,
                        color: if *selected {
                            self.theme.accent
                        } else {
                            self.theme.muted
                        },
                    });
                    out.push(Draw::Text {
                        rect: Rect::new(r.x + 2., r.y + 32., (r.w - 4.).max(0.), 20.),
                        clip: c,
                        text: label.clone(),
                        size: 12.,
                        color: if *selected {
                            self.theme.accent
                        } else {
                            self.theme.muted
                        },
                        align: TextAlign::Center,
                    });
                }
                Kind::Text(text, size) => out.push(Draw::Text {
                    rect: r,
                    clip: c,
                    text: text.clone(),
                    size: *size,
                    color: self.theme.text,
                    align: directional_align(e.direction, e.text_align),
                }),
                Kind::ProgressBar(value) => {
                    let radius = r.h * 0.5;
                    out.push(Draw::Quad {
                        rect: r,
                        clip: c,
                        color: self.theme.outline,
                        radius,
                        border: 0.,
                    });
                    out.push(Draw::Quad {
                        rect: Rect::new(r.x, r.y, r.w * value.clamp(0., 1.), r.h),
                        clip: c,
                        color: self.theme.accent,
                        radius,
                        border: 0.,
                    });
                }
                Kind::TextField {
                    value, placeholder, ..
                } => {
                    let vertical_pad = ((r.h - 22.4).max(0.) * 0.5).clamp(6., 13.);
                    out.push(Draw::Quad {
                        rect: r.inset(3.),
                        clip: c,
                        color: if focused {
                            self.theme.background
                        } else {
                            self.theme.surface
                        },
                        radius: 10.,
                        border: if focused { 1.5 } else { 0. },
                    });
                    if focused {
                        out.push(Draw::Quad {
                            rect: r.inset(3.),
                            clip: c,
                            color: self.theme.accent,
                            radius: 10.,
                            border: 1.5,
                        });
                    }
                    let s = &self.states[&e.key];
                    let mut text = if value.is_empty() && s.composition.is_empty() {
                        placeholder.clone()
                    } else {
                        value.clone()
                    };
                    if !s.composition.is_empty() {
                        let start = boundary(&text, s.anchor.min(s.cursor));
                        let end = boundary(&text, s.anchor.max(s.cursor));
                        text.replace_range(start..end, &s.composition);
                    }
                    out.push(Draw::TextField {
                        rect: Rect::new(
                            r.x + TEXT_FIELD_HORIZONTAL_INSET,
                            r.y + vertical_pad,
                            (r.w - 2. * TEXT_FIELD_HORIZONTAL_INSET).max(0.),
                            (r.h - 2. * vertical_pad).max(0.),
                        ),
                        clip: c.intersect(r.inset(2.)),
                        text,
                        size: 16.,
                        color: if value.is_empty() && s.composition.is_empty() {
                            self.theme.muted
                        } else {
                            self.theme.text
                        },
                        align: directional_align(e.direction, e.text_align),
                        state: s.clone(),
                        focused,
                    });
                }
                Kind::Image(image) => out.push(Draw::Image {
                    rect: r,
                    clip: c,
                    image: image.clone(),
                }),
                Kind::Icon(icon) => out.push(Draw::Icon {
                    rect: r,
                    clip: c,
                    icon: *icon,
                    color: self.theme.text,
                }),
                _ => {}
            }
        }
        out
    }
    pub fn editing(&self) -> Option<(Key, String, WidgetState, Rect)> {
        let (key, text, state, rect, _) = self.editing_aligned()?;
        Some((key, text, state, rect))
    }
    pub fn editing_aligned(&self) -> Option<(Key, String, WidgetState, Rect, TextAlign)> {
        self.nodes.iter().find_map(|n| {
            if self.focused == Some(n.element.key) {
                if let Kind::TextField { value, .. } = &n.element.kind {
                    let padding = TEXT_FIELD_HORIZONTAL_INSET;
                    let vertical_pad = ((n.rect.h - 22.4).max(0.) * 0.5).min(13.);
                    return Some((
                        n.element.key,
                        value.clone(),
                        self.states[&n.element.key].clone(),
                        Rect::new(
                            n.rect.x + padding,
                            n.rect.y + vertical_pad,
                            (n.rect.w - 2. * padding).max(0.),
                            (n.rect.h - 2. * vertical_pad).max(0.),
                        ),
                        directional_align(n.element.direction, n.element.text_align),
                    ));
                }
                None
            } else {
                None
            }
        })
    }
}
