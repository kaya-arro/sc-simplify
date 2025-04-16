use std::cmp::Ordering;
use std::fmt;

use crate::{BitAnd, Default, Display};
use crate::{HashSet, new_hs, new_vec, to_vec, to_sorted_vec};
// use crate::to_rev_sorted_vec;
// use crate::BTreeSet;
use std::ops::{BitOr, Sub};
use std::hash::{Hash, Hasher};


#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Simplex(pub HashSet<u32>);

// Considering making tuple an attribute so this doesn't have to be recalculated all the time.
impl Hash for Simplex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tuple().hash(state);
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


impl Display for Simplex {
    fn fmt (&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut vec = to_vec(&self.0);
        vec.sort_unstable();
        let mut vec_str = new_vec::<String>(vec.len());
        vec_str.extend(vec.into_iter().map(|v| format!["{}", v]));
        write![f, "[{}]", vec_str.join(" ")]
    }
}


impl BitAnd for &Simplex {
    type Output = Simplex;

    fn bitand(self, rhs: &Simplex) -> Simplex {
        // if self.is_disjoint(&rhs) { return Simplex::default(); }
        Simplex(&self.0 & &rhs.0)
    }
}


impl BitOr for &Simplex {
    type Output = Simplex;

    fn bitor(self, rhs: &Simplex) -> Simplex {
        Simplex(&self.0 | &rhs.0)
    }
}


impl Sub for &Simplex {
    type Output = Simplex;

    fn sub(self, rhs: &Simplex) -> Simplex {
        Simplex(&self.0 - &rhs.0)
    }
}


impl<T: IntoIterator<Item = u32>> From<T> for Simplex {
    fn from(col: T) -> Self {
        Self(col.into_iter().collect())
    }
}


impl Simplex {

    pub fn contains(&self, item: &u32) -> bool {
        self.0.contains(item)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_disjoint(&self, other: &Simplex) -> bool {
        self.0.is_disjoint(&other.0)
    }

    pub fn insert(&mut self, item: &u32) -> bool {
        self.0.insert(*item)
    }

    pub fn add_vertex(&self, item: &u32) -> Self {
        let mut new_set = new_hs(self.len() + 1);
        new_set.extend(&self.0);
        new_set.insert(*item);

        Self(new_set)
    }

    pub fn remove(&mut self, item: &u32) -> bool {
        self.0.remove(item)
    }

    pub fn intersection(&self, rhs: &Self) -> Self {
        Self(&self.0 & &rhs.0)
    }

    pub fn tuple(&self) -> Vec<u32> {
        to_sorted_vec(&self.0)
    }

    pub fn faces(&self) -> Vec<Self> {
        self.0.iter().map(|v| Simplex(
            self.0.clone().into_iter().filter(|u| u != v).collect()
        )).collect()
    }

    pub fn sgn(&self, face: &Self) -> i32 {
        // Maybe make tuple a method
        let tuple = self.tuple();
        for i in 0..tuple.len() {
            if !face.0.contains(&tuple[i]) { return (-1i32).pow(i as u32); }
        }

        0
    }
}


// We make a new struct so that we can use a different Ord
#[derive(Clone, PartialEq, Eq)]
pub struct PrettySimplex(Vec<u32>);


impl PrettySimplex {
    // We implement an ad hoc method instead of Display so that we can call the parameter d
    pub fn print(&self, d: usize) {
        let mut vec_str = new_vec::<String>(self.0.len());
        vec_str.extend(self.0.iter().map(|v| format!["{:>d$}", v]));
        println!("{}", vec_str.join(" "))
    }
}


impl From<&Simplex> for PrettySimplex {
    fn from (simp: &Simplex) -> Self {
        let mut verts: Vec<u32> = simp.0.iter().map(|v| *v).collect();
        verts.sort_unstable_by(|a, b| b.cmp(a));
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
