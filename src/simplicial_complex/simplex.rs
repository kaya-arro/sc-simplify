use std::collections::hash_set::{IntoIter, Iter};
use std::hash::{Hash, Hasher};
use std::iter::{Copied, Extend};
use std::mem::swap;
use std::ops::{Sub, SubAssign};

use super::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

use crate::Vertex;
use crate::helpers::{SCHashMap, SCHashSet};
use crate::{Debug, Default, Ordering, fmt};

#[derive(Default, Clone, PartialEq, Eq)]
pub struct Face<Point: Vertex> {
    vertices: SCHashSet<Point>,
}

impl<Point: Vertex> Debug for Face<Point> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.vertices.fmt(f)
    }
}

impl<Point: Vertex> From<SCHashSet<Point>> for Face<Point> {
    fn from(vertices: SCHashSet<Point>) -> Self {
        Self { vertices }
    }
}

impl<Point: Vertex> From<Point> for Face<Point> {
    fn from(vertex: Point) -> Self {
        Self::from_iter(std::iter::once(vertex))
    }
}

impl<Point: Vertex> FromIterator<Point> for Face<Point> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Point>,
    {
        Self::from(iter.into_iter().collect::<SCHashSet<Point>>())
    }
}

impl<Point: Vertex> Extend<Point> for Face<Point> {
    fn extend<T: IntoIterator<Item = Point>>(&mut self, iter: T) {
        self.vertices.extend(iter);
    }
}

impl<Point: Vertex> Hash for Face<Point> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tuple().hash(state);
    }
}

impl<Point: Vertex> PartialOrd for Face<Point> {
    fn partial_cmp(&self, rhs: &Self) -> Option<Ordering> {
        match self.len().cmp(&rhs.len()) {
            Ordering::Equal if self.iter().all(|v| rhs.contains(v)) => Some(Ordering::Equal),
            Ordering::Less if self.iter().all(|v| rhs.contains(v)) => Some(Ordering::Less),
            Ordering::Greater if rhs.iter().all(|v| self.contains(v)) => Some(Ordering::Greater),
            _ => None,
        }
    }
}

impl<Point: Vertex> BitAnd for &Face<Point> {
    type Output = Face<Point>;

    fn bitand(self, rhs: Self) -> Face<Point> {
        self.intersection(rhs)
    }
}

impl<Point: Vertex> BitAndAssign for Face<Point> {
    fn bitand_assign(&mut self, mut rhs: Self) {
        if self.len() <= rhs.len() {
            self.vertices.retain(|v| rhs.contains(*v));
        } else {
            swap(self, &mut rhs);
            self.vertices.retain(|v| rhs.contains(*v));
        }
        self.shrink_to_fit();
    }
}

impl<Point: Vertex> BitOr for &Face<Point> {
    type Output = Face<Point>;

    fn bitor(self, rhs: Self) -> Face<Point> {
        Face::from(&self.vertices | &rhs.vertices)
    }
}

impl<Point: Vertex> BitOrAssign for Face<Point> {
    fn bitor_assign(&mut self, mut rhs: Self) {
        if self.len() >= rhs.len() {
            self.vertices.extend(rhs.into_iter());
        } else {
            swap(self, &mut rhs);
            self.extend(rhs.into_iter());
        }
    }
}

impl<Point: Vertex> BitOrAssign<&Face<Point>> for Face<Point> {
    fn bitor_assign(&mut self, rhs: &Face<Point>) {
        self.extend(rhs.into_iter());
    }
}

impl<Point: Vertex> Sub for &Face<Point> {
    type Output = Face<Point>;

    fn sub(self, rhs: &Face<Point>) -> Face<Point> {
        if self.len() <= rhs.len() {
            Face::from(&self.vertices - &rhs.vertices)
        } else {
            let mut res = self.clone();
            for v in rhs {
                res.remove(v);
            }
            res.shrink_to_fit();

            res
        }
    }
}

impl<Point: Vertex> SubAssign<&Face<Point>> for Face<Point> {
    fn sub_assign(&mut self, rhs: &Face<Point>) {
        if self.len() <= rhs.len() {
            self.vertices.retain(|v| !rhs.contains(*v));
        } else {
            rhs.into_iter().for_each(|v| {
                self.remove(v);
            });
        }
    }
}

impl<'a, Point: Vertex> IntoIterator for &'a Face<Point> {
    type Item = Point;
    type IntoIter = Copied<Iter<'a, Point>>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.iter().copied()
    }
}

impl<Point: Vertex> IntoIterator for Face<Point> {
    type Item = Point;
    type IntoIter = IntoIter<Point>;

    fn into_iter(self) -> Self::IntoIter {
        self.vertices.into_iter()
    }
}

impl<Point: Vertex> From<Face<Point>> for Vec<Point> {
    fn from(s: Face<Point>) -> Self {
        s.into_iter().collect()
    }
}

impl<Point: Vertex> From<&Face<Point>> for Vec<Point> {
    fn from(s: &Face<Point>) -> Self {
        s.into_iter().collect()
    }
}

impl<Point: Vertex> Face<Point> {
    pub fn shrink_to_fit(&mut self) {
        self.vertices.shrink_to_fit();
    }

    pub fn is_empty(&self) -> bool {
        self.vertices.is_empty()
    }

    pub fn contains(&self, item: Point) -> bool {
        self.vertices.contains(&item)
    }

    pub fn len(&self) -> usize {
        self.vertices.len()
    }

    pub fn iter(&self) -> Copied<Iter<Point>> {
        self.into_iter()
    }

    pub fn is_disjoint(&self, other: &Face<Point>) -> bool {
        self.vertices.is_disjoint(&other.vertices)
    }

    pub fn leq(&self, other: &Self) -> bool {
        self.vertices.is_subset(&other.vertices)
    }

    pub fn insert(&mut self, item: Point) -> bool {
        self.vertices.insert(item)
    }

    pub fn remove(&mut self, item: Point) -> bool {
        self.vertices.remove(&item)
    }

    pub fn intersection(&self, other: &Self) -> Self {
        Self::from(&self.vertices & &other.vertices)
    }

    // Returns None if the intersection is empty, Some(intersection) otherwise.
    pub fn maybe_intersection(&self, other: &Self) -> Option<Self> {
        let res = self.intersection(other);
        if res.is_empty() { None } else { Some(res) }
    }

    pub fn union(&self, other: &Self) -> Self {
        Self::from(self | other)
    }

    pub fn to_vec(&self) -> Vec<Point> {
        self.into()
    }

    pub fn tuple(&self) -> Vec<Point> {
        let mut tuple = self.to_vec();
        tuple.sort_unstable();

        tuple
    }

    pub fn replace_verts_from_map(&mut self, dict: &SCHashMap<Point, Point>) {
        self.vertices = self.vertices.iter().map(|v| dict[v]).collect();
    }

    pub fn vertex_removed(&self, v: Point) -> Self {
        let mut res = self.clone();
        res.remove(v);

        res
    }

    pub fn vertex_inserted(&self, v: Point) -> Self {
        let mut vertices = self.vertices.clone();
        vertices.insert(v);

        Self::from(vertices)
    }
}
