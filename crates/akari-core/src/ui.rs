//! Retained widget state, layout, hit testing, focus, and event routing.

use super::*;
use std::collections::{HashMap, HashSet};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Clone, Debug)]
pub struct WidgetState {
    pub scroll: f32,
    pub scroll_max: f32,
    pub cursor: usize,
    /// Affinity at a boundary shared by differently directed text runs.
    pub cursor_after: bool,
    pub anchor: usize,
    pub anchor_after: bool,
    pub composition: String,
}
#[derive(Clone, Copy, Debug)]
pub enum EditKey {
    Left,
    Right,
    Home,
    End,
    Backspace,
    Delete,
    SelectAll,
}
#[derive(Clone, Debug)]
pub enum Event {
    PointerDown(f32, f32),
    PointerMove(f32, f32),
    PointerUp(f32, f32),
    Cancel,
    Scroll(f32, f32, f32),
    Tab(bool),
    Activate,
    Text(String),
    Edit(EditKey, bool),
    Composition(String),
    FocusLost,
}
struct Node<M> {
    element: Element<M>,
    rect: Rect,
    clip: Rect,
}

fn collect_widget_keys<M>(element: &Element<M>, keys: &mut HashSet<Key>) {
    keys.insert(element.key);
    for child in &element.children {
        collect_widget_keys(child, keys);
    }
}

fn repair_duplicate_widget_keys<M>(
    element: &mut Element<M>,
    reserved: &HashSet<Key>,
    used: &mut HashSet<Key>,
    next_fallback: &mut Key,
    duplicates: &mut Vec<Key>,
) {
    let original = element.key;
    if !used.insert(original) {
        duplicates.push(original);
        while reserved.contains(next_fallback) || used.contains(next_fallback) {
            *next_fallback = next_fallback.wrapping_sub(1);
        }
        element.key = *next_fallback;
        used.insert(element.key);
        *next_fallback = next_fallback.wrapping_sub(1);
    }
    for child in &mut element.children {
        repair_duplicate_widget_keys(child, reserved, used, next_fallback, duplicates);
    }
}

pub struct Ui<M> {
    pub states: HashMap<Key, WidgetState>,
    pub focused: Option<Key>,
    /// Original keys that appeared more than once in the last rebuilt tree.
    /// Later occurrences receive temporary unique keys so a bad app tree does
    /// not panic the runtime. Fix these at the call site for stable state.
    pub duplicate_keys: Vec<Key>,
    nodes: Vec<Node<M>>,
    pressed: Option<Key>,
    hovered: Option<Key>,
    pub theme: Theme,
}
impl<M: Clone> Default for Ui<M> {
    fn default() -> Self {
        Self {
            states: HashMap::new(),
            focused: None,
            duplicate_keys: vec![],
            nodes: vec![],
            pressed: None,
            hovered: None,
            theme: Theme::default(),
        }
    }
}
mod draw;
mod events;
mod layout;

fn boundary(text: &str, index: usize) -> usize {
    text.grapheme_indices(true)
        .map(|(i, _)| i)
        .chain(std::iter::once(text.len()))
        .rfind(|i| *i <= index)
        .unwrap_or(0)
}
