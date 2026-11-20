//! A list whose elements are converted into CEL's runtime form only when an expression iterates
//! them. Used for `inbound.<relation>` (docs/v1-spec.md §6.2): `size()` is O(1), and
//! `map`/`exists`/`all`/`filter` convert just the elements they visit.
//!
//! Supported operations: `size`, iteration macros. Indexing (`xs[0]`) and `in` are not
//! (they fail with a no-overload error, which the pipeline reports as a diagnostic).

use cel::Value;
use cel::common::traits::{Iterable, Iterator as CelIterator, Sizer};
use cel::common::types::{CelInt, CelMap, CelMapKey, CelString, LIST_TYPE, Type};
use cel::common::value::Val;
use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

#[derive(Debug)]
pub struct LazyList {
    items: Arc<Vec<Value>>,
    cache: Vec<OnceLock<Box<dyn Val>>>,
}

impl LazyList {
    pub fn new(items: Arc<Vec<Value>>) -> Self {
        let cache = (0..items.len()).map(|_| OnceLock::new()).collect();
        LazyList { items, cache }
    }

    fn at(&self, i: usize) -> &(dyn Val + 'static) {
        self.cache[i]
            .get_or_init(|| {
                self.items[i]
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

    fn clone_as_boxed<'v>(&self) -> Box<dyn Val + 'v>
    where
        Self: 'v,
    {
        Box::new(LazyList::new(self.items.clone()))
    }
}

/// A CEL map from names to lazy lists.
pub fn lazy_map(entries: impl IntoIterator<Item = (String, Arc<Vec<Value>>)>) -> Box<dyn Val> {
    let map: HashMap<CelMapKey<'static>, Box<dyn Val>> = entries
        .into_iter()
        .map(|(k, items)| {
            (
                CelMapKey::String(CelString::from(k)),
                Box::new(LazyList::new(items)) as Box<dyn Val>,
            )
        })
        .collect();
    Box::new(CelMap::from(map))
}
