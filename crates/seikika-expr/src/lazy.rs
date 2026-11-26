//! A list whose elements are converted into CEL's runtime form only when an expression iterates
//! them. Used for `inbound.<relation>` (docs/v1-spec.md §6.2): `size()` is O(1), and
//! `map`/`exists`/`all`/`filter` convert just the elements they visit.
//!
//! Supported operations: `size`, iteration macros and indexing by int (`xs[0]`, converting just
//! that element). `in` is not (it fails with a no-overload error, which the pipeline reports as a
//! diagnostic).

use crate::eval::{ExprValue, Lazy};
use cel::ExecutionError;
use cel::common::traits::{Indexer, Iterable, Iterator as CelIterator, Sizer};
use cel::common::types::{CelInt, CelMap, CelMapKey, CelString, LIST_TYPE, Type};
use cel::common::value::{CowVal, Val};
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub(crate) struct LazyList {
    items: Arc<Vec<ExprValue>>,
    cache: Vec<OnceLock<Box<dyn Val>>>,
}

impl LazyList {
    fn new(items: Arc<Vec<ExprValue>>) -> Self {
        let cache = (0..items.len()).map(|_| OnceLock::new()).collect();
        LazyList { items, cache }
    }

    fn at(&self, i: usize) -> &(dyn Val + 'static) {
        self.cache[i]
            .get_or_init(|| {
                self.items[i]
                    .0
                    .clone()
                    .try_into()
                    .expect("JSON-derived values convert")
            })
            .as_ref()
    }
}

impl Sizer for LazyList {
    fn size(&self) -> CelInt {
        CelInt::from(self.items.len() as i64)
    }
}

struct Iter<'b> {
    list: &'b LazyList,
    next: usize,
}

impl<'b, 'v> CelIterator<'b, 'v> for Iter<'b> {
    fn next(&mut self) -> Option<&'b (dyn Val + 'v)> {
        if self.next >= self.list.items.len() {
            return None;
        }
        self.next += 1;
        Some(self.list.at(self.next - 1))
    }
}

impl Iterable for LazyList {
    fn iter<'b, 'v>(&'b self) -> Box<dyn CelIterator<'b, 'v> + 'b>
    where
        Self: 'v,
    {
        Box::new(Iter {
            list: self,
            next: 0,
        })
    }
}

impl Indexer for LazyList {
    fn get<'b, 'v>(&'b self, idx: &dyn Val) -> Result<CowVal<'b, 'v>, ExecutionError>
    where
        Self: 'v,
    {
        let i = idx
            .downcast_ref::<CelInt>()
            .map(|i| *i.inner())
            // reported by the interpreter as no overload for `_[_]`
            .ok_or_else(|| ExecutionError::UnexpectedType {
                got: format!("{:?}", idx.get_type().kind()),
                want: "int".into(),
            })?;
        match usize::try_from(i).ok().filter(|&i| i < self.items.len()) {
            Some(i) => Ok(CowVal::Borrowed(self.at(i))),
            None => Err(ExecutionError::IndexOutOfBounds(i.into())),
        }
    }

    fn steal<'v>(self: Box<Self>, idx: &dyn Val) -> Result<Box<dyn Val + 'v>, ExecutionError>
    where
        Self: 'v,
    {
        self.get(idx).map(CowVal::into_owned)
    }
}

impl Val for LazyList {
    fn get_type(&self) -> &Type {
        &LIST_TYPE
    }

    fn cel_type() -> &'static Type {
        &LIST_TYPE
    }

    fn as_sizer(&self) -> Option<&dyn Sizer> {
        Some(self)
    }

    fn as_iterable<'b, 'v>(&'b self) -> Option<&'b (dyn Iterable + 'v)> {
        Some(self)
    }

    fn as_indexer<'b, 'v>(&'b self) -> Option<&'b (dyn Indexer + 'v)>
    where
        Self: 'v,
    {
        Some(self)
    }

    fn clone_as_boxed<'v>(&self) -> Box<dyn Val + 'v>
    where
        Self: 'v,
    {
        Box::new(LazyList::new(self.items.clone()))
    }
}

/// A map from names to lazy lists, bound with [`crate::with_vals`].
pub fn lazy_map(entries: impl IntoIterator<Item = (String, Arc<Vec<ExprValue>>)>) -> Lazy {
    let map: HashMap<CelMapKey<'static>, Box<dyn Val>> = entries
        .into_iter()
        .map(|(k, items)| {
            (
                CelMapKey::String(CelString::from(k)),
                Box::new(LazyList::new(items)) as Box<dyn Val>,
            )
        })
        .collect();
    Lazy(Box::new(CelMap::from(map)))
}
