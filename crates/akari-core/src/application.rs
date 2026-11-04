//! Elm-style application contracts and asynchronous task messages.

use super::*;

/// A blocking job executed off the event loop. Its result is delivered as a message.
pub struct Task<M>(pub Box<dyn FnOnce() -> M + Send>);
impl<M> Task<M> {
    pub fn perform(job: impl FnOnce() -> M + Send + 'static) -> Self {
        Self(Box::new(job))
    }
}
/// A stateful application component with the same model/update/view cycle used
/// by Relm4's `SimpleComponent`. Implement this trait for the application model;
/// the runtime adapts it to [`Application`].
pub trait SimpleComponent: 'static {
    type Input: Clone + Send + 'static;

    fn init() -> Self
    where
        Self: Sized + Default,
    {
        Self::default()
    }
    fn title(&self) -> &str {
        "Akari"
    }
    fn update(&mut self, input: Self::Input) -> Vec<Task<Self::Input>>;
    fn view(&self, size: [f32; 2]) -> Element<Self::Input>;
    fn renderer_info(&mut self, _info: RendererInfo) {}
    fn theme(&self) -> Theme {
        Theme::default()
    }
    fn theme_preference(&self) -> ThemePreference {
        ThemePreference::System
    }
    fn fonts(&self) -> Vec<&'static [u8]> {
        vec![]
    }
    fn startup(&mut self) -> Vec<Task<Self::Input>> {
        vec![]
    }
}

/// Runtime interface. Most applications should implement [`SimpleComponent`].
pub trait Application: 'static {
    type Message: Clone + Send + 'static;
    fn title(&self) -> &str {
        "Akari"
    }
    fn update(&mut self, message: Self::Message) -> Vec<Task<Self::Message>>;
    fn view(&self, size: [f32; 2]) -> Element<Self::Message>;
    fn renderer_info(&mut self, _info: RendererInfo) {}
    fn theme(&self) -> Theme {
        Theme::default()
    }
    fn theme_preference(&self) -> ThemePreference {
        ThemePreference::System
    }
    fn fonts(&self) -> Vec<&'static [u8]> {
        vec![]
    }
    fn startup(&mut self) -> Vec<Task<Self::Message>> {
        vec![]
    }
}

impl<C: SimpleComponent> Application for C {
    type Message = C::Input;

    fn title(&self) -> &str {
        SimpleComponent::title(self)
    }
    fn update(&mut self, message: Self::Message) -> Vec<Task<Self::Message>> {
        SimpleComponent::update(self, message)
    }
    fn view(&self, size: [f32; 2]) -> Element<Self::Message> {
        SimpleComponent::view(self, size)
    }
    fn renderer_info(&mut self, info: RendererInfo) {
        SimpleComponent::renderer_info(self, info)
    }
    fn theme(&self) -> Theme {
        SimpleComponent::theme(self)
    }
    fn theme_preference(&self) -> ThemePreference {
        SimpleComponent::theme_preference(self)
    }
    fn fonts(&self) -> Vec<&'static [u8]> {
        SimpleComponent::fonts(self)
    }
    fn startup(&mut self) -> Vec<Task<Self::Message>> {
        SimpleComponent::startup(self)
    }
}
