pub mod selection_ext;
pub mod typed_grid_view;
pub mod typed_list_item;
pub mod typed_list_view;

use std::cell::{Ref, RefMut};

use relm4::gtk::glib;
use relm4::gtk::prelude::Cast;
pub use typed_grid_view::*;
pub use typed_list_item::*;
pub use typed_list_view::*;

fn get_value<T: 'static>(obj: &glib::Object) -> Ref<'_, T> {
    let wrapper = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
    wrapper.borrow()
}

fn get_mut_value<T: 'static>(obj: &glib::Object) -> RefMut<'_, T> {
    let wrapper = obj.downcast_ref::<glib::BoxedAnyObject>().unwrap();
    wrapper.borrow_mut()
}
