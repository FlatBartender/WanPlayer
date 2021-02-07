// From https://github.com/DankBSD/waysmoke/commit/aa74cc51cea3dad000211cde4eae99affd126e29

//! The antidote to iced's annoying rigidity and inflexibility,
//! the equivalent of anything.addEventListener('mouseover', ..) :P

use iced_native::*;
use std::hash::Hash;

pub use iced_native::{
    event::*,
    mouse,
};

pub struct AddEventListener<'a, Message, Renderer: self::Renderer> {
    content: Element<'a, Message, Renderer>,
    listeners: Vec<(event::Event, Message)>,
}

impl<'a, Message, Renderer> AddEventListener<'a, Message, Renderer>
where
    Renderer: self::Renderer,
{
    pub fn new<T>(content: T) -> Self
    where
        T: Into<Element<'a, Message, Renderer>>,
    {
        AddEventListener {
            content: content.into(),
            listeners: Vec::new(),
        }
    }

    pub fn add_event_listener(mut self, event: Event, msg: Message) -> Self {
        self.listeners.push((event, msg));
        self
    }
}

impl<'a, Message, Renderer> Widget<Message, Renderer> for AddEventListener<'a, Message, Renderer>
where
    Renderer: self::Renderer,
    Message: Clone,
{
    fn width(&self) -> Length {
        Length::Shrink
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        let content = self.content.layout(renderer, &limits.loose());
        let size = limits.resolve(content.size());
        layout::Node::with_children(size, vec![content])
    }

    fn on_event(
        &mut self,
        event: Event,
        layout: Layout<'_>,
        cursor_position: Point,
        messages: &mut Vec<Message>,
        renderer: &Renderer,
        clipboard: Option<&dyn Clipboard>,
    ) -> event::Status {
        for (e, m) in self.listeners.iter() {
            if e == &event {
                messages.push(m.clone());
            }
        }

        self.content.on_event(
            event,
            layout.children().next().unwrap(),
            cursor_position,
            messages,
            renderer,
            clipboard,
        )
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        defaults: &Renderer::Defaults,
        layout: Layout<'_>,
        cursor_position: Point,
        viewport: &Rectangle,
    ) -> Renderer::Output {
        renderer.draw(
            defaults,
            cursor_position,
            &self.content,
            layout.children().next().unwrap(),
            viewport,
        )
    }

    fn hash_layout(&self, state: &mut Hasher) {
        struct Marker;
        std::any::TypeId::of::<Marker>().hash(state);

        self.content.hash_layout(state);
    }
}

pub trait Renderer: iced_native::Renderer {
    fn draw<Message>(
        &mut self,
        defaults: &Self::Defaults,
        cursor_position: Point,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
    ) -> Self::Output;
}

impl<'a, Message, Renderer> From<AddEventListener<'a, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Renderer: 'a + self::Renderer,
    Message: 'a + Clone,
{
    fn from(x: AddEventListener<'a, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(x)
    }
}

impl<B> Renderer for iced_graphics::Renderer<B>
where
    B: iced_graphics::Backend,
{
    fn draw<Message>(
        &mut self,
        defaults: &iced_graphics::Defaults,
        cursor_position: Point,
        content: &Element<'_, Message, Self>,
        content_layout: Layout<'_>,
        viewport: &Rectangle,
    ) -> Self::Output {
        content.draw(self, defaults, content_layout, cursor_position, viewport)
    }
}
