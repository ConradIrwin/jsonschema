//! This module provides a mechanism for creating and managing composite keys
//! used in anchor lookups. It allows for efficient lookups without the need
//! to construct data structures with owned values.
//!
//! The key components are:
//! - `AnchorKey`: An owned version of the composite key.
//! - `AnchorKeyRef`: A borrowed version of the composite key.
//! - `BorrowDyn`: A trait that allows for dynamic borrowing of key components.
//!
//! This design enables the use of borrowed data in hash map lookups while
//! still storing owned data.
use std::{
    borrow::Borrow,
    hash::{Hash, Hasher},
};

use fluent_uri::Uri;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub(crate) struct AnchorKey {
    uri: Uri<String>,
    name: String,
}

impl AnchorKey {
    pub(crate) fn new(uri: Uri<String>, name: String) -> Self {
        Self { uri, name }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub(crate) struct AnchorKeyRef<'a> {
    uri: &'a Uri<String>,
    name: &'a str,
}

impl<'a> AnchorKeyRef<'a> {
    pub(crate) fn new(uri: &'a Uri<String>, name: &'a str) -> Self {
        AnchorKeyRef { uri, name }
    }

    pub(crate) fn borrow_dyn(&self) -> &dyn BorrowDyn {
        self as &dyn BorrowDyn
    }
}

pub(crate) trait BorrowDyn {
    fn borrowed_key(&self) -> AnchorKeyRef;
}

impl BorrowDyn for AnchorKey {
    fn borrowed_key(&self) -> AnchorKeyRef {
        AnchorKeyRef::new(&self.uri, &self.name)
    }
}

impl BorrowDyn for AnchorKeyRef<'_> {
    fn borrowed_key(&self) -> AnchorKeyRef<'_> {
        *self
    }
}

impl<'a> Borrow<dyn BorrowDyn + 'a> for AnchorKey {
    fn borrow(&self) -> &(dyn BorrowDyn + 'a) {
        self
    }
}

impl Eq for (dyn BorrowDyn + '_) {}

impl PartialEq for (dyn BorrowDyn + '_) {
    fn eq(&self, other: &dyn BorrowDyn) -> bool {
        self.borrowed_key().eq(&other.borrowed_key())
    }
}

impl<'a> Hash for (dyn BorrowDyn + 'a) {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.borrowed_key().hash(state);
    }
}
