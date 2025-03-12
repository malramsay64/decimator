use std::{cell::RefCell, collections::HashMap, num::NonZero};

use camino::Utf8PathBuf;
use either::Either;
use entity::Selection;
use iced::{
    widget::{
        column, container, horizontal_space,
        image::{self, viewer},
        row, scrollable,
        scrollable::Id,
    },
    Element,
    Length::{self, Fill},
};
use itertools::Itertools;
use lru::LruCache;
use uuid::Uuid;

use crate::{
    picture::{self, PictureThumbnail},
    Message,
};

/// Provide the opportunity to filter thumbnails
///
/// Values are true when the filter is enabled and false
/// when they are disabled.
#[derive(Debug)]
struct ThumbnailFilter {
    ignore: bool,
    ordinary: bool,
    pick: bool,
    hidden: bool,
}

impl Default for ThumbnailFilter {
    fn default() -> Self {
        Self {
            ignore: true,
            ordinary: true,
            pick: true,
            hidden: false,
        }
    }
}

impl ThumbnailFilter {
    fn filter(&self, thumbnail: &PictureThumbnail) -> bool {
        let mut value = false;

        if self.ignore {
            value = value || thumbnail.data.selection == Selection::Ignore;
        }
        if self.ordinary {
            value = value || thumbnail.data.selection == Selection::Ordinary;
        }
        if self.pick {
            value = value || thumbnail.data.selection == Selection::Pick;
        }
        if self.hidden {
            value = value && !thumbnail.data.hidden
        }
        value
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
pub enum Order {
    #[default]
    Ascending,
    Descending,
}

#[derive(Debug, Default)]
pub enum Active {
    #[default]
    None,
    Single(Uuid),
    Multiple(Vec<Uuid>),
}

/// Provide an o
#[derive(Debug)]
pub struct ThumbnailView {
    // All the thumbnails that have been loaded
    thumbnails: HashMap<Uuid, PictureThumbnail>,
    // The filter applied to the thumbnails that have been loaded
    filter: ThumbnailFilter,
    // The sort ordering of the thumbnails
    sort: Order,
    // The items that have been selected
    selection: Active,

    scroller: Id,
    preview_cache: RefCell<lru::LruCache<Uuid, iced::widget::image::Handle>>,
}

impl ThumbnailView {
    pub fn new(cache_size: NonZero<usize>) -> Self {
        Self {
            thumbnails: Default::default(),
            filter: Default::default(),
            sort: Default::default(),
            selection: Default::default(),
            preview_cache: RefCell::new(LruCache::new(cache_size)),
            scroller: Id::unique(),
        }
    }

    pub fn positions(&self) -> impl Iterator<Item = Uuid> + use<'_> {
        let positions = self
            .thumbnails
            .values()
            .filter(|t| self.filter.filter(t))
            .sorted()
            .map(|t| t.data.id);
        if self.sort == Order::Descending {
            Either::Left(positions.rev())
        } else {
            Either::Right(positions)
        }
    }

    pub fn get_position(&self, id: Uuid) -> Option<usize> {
        self.positions().position(|i| i == id)
    }

    pub fn next(&mut self, id: Option<Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            let positions: Vec<_> = self.positions().collect();
            self.get_position(id)
                .map(|i| (i + 1).clamp(0, positions.len() - 1))
                .and_then(|i| positions.get(i))
                .copied()
        } else {
            None
        }
    }

    pub fn prev(&mut self, id: Option<Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            let positions: Vec<_> = self.positions().collect();
            self.get_position(id)
                .map(|i| (i - 1).clamp(0, positions.len() - 1))
                .and_then(|i| positions.get(i))
                .copied()
        } else {
            None
        }
    }

    pub fn pick(&self) -> bool {
        self.filter.pick
    }

    pub fn ordinary(&self) -> bool {
        self.filter.ordinary
    }

    pub fn ignore(&self) -> bool {
        self.filter.ignore
    }

    pub fn hidden(&self) -> bool {
        self.filter.hidden
    }

    pub fn set_thumbnails(&mut self, thumbnails: Vec<PictureThumbnail>) {
        self.thumbnails = thumbnails.into_iter().map(|t| (t.data.id, t)).collect();
    }

    pub fn set_ignore(&mut self, value: bool) {
        self.filter.ignore = value;
    }

    pub fn set_ordinary(&mut self, value: bool) {
        self.filter.ordinary = value;
    }

    pub fn set_pick(&mut self, value: bool) {
        self.filter.pick = value;
    }

    pub fn set_hidden(&mut self, value: bool) {
        self.filter.hidden = value;
    }

    pub fn set_selection(&mut self, id: &Uuid, selection: Selection) {
        self.thumbnails.get_mut(id).unwrap().data.selection = selection;
    }

    pub fn set_thumbnail(&mut self, id: &Uuid, handle: image::Handle) {
        self.thumbnails.get_mut(id).unwrap().handle = Some(handle)
    }

    pub fn get_view<'a>(&'a self) -> impl Iterator<Item = &'a PictureThumbnail> {
        self.positions().map(|i| self.thumbnails.get(&i).unwrap())
    }

    pub fn get_preview_view(&self) -> Element<'_, Message> {
        let preview: Element<'_, Message> = if let Active::Single(preview_id) = self.selection {
            // Check cache, and use if available
            let handle: image::Handle = match self.preview_cache.borrow_mut().get(&preview_id) {
                Some(x) => x.clone(),
                None => self
                    .thumbnails
                    .get(&preview_id)
                    .unwrap()
                    .handle
                    .clone()
                    .unwrap(),
            };
            viewer(handle)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        } else {
            horizontal_space().height(Length::Fill).into()
        };

        column![
            preview,
            scrollable(row(self.get_view().map(PictureThumbnail::view)).spacing(10))
                .id(self.scroller.clone())
                .direction(scrollable::Direction::Horizontal(
                    scrollable::Scrollbar::default(),
                ))
        ]
        .into()
    }

    pub fn get_grid_view(&self) -> Element<'_, Message> {
        scrollable(
            container(
                row(self.get_view().map(PictureThumbnail::view))
                    .spacing(10)
                    .wrap(),
            ), // .center_x(Fill),
        )
        .direction(scrollable::Direction::Vertical(
            scrollable::Scrollbar::new().width(2.).scroller_width(10.),
        ))
        .into()
    }

    pub fn get_filepath(&self, id: &Uuid) -> Option<Utf8PathBuf> {
        self.thumbnails.get(id).map(|t| t.data.filepath.clone())
    }
}
