use std::hash::Hash;
use std::marker::PhantomData;

use iced::advanced::layout::{Limits, Node};
use iced::advanced::widget::tree::{State, Tag};
use iced::advanced::widget::Tree;
use iced::advanced::{renderer, Clipboard, Layout, Shell, Widget};
use iced::mouse::Cursor;
use iced::widget::{container, scrollable};
use iced::{event, mouse, touch, Alignment, Element, Event, Length, Padding, Point, Rectangle};

mod style;

use style::StyleSheet;

#[derive(PartialEq, Clone, Copy, Default, Debug, Eq)]
pub enum Direction {
    #[default]
    Vertical,
    Horizontal,
}

/// The Private [`ListState`] Handles the State of the inner list.
#[derive(Debug, Clone, Default)]
pub struct ListState {
    /// Statehood of hovered_option
    pub hovered_option: Option<usize>,
    /// The index in the list of options of the last chosen Item Clicked for Processing
    pub selected_index: Option<usize>,
}

pub struct SelectionList<'a, Label, Message, Theme, Renderer = iced::Renderer>
where
    Label: Eq + Hash + Clone,
    Message: Clone,
    Renderer: renderer::Renderer,
    Theme: StyleSheet + 'a,
{
    /// The items we are going to display within this widget
    items: Vec<Element<'a, Message, Theme, Renderer>>,
    labels: Vec<Label>,
    on_selected: Box<dyn Fn(Label) -> Message + 'a>,
    selected: Option<usize>,
    manual_selection: Option<usize>,
    /// Style for Font colors and Box hover colors.
    pub style: <Theme as StyleSheet>::Style,
    item_width: f32,
    item_height: f32,
    width: Length,
    height: Length,
    alignment: Alignment,
    /// The padding Width
    padding: f32,
    direction: Direction,
    /// Shadow Type holder for Renderer.
    renderer: PhantomData<Renderer>,
}

impl<'a, Label, Message, Theme, Renderer> SelectionList<'a, Label, Message, Theme, Renderer>
where
    Label: Eq + Hash + Clone + 'a,
    Message: Clone + 'a,
    Renderer: renderer::Renderer + 'a,
    Theme: StyleSheet + scrollable::StyleSheet + container::StyleSheet + 'a,
{
    pub fn new(
        values: Vec<(Label, Element<'a, Message, Theme, Renderer>)>,
        on_selected: impl Fn(Label) -> Message + 'a,
    ) -> Self {
        Self::new_with_selection(values, on_selected, None)
    }

    pub fn new_with_selection(
        values: Vec<(Label, Element<'a, Message, Theme, Renderer>)>,
        on_selected: impl Fn(Label) -> Message + 'a,
        selection: Option<usize>,
    ) -> Self {
        let (labels, items) = values.into_iter().unzip();
        Self {
            labels,
            items,
            width: Length::Shrink,
            height: Length::Shrink,
            item_width: 0.,
            item_height: 0.,
            direction: Direction::Vertical,
            on_selected: Box::new(on_selected),
            manual_selection: selection,
            selected: selection,
            style: <Theme as StyleSheet>::Style::default(),
            padding: 0.,
            renderer: PhantomData,
            alignment: Alignment::Start,
        }
    }

    #[must_use]
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    #[must_use]
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    #[must_use]
    pub fn item_height(mut self, item_height: f32) -> Self {
        self.item_height = item_height;
        self
    }

    #[must_use]
    pub fn item_width(mut self, item_width: f32) -> Self {
        self.item_width = item_width;
        self
    }

    #[must_use]
    pub fn direction(mut self, direction: Direction) -> Self {
        self.direction = direction;
        self
    }

    // pub fn view(self) -> Element<'a, Message, Theme, Renderer> {
    //     let scrollable_direction = match self.direction {
    //         Direction::Vertical => scrollable::Direction::Vertical(Properties::default()),
    //         Direction::Horizontal => scrollable::Direction::Horizontal(Properties::default()),
    //     };
    //     let container = Container::new(
    //         Scrollable::new(
    //             list::List::new(
    //                 self.values,
    //                 self.on_selected,
    //                 self.manual_selection,
    //                 self.item_width,
    //                 self.item_height,
    //             )
    //             .direction(self.direction),
    //         )
    //         .direction(scrollable_direction)
    //         .id(self.scroll_id.clone()),
    //     )
    //     .width(self.width)
    //     .height(self.height);
    //     container.into()
    // }
}

impl<'a, Label, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for SelectionList<'a, Label, Message, Theme, Renderer>
where
    Label: Clone + Eq + Hash + 'a,
    Renderer: renderer::Renderer,
    Theme: StyleSheet + 'a,
    Message: std::clone::Clone,
{
    fn tag(&self) -> Tag {
        Tag::of::<ListState>()
    }

    fn state(&self) -> State {
        {
            let state: ListState = ListState {
                hovered_option: None,
                selected_index: self.selected,
            };
            State::Some(Box::new(state))
        }
    }

    fn size(&self) -> iced::Size<Length> {
        match self.direction {
            Direction::Vertical => iced::Size {
                width: Length::Fill,
                height: Length::Shrink,
            },
            Direction::Horizontal => iced::Size {
                width: Length::Shrink,
                height: Length::Fill,
            },
        }
    }
    fn children(&self) -> Vec<Tree> {
        self.items.iter().map(Tree::new).collect()
    }

    fn diff(&self, tree: &mut Tree) {
        tree.diff_children(&self.items);
    }
    fn on_event(
        &mut self,
        state: &mut Tree,
        event: Event,
        layout: Layout<'_>,
        cursor: Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn Clipboard,
        shell: &mut Shell<Message>,
        viewport: &Rectangle,
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
                        if self.labels.get(id).is_some() {
                            list_state.selected_index = Some(id);
                        }
                    }

                    status =
                        list_state
                            .selected_index
                            .as_ref()
                            .map_or(event::Status::Ignored, |last| {
                                if let Some(option) = self.labels.get(*last) {
                                    shell.publish((self.on_selected)(option.clone()));
                                    event::Status::Captured
                                } else {
                                    event::Status::Ignored
                                }
                            });
                }
                _ => {}
            }
        } else {
            list_state.hovered_option = None;
        }
        // In addition to handling the events associated with selecting items
        // from the list, we also need to handle events that occur within each
        // item. This iterates over each item to handle the events there.
        status.merge(
            self.items
                .iter_mut()
                .zip(layout.children())
                .enumerate()
                .fold(event::Status::Ignored, |status, (index, (item, layout))| {
                    status.merge(item.as_widget_mut().on_event(
                        &mut state.children[index],
                        event.clone(),
                        layout,
                        cursor,
                        renderer,
                        clipboard,
                        shell,
                        viewport,
                    ))
                }),
        )
    }

    fn layout(
        &self,
        tree: &mut iced::advanced::widget::Tree,
        renderer: &Renderer,
        limits: &iced::advanced::layout::Limits,
    ) -> iced::advanced::layout::Node {
        let padding = Padding::from(self.padding);
        #[allow(clippy::cast_precision_loss)] // TODO: possible precision loss
        // let line_minimal_length = self.line_minimal_length;
        // let line_minimal_length = 0.;
        let limits = limits.shrink(padding).width(self.width).height(self.height);
        // .max_width(self.max_width)
        // .max_height(self.max_height);
        // let max_width = limits.max().width;
        let mut children = tree.children.iter_mut();
        // let mut curse = padding.left;
        // let deep_curse = padding.left;
        // let mut current_line_height = line_minimal_length;
        // let mut current_line_height = 0.0_f32;
        let item_size = iced::Size {
            width: self.item_width,
            height: self.item_height,
        };
        let nodes: Vec<Node> = self
            .items
            .iter()
            .enumerate()
            .map(|(index, elem)| {
                let node_limit = Limits::new(item_size, item_size);
                let mut node = elem.as_widget().layout(
                    children.next().expect("wrap missing expected child"),
                    renderer,
                    &node_limit,
                );

                // let size = node.size();

                // let offset_init = size.width + spacing;
                // let offset = curse + offset_init;

                node.move_to_mut(Point::new(self.padding, self.item_height * index as f32));
                // current_line_height = current_line_height.max(size.height);

                node
            })
            .collect();
        // if end != start {
        //     align.push((start..end, current_line_height));
        // }
        // for (range, max_length) in align {
        //     nodes[range].iter_mut().for_each(|node| {
        //         let size = node.size();
        //         let space = iced::Size::new(size.width, max_length);
        //         node.align_mut(Alignment::Start, self.alignment, space);
        //     });
        // }
        let (width, height) = (self.item_width, self.item_height * self.items.len() as f32);
        let size = limits.resolve(self.width, self.height, iced::Size::new(width, height));

        Node::with_children(size.expand(padding), nodes)
    }

    fn draw(
        &self,
        state: &iced::advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: iced::advanced::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let list_state = state.state.downcast_ref::<ListState>();

        for (((index, child), state), layout) in self
            .items
            .iter()
            .enumerate()
            .zip(&state.children)
            .zip(layout.children())
        {
            // Determine the style of each element
            let is_selected = list_state.selected_index == Some(index);
            let is_hovered = list_state.hovered_option == Some(index);

            let (text_color, background_colour) = if is_selected {
                (
                    theme.style(self.style).selected_text_color,
                    theme.style(self.style).selected_background,
                )
            } else if is_hovered {
                (
                    theme.style(self.style).hovered_text_color,
                    theme.style(self.style).hovered_background,
                )
            } else {
                (
                    theme.style(self.style).text_color,
                    theme.style(self.style).background,
                )
            };
            let border = iced::Border {
                color: iced::Color::from_rgb(0.1, 0.1, 0.1),
                width: 1.0,
                radius: 5.0.into(),
            };

            // Render a the background of the item first, so it remains behind the image
            renderer.fill_quad(
                renderer::Quad {
                    border,
                    bounds: layout.bounds(),
                    ..Default::default()
                },
                background_colour,
            );

            let style = renderer::Style { text_color };
            child
                .as_widget()
                .draw(state, renderer, theme, &style, layout, cursor, viewport);
        }
    }
}

impl<'a, Label, Message, Theme, Renderer> From<SelectionList<'a, Label, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Label: Clone + Eq + Hash + 'a,
    Renderer: 'a + renderer::Renderer,
    Message: 'a + std::clone::Clone,
    Theme: StyleSheet + 'a,
{
    fn from(
        list: SelectionList<'a, Label, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(list)
    }
}
