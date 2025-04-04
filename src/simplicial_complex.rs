use rustc_hash::FxHashMap as HashMap;
use std::collections::BTreeSet;

use crate::{BitAnd, Default, HashSet, c, simplex::{Simplex, Edge}, the_hasher};

fn len_sort(vec: &mut Vec<Simplex>) {
    vec.sort_by(|a, b| b.len().cmp(&a.len()));
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct SimplicialComplex {
    pub facets: Vec<Simplex>,
}

impl From<&Simplex> for SimplicialComplex {
    fn from(simplex: &Simplex) -> Self {
        Self {
            facets: Vec::from([simplex.clone()]),
        }
    }
}

impl BitAnd for &SimplicialComplex {
    type Output = SimplicialComplex;

    fn bitand(self, rhs: Self) -> SimplicialComplex {
        self.intersection_with_complex(rhs)
    }
}

impl SimplicialComplex {
    pub fn from_check(mut facets: Vec<Simplex>) -> Self {
        match facets.len() {
            0 => Self::default(),
            _ => {
                len_sort(&mut facets);
                let old_facets = facets.clone();
                facets.clear();
                for old_facet in old_facets {
                    if !facets.iter().any(|new_facet| old_facet <= *new_facet) {
                        facets.push(old_facet);
                    }
                }

                Self { facets }
            }
        }
    }

    pub fn first_len(&self) -> usize {
        return self.facets[0].len();
    }

    pub fn vertex_set(&self) -> HashSet<u32> {
        let mut vertex_set: HashSet<u32> =
            HashSet::with_capacity_and_hasher(self.facets.len() * self.first_len(), the_hasher());
        for facet in &self.facets {
            vertex_set.extend(&facet.0);
        }

        vertex_set
    }

    fn intersection_with_simplex(&self, other: &Simplex) -> Self {
        let int_faces = c![f & other, for f in &self.facets];

        Self::from_check(int_faces)
    }

    fn intersection_with_complex(&self, other: &Self) -> Self {
        let mut int_faces: HashSet<BTreeSet<u32>> =
            HashSet::with_capacity_and_hasher(self.facets.len() * other.facets.len(), the_hasher());
        for facet in &self.facets {
            for other_facet in &other.facets {
                int_faces.insert((facet & other_facet).0.into_iter().collect());
            }
        }

        Self::from_check(int_faces.into_iter().map(|f| Simplex::from(f)).collect())
    }

    fn nerve(&self) -> Self {
        let vertex_set = self.vertex_set();
        let facet_count = self.facets.len();

        let nerve_faces = c![Simplex::from(
            c![i as u32, for i in 0..facet_count, if self.facets[i].contains(&v)]
        ), for v in vertex_set];

        Self::from_check(nerve_faces)
    }

    pub fn reduce(&mut self) {
        let mut return_base = true;
        let mut base_vertex_count = self.vertex_set().len();
        if base_vertex_count == 0 {
            return;
        }
        let mut nerve = self.nerve();
        while (return_base
            && (nerve.first_len() < self.first_len() || nerve.facets.len() < base_vertex_count))
            || (!return_base
                && (nerve.first_len() > self.first_len() || nerve.facets.len() > base_vertex_count))
        {
            if return_base {
                *self = nerve.nerve();
                base_vertex_count = self.vertex_set().len();
            } else {
                nerve = self.nerve();
            }
            return_base = !return_base;
        }
        if !return_base {
            *self = nerve;
        }
    }

    fn is_contractible(&self, thorough: bool) -> bool {
        if self.first_len() == 0 {
            return false;
        }
        let mut sc = self;
        let mut facet_count = sc.facets.len();
        let mut first_len: usize;
        let mut nerve: Self;
        let mut nerve_facet_count: usize;
        loop {
            if facet_count == 1 {
                return true;
            }
            first_len = sc.first_len();
            if first_len == 1 {
                return false;
            }
            let vertex_count = sc.vertex_set().len();
            if facet_count == 2 {
                return vertex_count != first_len + sc.facets[1].0.len();
            }
            nerve = sc.nerve();
            nerve_facet_count = nerve.facets.len();
            if vertex_count == nerve_facet_count {
                if !thorough || vertex_count < 5 || facet_count < 5 || first_len < 3 {
                    return false;
                }
                // At this point, we are not using the heuristic. We check whether to perform the
                // thorough check on the current complex or its nerve based on which has lesser
                // dimension.
                if first_len > nerve.first_len() {
                    // The nerve has lesser dimension: perform the check on it.
                    return nerve.first_facet_to_complex().is_deformation_retract(&nerve, true);
                }
                // The nerve has greater dimension: revert to the previous complex by taking the
                // nerve again (necessary because we have already moved `sc` into `nerve`.
                let nsc = nerve.nerve();
                return nsc.first_facet_to_complex().is_deformation_retract(&nsc, true);
            }
            sc = &nerve;
            facet_count = nerve_facet_count;
        }
    }

    fn enlarge_in_supercomplex<'a>(&mut self, supercomplex: &'a Self, thorough: bool) -> Vec<&'a Simplex> {
        let mut remainder = c![
            &facet, for facet in &supercomplex.facets, if !self.facets.contains(&facet)
        ];
        let mut remove_these: Vec<&Simplex>;
        let mut done = false;
        while !done {
            done = true;
            remove_these = Vec::with_capacity(remainder.len());
            for f in &remainder {
                if self.intersection_with_simplex(f).is_contractible(thorough) {
                    self.facets.push((*f).clone());
                    remove_these.push(f);
                    done = false;
                }
            }
            remainder = c![
                &facet, for facet in remainder, if !remove_these.contains(&facet)
            ];
        }

        *self = Self::from_check(self.facets.clone());
        remainder
    }

    fn is_deformation_retract(&mut self, supercomplex: &Self, thorough: bool) -> bool {
        return self.enlarge_in_supercomplex(supercomplex, thorough).is_empty();
    }

    // It may seem odd that we always calculate exactly three links at a time. The reason is
    // because it is more efficient to calculate links simultaneously than one-by-one: we avoid
    // looping through the facets unnecessarily. We could calculate an arbitrary number of links at
    // a time, but that would require using Vecs. By calculating three at a time, we can use arrays
    // and keep more of our data on the stack. We only ever wind up needing to calculate exactly
    // three links at a time, so this works out fine despite looking weird.
    fn three_links(&self, faces: [Simplex; 3]) -> [Self; 3] {
        let facets_len = self.facets.len();
        let mut links = [
            Self { facets: Vec::with_capacity(facets_len) },
            Self { facets: Vec::with_capacity(facets_len) },
            Self { facets: Vec::with_capacity(facets_len) },
        ];
        for facet in &self.facets {
            for n in 0..3 {
                let face = &faces[n];
                if face <= facet {
                    links[n].facets.push(facet - face);
                }
            }
        }

        links
    }

    fn edges(&self) -> Vec<Edge> {
        let first_len = self.first_len();
        if first_len == 0 {
            return Vec::new();
        }
        let upper_bound = self.facets.len() * first_len * (first_len - 1);
        let mut edge_set: HashSet<Edge> =
        HashSet::with_capacity_and_hasher(upper_bound, the_hasher());

        for facet in &self.facets {
            let mut tuple: Vec<u32> = c![*v, for v in &facet.0];
            let len = tuple.len();
            tuple.sort_unstable();
            for i in 0..len - 1 {
                for j in i + 1..len {
                    edge_set.insert(Edge(tuple[j], tuple[i]));
                }
            }
        }

        let mut edge_vec = c![edge, for edge in edge_set];
        edge_vec.sort_unstable();
        return edge_vec;
    }

    pub fn pinch(&mut self, thorough: bool) {
        let mut moved = HashSet::<u32>::with_capacity_and_hasher(
            self.facets.len() * self.first_len(),
            the_hasher(),
        );
        for edge in self.edges() {
            let old = edge.0;
            let new = edge.1;

            if moved.contains(&old) || moved.contains(&new) {
                continue;
            }

            let o_s = Simplex::from([old]);
            let n_s = Simplex::from([new]);
            let e_s = Simplex::from([old, new]);

            let [ref o_link, ref n_link, ref mut e_link] = self.three_links([o_s, n_s, e_s]);
            let intersection = o_link & n_link;

            if e_link.is_deformation_retract(&intersection, thorough) {
                moved.insert(old);

                for facet in &mut self.facets {
                    if facet.remove(&old) {
                        facet.insert(&new);
                    }
                }
            }
        }

        *self = Self::from_check(self.facets.clone());
    }

    fn first_facet_to_complex(&self) -> Self {
        Self {
            facets: Vec::from([self.facets[0].clone()]),
        }
    }

    pub fn contractible_subcomplex(&self, thorough: bool) -> Self {
        let mut contractible = self.first_facet_to_complex();
        contractible.enlarge_in_supercomplex(self, thorough);

        contractible
    }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vertex_dict: HashMap<u32, u32> =
        HashMap::with_capacity_and_hasher(vertex_set.len(), the_hasher());
        let mut n = 0u32;
        for v in vertex_set.into_iter().collect::<Vec<u32>>() {
            vertex_dict.insert(v, n);
            n += 1;
        }
        for facet in &mut self.facets {
            facet.0 = facet.0.iter().map(|v| vertex_dict[v]).collect();
        }
    }

}
