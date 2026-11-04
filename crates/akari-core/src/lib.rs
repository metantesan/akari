//! Platform-independent widgets, layout, interaction, and application contracts.

mod application;
mod element;
mod geometry;
mod theme;
mod ui;

pub use application::*;
pub use element::*;
pub use geometry::*;
pub use theme::*;
pub use ui::*;

/// Build Akari's custom widgets with nested, declarative syntax.
///
/// ```ignore
/// akari::view! {
///     Column(0) [
///         akari::view! { Text(1, "Hello", 28.) },
///         akari::view! { Button(2, "Add", Message::Add).width(72.) },
///     ].padding(16.).gap(12.)
/// }
/// ```
#[macro_export]
macro_rules! view {
    (Row($key:expr) [$($child:expr),* $(,)?] $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::row($key, vec![$($child),*]);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Column($key:expr) [$($child:expr),* $(,)?] $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::column($key, vec![$($child),*]);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Column($key:expr) from[$children:expr] $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::column($key, $children);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Scroll($key:expr) [$($child:expr),* $(,)?] $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::scroll($key, vec![$($child),*]);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Stack($key:expr) [$($child:expr),* $(,)?] $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::stack($key, vec![$($child),*]);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Container($key:expr, $child:expr) $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::container($key, $child);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Text($key:expr, $text:expr, $size:expr) $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::text($key, $text, $size);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (Button($key:expr, $label:expr, $message:expr) $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::button($key, $label, $message);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
    (TextField($key:expr, $value:expr, $placeholder:expr, $on_change:expr) $(.$method:ident($($arg:expr),*))* $(,)?) => {{
        let widget = $crate::Element::text_field($key, $value, $placeholder, $on_change);
        $(let widget = widget.$method($($arg),*);)*
        widget
    }};
}
