//! Build and show dropdown `ListMenus`.
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::Limits;
use iced::advanced::mouse::{self, Cursor};
use iced::advanced::widget::tree::{State, Tag};
use iced::advanced::widget::Tree;
use iced::advanced::{layout, renderer, Clipboard, Layout, Shell, Widget};
use iced::{event, touch, Color, Element, Event, Length, Point, Rectangle, Size};

use super::StyleSheet;

/// The Private [`List`] Handles the Actual list rendering.
#[allow(missing_debug_implementations)]
pub struct List<'a, Label, Message, Renderer = iced::Renderer>
where
    Label: Clone + Eq + Hash,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub items: HashMap<Label, Element<'a, Message, Renderer>>,
    pub ordering: Vec<Label>,
    /// Hovered Item Pointer
    /// Style for Font colors and Box hover colors.
    pub style: <Renderer::Theme as StyleSheet>::Style,
    /// Function Pointer On Select to call on Mouse button press.
    pub on_selected: Box<dyn Fn(Label) -> Message>,
    /// The padding Width
    pub padding: f32,
    pub item_width: f32,
    pub item_height: f32,
    /// Set the Selected ID manually.
    pub selected: Option<Label>,
    /// Shadow Type holder for Renderer.
    pub renderer: PhantomData<Renderer>,
}

impl<'a, Label, Message, Renderer> List<'a, Label, Message, Renderer>
where
    Label: Clone + Eq + Hash,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new(
        items: Vec<Element<'a, Message, Renderer>>,
        labels: Vec<Label>,
        on_selected: impl Fn(Label) -> Message + 'static,
        item_width: f32,
        item_height: f32,
    ) -> Self {
        let items: HashMap<_, _> = labels.clone().into_iter().zip(items).collect();
        Self {
            items,
            item_width,
            item_height,
            ordering: labels,
            selected: None,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            on_selected: Box::new(on_selected),
            renderer: PhantomData,
            padding: 0.,
        }
    }
    pub fn items(&self) -> Vec<&Element<'a, Message, Renderer>> {
        self.ordering
            .iter()
            .filter_map(|i| self.items.get(i))
            .collect()
    }
}

/// The Private [`ListState`] Handles the State of the inner list.
#[derive(Debug, Clone, Default)]
pub struct ListState<Label: Hash + Eq + Clone> {
    /// Statehood of hovered_option
    pub hovered_option: Option<usize>,
    /// The index in the list of options of the last chosen Item Clicked for Processing
    pub last_selected_index: Option<Label>,
}

impl<'a, Label, Message, Renderer> Widget<Message, Renderer> for List<'a, Label, Message, Renderer>
where
    Label: Clone + Eq + Hash + 'static,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> Tag {
        Tag::of::<ListState<Label>>()
    }

    fn state(&self) -> State {
        {
            let state: ListState<Label> = ListState {
                hovered_option: None,
                last_selected_index: None,
            };
            State::Some(Box::new(state))
        }
    }
    fn children(&self) -> Vec<Tree> {
        self.items().into_iter().map(Tree::new).collect()
    }

    fn diff(&self, state: &mut Tree) {
        state.diff_children(&self.items());
        let list_state = state.state.downcast_mut::<ListState<Label>>();

        if let Some(id) = &self.selected {
            if let Some(_option) = self.items.get(id) {
                list_state.last_selected_index = Some(id.clone());
            } else {
                list_state.last_selected_index = None;
            }
        } else if let Some(id) = &list_state.last_selected_index {
            if let Some(_option) = self.items.get(id) {
            } else {
                list_state.last_selected_index = None;
            }
        }
    }

    fn width(&self) -> Length {
        Length::Fill
    }

    fn height(&self) -> Length {
        Length::Shrink
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        use std::f32;
        let limits = limits.height(Length::Fill).width(Length::Fill);

        #[allow(clippy::cast_precision_loss)]
        let intrinsic = Size::new(
            limits.fill().width,
            (self.item_height + (self.padding * 2.0)) * self.ordering.len() as f32,
        );
        let mut nodes: Vec<layout::Node> = Vec::with_capacity(self.ordering.len());
        nodes.resize(self.ordering.len(), layout::Node::default());

        for (index, (node, child)) in nodes.iter_mut().zip(self.items()).enumerate() {
            let child_limits = Limits::new(
                Size::new(self.item_width, self.item_height),
                Size::new(self.item_width, self.item_height),
            );

            *node = child.as_widget().layout(renderer, &child_limits);
            node.move_to(Point::new(0., index as f32 * self.item_height))
        }

        layout::Node::with_children(intrinsic, nodes)
    }

    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
        _viewport: &Rectangle,
    ) -> event::Status {
        let bounds = layout.bounds();
        let mut status = event::Status::Ignored;
        let list_state = state.state.downcast_mut::<ListState<Label>>();

        if let Some(cursor) = cursor.position_over(bounds) {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    list_state.hovered_option = Some(
                        ((cursor.y - bounds.y) / (self.item_height + (self.padding * 2.0)))
                            as usize,
                    );
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    dbg!(bounds);
                    dbg!(cursor);
                    list_state.hovered_option = Some(
                        ((cursor.y - bounds.y) / (self.item_height + (self.padding * 2.0)))
                            as usize,
                    );

                    if let Some(id) = list_state.hovered_option {
                        if let Some(option) = self.ordering.get(id) {
                            list_state.last_selected_index = Some(option.clone());
                        }
                    }

                    status = list_state.last_selected_index.as_ref().map_or(
                        event::Status::Ignored,
                        |last| {
                            if let Some(_option) = self.items.get(last) {
                                shell.publish((self.on_selected)(last.clone()));
                                event::Status::Captured
                            } else {
                                event::Status::Ignored
                            }
                        },
                    );
                }
                _ => {}
            }
        } else {
            list_state.hovered_option = None;
        }

        status
    }

    fn mouse_interaction(
        &self,
        _state: &Tree,
        layout: Layout<'_>,
        cursor: Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let bounds = layout.bounds();

        if bounds.contains(cursor.position().unwrap_or_default()) {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Renderer::Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: Cursor,
        viewport: &Rectangle,
    ) {
        use std::f32;

        let bounds = layout.bounds();
        let option_height = self.item_height + (self.padding * 2.0);
        let offset = viewport.y - bounds.y;
        let start = (offset / option_height) as usize;
        let end = ((offset + viewport.height) / option_height).ceil() as usize;
        let mut layout_iter = layout.children().skip(start);
        let visible_options = &self.ordering[start..end.min(self.ordering.len())];
        let list_state = state.state.downcast_ref::<ListState<Label>>();

        for (i, option) in visible_options.iter().enumerate() {
            let i = start + i;
            let is_selected = list_state
                .last_selected_index
                .as_ref()
                .map(|i| i == option)
                .unwrap_or(false);
            let is_hovered = list_state.hovered_option == Some(i);

            let bounds = Rectangle {
                x: bounds.x,
                y: bounds.y + option_height * i as f32,
                width: bounds.width,
                height: option_height,
            };

            if is_selected || is_hovered {
                renderer.fill_quad(
                    renderer::Quad {
                        bounds,
                        border_radius: (0.0).into(),
                        border_width: 0.0,
                        border_color: Color::TRANSPARENT,
                    },
                    if is_selected {
                        theme.style(self.style).selected_background
                    } else {
                        theme.style(self.style).hovered_background
                    },
                );
            }

            let text_color = if is_selected {
                theme.style(self.style).selected_text_color
            } else if is_hovered {
                theme.style(self.style).hovered_text_color
            } else {
                theme.style(self.style).text_color
            };

            let style = renderer::Style { text_color };

            self.items.get(option).unwrap().as_widget().draw(
                &state.children[i],
                renderer,
                theme,
                &style,
                layout_iter.next().unwrap(),
                cursor,
                &bounds,
            );
        }
    }
}

impl<'a, Label, Message, Renderer> From<List<'a, Label, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Label: Clone + Eq + Hash + 'static,
    Message: 'a,
    Renderer: 'a + renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(list: List<'a, Label, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(list)
    }
}
