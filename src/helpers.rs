// use crate::{Rc, Hash};

pub(crate) use rustc_hash::{FxBuildHasher as SCBuildHasher, FxHashMap as SCHashMap};
pub(crate) use std::collections::VecDeque;

pub(crate) use rustc_hash::FxHashSet as SCHashSet;

// use hashbrown::HashSet;
// pub(crate) type SCHashSet<T> = HashSet<T, SCBuildHasher>;

pub(crate) fn new_hs<T>(len: usize) -> SCHashSet<T> {
    SCHashSet::with_capacity_and_hasher(len, SCBuildHasher::default())
}

pub(crate) fn new_hm<S, T>(len: usize) -> SCHashMap<S, T> {
    SCHashMap::with_capacity_and_hasher(len, SCBuildHasher::default())
}

pub(crate) fn new_vec<T>(len: usize) -> Vec<T> {
    Vec::<T>::with_capacity(len)
}

pub(crate) fn new_vd<T>(len: usize) -> VecDeque<T> {
    VecDeque::<T>::with_capacity(len)
}

pub(crate) fn to_sorted_vec<T: Ord + Copy>(set: &SCHashSet<T>) -> Vec<T> {
    let mut vec: Vec<T> = set.iter().copied().collect();
    vec.sort_unstable_by_key(|v| crate::Reverse(*v));

    vec
}

/*
pub(crate) fn update_dict<T: Eq + Hash>(dict: &mut SCHashMap<T, Point>, item: T, n: &mut Point) -> Point {
    if let Some(val) = dict.get(&item) {
        *val
    } else {
        dict.insert(item, *n);
        *n += 1;
        *n - 1
    }
}
*/

/*
#[derive(Default)]
struct UniQ<T: Hash + Eq> {
    pub(crate) queue: VecDeque<Rc<T>>,
    pub(crate) set: SCHashSet<Rc<T>>,
}

impl<T: Hash + Eq> UniQ<T> {
    pub(crate) fn contains(&self, val: &T) -> bool {
        self.set.contains(val)
    }

    pub(crate) fn push(&mut self, val: Rc<T>) -> bool {
        if self.set.insert(val.clone()) {
            self.queue.push_back(val.clone());
            true
        } else {
            false
        }
    }

    pub(crate) fn pop(&mut self) -> Option<Rc<T>> {
        match self.queue.pop_front() {
            Some(val) => {
                self.set.remove(&val);

                Some(val)
            }
            None => None,
        }
    }

    pub(crate) fn extend(&mut self, it: impl Iterator<Item = Rc<T>>) {
        for item in it {
            self.push(item);
        }
    }
}
*/
