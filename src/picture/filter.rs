use gtk::prelude::*;

use relm4::gtk;

use super::{PictureData, Selection};

#[derive(Default, Debug, Clone)]
pub enum FilterState {
    #[default]
    Include,
    Exclude,
}

impl FilterState {
    fn toggle(&mut self) {
        match self {
            FilterState::Include => std::mem::swap(self, &mut FilterState::Exclude),
            FilterState::Exclude => std::mem::swap(self, &mut FilterState::Include),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct FilterSettings {
    selection_ignore: FilterState,
    selection_ordinary: FilterState,
    selection_pick: FilterState,
}

impl FilterSettings {
    pub fn toggle_ignore(&mut self) {
        self.selection_ignore.toggle()
    }

    pub fn toggle_ordinary(&mut self) {
        self.selection_ordinary.toggle()
    }

    pub fn toggle_pick(&mut self) {
        self.selection_pick.toggle()
    }

    pub fn filter(&self, picture: &PictureData) -> bool {
        match picture.selection {
            Selection::Ignore => match self.selection_ignore {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
            Selection::Ordinary => match self.selection_ordinary {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
            Selection::Pick => match self.selection_pick {
                FilterState::Include => true,
                FilterState::Exclude => false,
            },
        }
    }
}
