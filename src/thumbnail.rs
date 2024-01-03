use std::collections::HashMap;

use camino::Utf8PathBuf;
use entity::Selection;
use itertools::Itertools;
use uuid::Uuid;

use crate::picture::PictureThumbnail;

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
            value = value || thumbnail.selection == Selection::Ignore;
        }
        if self.ordinary {
            value = value || thumbnail.selection == Selection::Ordinary;
        }
        if self.pick {
            value = value || thumbnail.selection == Selection::Pick;
        }
        if self.hidden {
            value = value && !thumbnail.hidden
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

/// Provide an o
#[derive(Debug, Default)]
pub struct ThumbnailData {
    // All the thumbnails that have been loaded
    thumbnails: HashMap<Uuid, PictureThumbnail>,
    // The filter applied to the thumbnails that have been loaded
    filter: ThumbnailFilter,
    // The sort ordering of the thumbnails
    sort: Order,
}

impl ThumbnailData {
    pub fn positions(&self) -> Vec<Uuid> {
        let mut positions: Vec<_> = self
            .thumbnails
            .values()
            .filter(|t| self.filter.filter(t))
            .sorted()
            .map(|t| t.id)
            .collect();
        if self.sort == Order::Descending {
            positions.reverse()
        }
        positions
    }

    pub fn get_position(&self, id: &Uuid) -> Option<usize> {
        self.positions().iter().position(|i| i == id)
    }

    pub fn next(&mut self, id: Option<&Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            let positions = self.positions();
            positions
                .iter()
                .position(|i| i == id)
                .map(|i| (i + 1).clamp(0, positions.len() - 1))
                .and_then(|i| positions.get(i))
                .copied()
        } else {
            None
        }
    }

    pub fn prev(&mut self, id: Option<&Uuid>) -> Option<Uuid> {
        if let Some(id) = id {
            let positions = self.positions();
            self.positions()
                .iter()
                .position(|i| i == id)
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
        self.thumbnails = thumbnails.into_iter().map(|t| (t.id, t)).collect();
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
        self.thumbnails.get_mut(id).unwrap().selection = selection;
    }

    pub fn get_view(&self) -> Vec<PictureThumbnail> {
        self.positions()
            .iter()
            .map(|i| self.thumbnails.get(i).unwrap())
            .cloned()
            .collect()
    }

    pub fn get_filepath(&self, id: &Uuid) -> Option<Utf8PathBuf> {
        self.thumbnails.get(id).map(|t| t.filepath.clone())
    }
}
