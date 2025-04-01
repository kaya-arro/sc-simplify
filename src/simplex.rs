use std::{fmt, cmp::Ordering};

use super::{BitAnd, Default, c, HashSet, the_hasher};
use std::ops::{BitOr, Sub};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simplex(pub HashSet<u32>);

impl fmt::Display for Simplex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", c![v.to_string(), for v in &self.0].join(" "))
    }
}

impl Default for Simplex {
    fn default() -> Self { Self(HashSet::with_hasher(the_hasher())) }
}

impl PartialOrd for Simplex {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.0 == rhs.0 {
            return Some(Ordering::Equal);
        } else if self.0.is_subset(&rhs.0) {
            return Some(Ordering::Less);
        } else if rhs.0.is_subset(&self.0) {
            return Some(Ordering::Greater);
        } else {
            return None;
        }
    }
}

impl BitAnd for &Simplex {
    type Output = Simplex;

    fn bitand(self, rhs: &Simplex) -> Simplex { Simplex(&self.0 & &rhs.0) }
}

impl BitOr for &Simplex {
    type Output = Simplex;

    fn bitor(self, rhs: &Simplex) -> Simplex { Simplex(&self.0 | &rhs.0) }
}

impl Sub for &Simplex {
    type Output = Simplex;

    fn sub(self, rhs: &Simplex) -> Simplex { Simplex(&self.0 - &rhs.0) }
}

impl<T: IntoIterator<Item=u32>> From<T> for Simplex {
    fn from(col: T) -> Self { Self(col.into_iter().collect()) }
}

impl Simplex {

    pub fn contains(&self, item: &u32) -> bool { self.0.contains(item) }

    pub fn len(&self) -> usize { self.0.len() }

    pub fn is_disjoint(&self, other: &Simplex) -> bool { self.0.is_disjoint(&other.0) }

    pub fn insert(&mut self, item: &u32) -> bool { self.0.insert(*item) }

    pub fn remove(&mut self, item: &u32) -> bool { self.0.remove(item) }

    pub fn intersection(&self, rhs: &Self) -> Self { Self(&self.0 & &rhs.0) }

}
