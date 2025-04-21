// use std::iter::once;

use crate::{BitAnd, Default, Ordering};
use crate::{HashSet, to_sorted_vec};
// use crate::BTreeSet;
use std::hash::{Hash, Hasher};
use std::ops::Sub;

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Simplex {
    pub vertices: HashSet<u32>,
}

impl Hash for Simplex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        to_sorted_vec(&self.vertices).hash(state);
    }
}

impl PartialOrd for Simplex {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.vertices == rhs.vertices {
            return Some(Ordering::Equal);
        } else if self.vertices.is_subset(&rhs.vertices) {
            return Some(Ordering::Less);
        } else if rhs.vertices.is_subset(&self.vertices) {
            return Some(Ordering::Greater);
        } else {
            return None;
        }
    }
}

impl BitAnd for &Simplex {
    type Output = Simplex;

    fn bitand(self, rhs: &Simplex) -> Simplex {
        Simplex::from(&self.vertices & &rhs.vertices)
    }
}

impl Sub for &Simplex {
    type Output = Simplex;

    fn sub(self, rhs: &Simplex) -> Simplex {
        Simplex::from(&self.vertices - &rhs.vertices)
    }
}

impl From<HashSet<u32>> for Simplex {
    fn from(vertices: HashSet<u32>) -> Self {
        Self { vertices }
    }
}

impl FromIterator<u32> for Simplex {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = u32>,
    {
        Self {
            vertices: iter.into_iter().collect(),
        }
    }
}

impl Simplex {
    pub fn contains(&self, item: &u32) -> bool {
        self.vertices.contains(item)
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn is_disjoint(&self, other: &Simplex) -> bool {
        self.vertices.is_disjoint(&other.vertices)
    }

    pub fn insert(&mut self, item: &u32) -> bool {
        self.vertices.insert(*item)
    }

    // fn add_vertex(&self, item: &u32) -> Self {
    //     Self::from_iter(self.vertices.iter().copied().chain(once(*item)))
    // }

    pub fn remove(&mut self, item: &u32) -> bool {
        self.vertices.remove(item)
    }

    pub fn intersection(&self, rhs: &Self) -> Self {
        Self::from(&self.vertices & &rhs.vertices)
    }

    pub fn tuple(&self) -> Vec<u32> {
        to_sorted_vec(&self.vertices)
    }
}
