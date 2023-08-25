//! Build and show dropdown `ListMenus`.
use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::Limits;
use iced::advanced::mouse::{self, Cursor};
use iced::advanced::widget::tree::{State, Tag};
use iced::advanced::widget::Tree;
use iced::advanced::{layout, renderer, Clipboard, Layout, Shell, Widget};
use iced::{event, touch, Color, Element, Event, Length, Point, Rectangle, Size};

use super::StyleSheet;

#[derive(PartialEq, Clone, Copy, Default, Debug, Eq)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

/// The Private [`List`] Handles the Actual list rendering.
#[allow(missing_debug_implementations)]
pub struct List<'a, Label, Message, Renderer = iced::Renderer>
where
    Label: Clone + Eq + Hash + 'a,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub items: Vec<Element<'a, Message, Renderer>>,
    pub labels: Vec<Label>,
    /// Hovered Item Pointer
    /// Style for Font colors and Box hover colors.
    pub style: <Renderer::Theme as StyleSheet>::Style,
    /// Function Pointer On Select to call on Mouse button press.
    pub on_selected: Box<dyn Fn(Label) -> Message + 'a>,
    /// The padding Width
    pub padding: f32,
    pub item_width: f32,
    pub item_height: f32,
    /// Set the Selected ID manually.
    pub selected: Option<Label>,
    /// Shadow Type holder for Renderer.
    pub renderer: PhantomData<Renderer>,
    pub direction: Direction,
}

impl<'a, Label, Message, Renderer> List<'a, Label, Message, Renderer>
where
    Label: Clone + Eq + Hash,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    pub fn new(
        values: Vec<(Label, Element<'a, Message, Renderer>)>,
        on_selected: impl Fn(Label) -> Message + 'a,
        item_width: f32,
        item_height: f32,
    ) -> Self {
        let (labels, items) = values.into_iter().unzip();
        Self {
            items,
            labels,
            item_width,
            item_height,
            selected: None,
            style: <Renderer::Theme as StyleSheet>::Style::default(),
            on_selected: Box::new(on_selected),
            renderer: PhantomData,
            padding: 0.,
            direction: Default::default(),
        }
    }

    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }
    #[must_use]
    pub fn item_width(mut self, item_width: f32) -> Self {
        self.item_width = item_width;
        self
    }
    #[must_use]
    pub fn item_height(mut self, item_height: f32) -> Self {
        self.item_height = item_height;
        self
    }
}

/// The Private [`ListState`] Handles the State of the inner list.
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Statehood of hovered_option
    pub hovered_option: Option<usize>,
    /// The index in the list of options of the last chosen Item Clicked for Processing
    pub last_selected_index: Option<usize>,
}

impl<'a, Label, Message, Renderer> Widget<Message, Renderer> for List<'a, Label, Message, Renderer>
where
    Label: Clone + Eq + Hash,
    Renderer: renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn tag(&self) -> Tag {
        Tag::of::<ListState>()
    }

    fn state(&self) -> State {
        {
            let state: ListState = ListState {
                hovered_option: None,
                last_selected_index: None,
            };
            State::Some(Box::new(state))
        }
    }
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }

    fn diff(&self, state: &mut Tree) {
        state.diff_children(&self.items);
        let list_state = state.state.downcast_mut::<ListState>();

        // if let Some(id) = &self.selected {
        //     list_state.last_selected_index = self.labels.iter().position(|x| x == id);
        // } else if let Some(_id) = &list_state.last_selected_index {
        //     // list_state.last_selected_index = self.ordering.iter().position(|x| x == id);
        // }
    }

    fn width(&self) -> Length {
        match self.direction {
            Direction::Vertical => Length::Fill,
            Direction::Horizontal => Length::Shrink,
        }
    }

    fn height(&self) -> Length {
        match self.direction {
            Direction::Vertical => Length::Shrink,
            Direction::Horizontal => Length::Fill,
        }
    }

    fn layout(&self, renderer: &Renderer, limits: &layout::Limits) -> layout::Node {
        use std::f32;
        let limits = limits.height(Length::Fill).width(Length::Fill);

        let intrinsic = match self.direction {
            Direction::Vertical => Size::new(
                limits.fill().width,
                (self.item_height + (self.padding * 2.0)) * self.labels.len() as f32,
            ),
            Direction::Horizontal => Size::new(
                (self.item_width + (self.padding * 2.0)) * self.labels.len() as f32,
                limits.fill().height,
            ),
        };
        let mut nodes: Vec<layout::Node> = Vec::with_capacity(self.labels.len());
        nodes.resize(self.labels.len(), layout::Node::default());

        for (index, (node, child)) in nodes.iter_mut().zip(self.items.iter()).enumerate() {
            let child_limits = Limits::new(
                Size::new(self.item_width, self.item_height),
                Size::new(self.item_width, self.item_height),
            );

            *node = child.as_widget().layout(renderer, &child_limits);
            match self.direction {
                Direction::Vertical => {
                    node.move_to(Point::new(0., index as f32 * self.item_height))
                }
                Direction::Horizontal => {
                    node.move_to(Point::new(index as f32 * self.item_width, 0.))
                }
            }
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
        let list_state = state.state.downcast_mut::<ListState>();

        if let Some(cursor) = cursor.position_over(bounds) {
            match event {
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    list_state.hovered_option = match self.direction {
                        Direction::Vertical => Some(
                            ((cursor.y - bounds.y) / (self.item_height + (self.padding * 2.0)))
                                as usize,
                        ),
                        Direction::Horizontal => Some(
                            ((cursor.x - bounds.x) / (self.item_width + (self.padding * 2.0)))
                                as usize,
                        ),
                    }
                }
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    list_state.hovered_option = match self.direction {
                        Direction::Vertical => Some(
                            ((cursor.y - bounds.y) / (self.item_height + (self.padding * 2.0)))
                                as usize,
                        ),
                        Direction::Horizontal => Some(
                            ((cursor.x - bounds.x) / (self.item_width + (self.padding * 2.0)))
                                as usize,
                        ),
                    };

                    if let Some(id) = list_state.hovered_option {
                        if let Some(_option) = self.labels.get(id) {
                            list_state.last_selected_index = Some(id);
                        }
                    }

                    status = list_state.last_selected_index.as_ref().map_or(
                        event::Status::Ignored,
                        |last| {
                            if let Some(option) = self.labels.get(*last) {
                                shell.publish((self.on_selected)(option.clone()));
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

        if cursor.is_over(bounds) {
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
        let option_width = self.item_width + (self.padding * 2.0);
        let (skip, take) = match self.direction {
            Direction::Vertical => {
                let offset = viewport.y - bounds.y;
                (
                    (offset / option_height).floor() as usize,
                    ((offset + viewport.height) / option_height).ceil() as usize,
                )
            }
            Direction::Horizontal => {
                let offset = viewport.x - bounds.x;
                (
                    (offset / option_width).floor() as usize,
                    ((offset + viewport.width) / option_width).ceil() as usize,
                )
            }
        };

        let list_state = state.state.downcast_ref::<ListState>();

        for (index, (item, layout)) in self
            .items
            .iter()
            .zip(layout.children())
            .enumerate()
            .skip(skip)
            .take(take)
        {
            let is_selected = list_state.last_selected_index == Some(index);
            let is_hovered = list_state.hovered_option == Some(index);

            let bounds = match self.direction {
                Direction::Vertical => Rectangle {
                    x: bounds.x,
                    y: bounds.y + option_height * index as f32,
                    width: option_width,
                    height: option_height,
                },
                Direction::Horizontal => Rectangle {
                    x: bounds.x + option_width * index as f32,
                    y: bounds.y,
                    width: option_width,
                    height: option_height,
                },
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

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border_radius: (0.0).into(),
                    border_width: 5.0,
                    border_color: Color::new(1., 0., 0., 1.),
                },
                theme.style(self.style).background,
            );

            let text_color = if is_selected {
                theme.style(self.style).selected_text_color
            } else if is_hovered {
                theme.style(self.style).hovered_text_color
            } else {
                theme.style(self.style).text_color
            };

            let style = renderer::Style { text_color };

            item.as_widget().draw(
                &state.children[index],
                renderer,
                theme,
                &style,
                layout,
                cursor,
                &bounds,
            );
        }
    }
}

impl<'a, Label, Message, Renderer> From<List<'a, Label, Message, Renderer>>
    for Element<'a, Message, Renderer>
where
    Label: Clone + Eq + Hash,
    Message: 'a,
    Renderer: 'a + renderer::Renderer,
    Renderer::Theme: StyleSheet,
{
    fn from(list: List<'a, Label, Message, Renderer>) -> Element<'a, Message, Renderer> {
        Element::new(list)
    }
}
