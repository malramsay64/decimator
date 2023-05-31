use std::any::Any;
use std::cell::{Ref, RefMut};
use std::marker::PhantomData;

use gtk::glib;
use gtk::prelude::IsA;
use relm4::gtk;

/// An item of a [`TypedGridView`].
pub trait RelmListItem: Any {
    /// The top-level widget for the list item.
    type Root: IsA<gtk::Widget>;

    /// The widgets created for the list item.
    type Widgets;

    /// Construct the widgets.
    fn setup(list_item: &gtk::ListItem) -> (Self::Root, Self::Widgets);

    /// Bind the widgets to match the data of the list item.
    fn bind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}

    /// Undo the steps of [`RelmListItem::bind()`] if necessary.
    fn unbind(&mut self, _widgets: &mut Self::Widgets, _root: &mut Self::Root) {}

    /// Undo the steps of [`RelmListItem::setup()`] if necessary.
    fn teardown(_list_item: &gtk::ListItem) {}
}

/// And item of a [`TypedListView`].
///
/// The interface is very similar to [`std::cell::RefCell`].
/// Ownership is calculated at runtime, so you have to borrow the
/// value explicitly which might panic if done incorrectly.
#[derive(Debug, Clone)]
pub struct TypedListItem<T> {
    inner: glib::BoxedAnyObject,
    _ty: PhantomData<*const T>,
}

impl<T: 'static> TypedListItem<T> {
    pub(crate) fn new(inner: glib::BoxedAnyObject) -> Self {
        Self {
            inner,
            _ty: PhantomData,
        }
    }

    /*
    // rustdoc-stripper-ignore-next
    /// Immutably borrows the wrapped value, returning an error if the value is currently mutably
    /// borrowed or if it's not of type `T`.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple immutable borrows can be
    /// taken out at the same time.
    ///
    /// This is the non-panicking variant of [`borrow`](#method.borrow).
    pub fn try_borrow(&self) -> Result<Ref<'_, T>, BorrowError> {
        self.inner.try_borrow()
    }

    // rustdoc-stripper-ignore-next
    /// Mutably borrows the wrapped value, returning an error if the value is currently borrowed.
    /// or if it's not of type `T`.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// This is the non-panicking variant of [`borrow_mut`](#method.borrow_mut).
    pub fn try_borrow_mut(&mut self) -> Result<RefMut<'_, T>, BorrowMutError> {
        self.inner.try_borrow_mut()
    } */

    // rustdoc-stripper-ignore-next
    /// Immutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `Ref` exits scope. Multiple
    /// immutable borrows can be taken out at the same time.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently mutably borrowed.
    ///
    /// For a non-panicking variant, use
    /// [`try_borrow`](#method.try_borrow).
    #[must_use]
    pub fn borrow(&self) -> Ref<'_, T> {
        self.inner.borrow()
    }

    // rustdoc-stripper-ignore-next
    /// Mutably borrows the wrapped value.
    ///
    /// The borrow lasts until the returned `RefMut` or all `RefMut`s derived
    /// from it exit scope. The value cannot be borrowed while this borrow is
    /// active.
    ///
    /// # Panics
    ///
    /// Panics if the value is currently borrowed.
    ///
    /// For a non-panicking variant, use
    /// [`try_borrow_mut`](#method.try_borrow_mut).
    #[must_use]
    pub fn borrow_mut(&self) -> RefMut<'_, T> {
        self.inner.borrow_mut()
    }
}
