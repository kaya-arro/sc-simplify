use std::cmp::Ordering;

use crate::{BitAnd, Default, BTreeSet, HashSet, new_v, the_hasher};
use std::ops::{BitOr, Sub};
use std::hash::{Hash, Hasher};


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Simplex(pub HashSet<u32>);

impl Default for Simplex {
    #[inline]
    fn default() -> Self {
        Self(HashSet::with_hasher(the_hasher()))
    }
}

impl Hash for Simplex {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.iter().collect::<BTreeSet<&u32>>().hash(state);
    }
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

    #[inline]
    fn bitand(self, rhs: &Simplex) -> Simplex {
        Simplex(&self.0 & &rhs.0)
    }
}

impl BitOr for &Simplex {
    type Output = Simplex;

    #[inline]
    fn bitor(self, rhs: &Simplex) -> Simplex {
        Simplex(&self.0 | &rhs.0)
    }
}

impl Sub for &Simplex {
    type Output = Simplex;

    #[inline]
    fn sub(self, rhs: &Simplex) -> Simplex {
        Simplex(&self.0 - &rhs.0)
    }
}

impl<T: IntoIterator<Item = u32>> From<T> for Simplex {
    #[inline]
    fn from(col: T) -> Self {
        Self(col.into_iter().collect())
    }
}

impl Simplex {

    #[inline]
    pub fn contains(&self, item: &u32) -> bool {
        self.0.contains(item)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[inline]
    pub fn is_disjoint(&self, other: &Simplex) -> bool {
        self.0.is_disjoint(&other.0)
    }

    #[inline]
    pub fn insert(&mut self, item: &u32) -> bool {
        self.0.insert(*item)
    }

    #[inline]
    pub fn remove(&mut self, item: &u32) -> bool {
        self.0.remove(item)
    }

    #[inline]
    pub fn intersection(&self, rhs: &Self) -> Self {
        Self(&self.0 & &rhs.0)
    }
}


#[derive(Clone, PartialEq, Eq)]
pub struct PrettySimplex(Vec<u32>);

impl PrettySimplex {

    pub fn print(&self, d: usize) {
        let mut vec_str = new_v::<String>(self.0.len());
        vec_str.extend(self.0.iter().map(|v| format!("{:>d$}", v)));
        println!("{}", vec_str.join(" "))
    }

}

impl From<&Simplex> for PrettySimplex {
    fn from (simp: &Simplex) -> Self {
        let mut verts: Vec<u32> = simp.0.iter().map(|v| *v).collect();
        verts.sort_by(|a, b| b.cmp(a));
        Self(verts)
    }
}

impl PartialOrd for PrettySimplex {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        if self.0.len() > rhs.0.len() {
            return Some(Ordering::Less);
        }
        if self.0.len() < rhs.0.len() {
            return Some(Ordering::Greater);
        }
        if self.0 > rhs.0 {
            return Some(Ordering::Less);
        }
        if self.0 < rhs.0 {
            return Some(Ordering::Greater);
        }

        Some(Ordering::Equal)
    }
}

impl Ord for PrettySimplex {
    fn cmp(&self, rhs: &Self) -> Ordering {
        if self.0.len() > rhs.0.len() {
            return Ordering::Less;
        }
        if self.0.len() < rhs.0.len() {
            return Ordering::Greater;
        }
        if self.0 > rhs.0 {
            return Ordering::Less;
        }
        if self.0 < rhs.0 {
            return Ordering::Greater;
        }

        Ordering::Equal
    }
}
