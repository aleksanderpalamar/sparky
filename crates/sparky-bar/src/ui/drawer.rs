use iced::advanced::Renderer as _;
use iced::advanced::widget::{Operation, Tree};
use iced::advanced::{Clipboard, Layout, Shell, Widget, layout, mouse, overlay, renderer};
use iced::{Element, Event, Length, Rectangle, Renderer, Size, Theme, Vector};

use crate::app::message::Message;

pub fn drawer<'a>(
    content: Element<'a, Message>,
    extent: f32,
    progress: f32,
) -> Element<'a, Message> {
    Element::new(Drawer {
        content,
        extent,
        progress: progress.clamp(0.0, 1.0),
    })
}

struct Drawer<'a> {
    content: Element<'a, Message>,
    extent: f32,
    progress: f32,
}

impl Widget<Message, Theme, Renderer> for Drawer<'_> {
    fn size(&self) -> Size<Length> {
        Size::new(Length::Fixed(self.extent * self.progress), Length::Fill)
    }

    fn children(&self) -> Vec<Tree> {
        vec![Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(std::slice::from_ref(&self.content));
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let Some(state) = tree.children.first_mut() else {
            return layout::Node::new(Size::ZERO);
        };
        let full = Size::new(self.extent, limits.max().height);
        let child = self.content.as_widget_mut().layout(
            state,
            renderer,
            &layout::Limits::new(Size::ZERO, full).width(self.extent),
        );
        let size = Size::new(
            (self.extent * self.progress).min(limits.max().width),
            child.size().height,
        );
        layout::Node::with_children(size, vec![child])
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let Some(visible) = layout
            .bounds()
            .intersection(viewport)
            .filter(|bounds| bounds.width > 0.0)
        else {
            return;
        };
        let (Some(state), Some(child)) = (tree.children.first(), layout.children().next()) else {
            return;
        };
        renderer.with_layer(visible, |renderer| {
            self.content.as_widget().draw(
                state,
                renderer,
                theme,
                style,
                child,
                clipped(cursor, visible),
                &visible,
            );
        });
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let (Some(state), Some(child)) = (tree.children.first_mut(), layout.children().next())
        else {
            return;
        };
        let visible = layout.bounds().intersection(viewport).unwrap_or_default();
        self.content.as_widget_mut().update(
            state,
            event,
            child,
            clipped(cursor, visible),
            renderer,
            clipboard,
            shell,
            &visible,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        if !cursor.is_over(layout.bounds()) {
            return mouse::Interaction::None;
        }
        let (Some(state), Some(child)) = (tree.children.first(), layout.children().next()) else {
            return mouse::Interaction::None;
        };
        self.content
            .as_widget()
            .mouse_interaction(state, child, cursor, viewport, renderer)
    }

    fn operate(
        &mut self,
        tree: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        let (Some(state), Some(child)) = (tree.children.first_mut(), layout.children().next())
        else {
            return;
        };
        self.content
            .as_widget_mut()
            .operate(state, child, renderer, operation);
    }

    fn overlay<'a>(
        &'a mut self,
        tree: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'a, Message, Theme, Renderer>> {
        let state = tree.children.first_mut()?;
        let child = layout.children().next()?;
        self.content
            .as_widget_mut()
            .overlay(state, child, renderer, viewport, translation)
    }
}

fn clipped(cursor: mouse::Cursor, visible: Rectangle) -> mouse::Cursor {
    match cursor.is_over(visible) && visible.width > 0.0 {
        true => cursor,
        false => mouse::Cursor::Unavailable,
    }
}
