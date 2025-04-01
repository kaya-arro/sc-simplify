use std::collections::BTreeSet;
use rustc_hash::FxHashMap as HashMap;

use super::{BitAnd, Default, c, HashSet, the_hasher, simplex::Simplex};

fn len_sort(vec: &mut Vec<Simplex>) {
    vec.sort_by(|a, b| b.len().cmp(&a.len()));
}

#[derive(Default, Clone)]
pub struct SimplicialComplex {
    pub facets: Vec<Simplex>
}

impl From<&Simplex> for SimplicialComplex {
    fn from(simplex: &Simplex) -> Self { Self{ facets: Vec::from([simplex.clone()]) } }
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

    fn first_len(&self) -> usize { return self.facets[0].len() }

    fn vertex_set(&self) -> HashSet<u32> {
        let mut vertex_set: HashSet<u32> = HashSet::with_capacity_and_hasher(
            self.facets.len() * self.first_len(), the_hasher()
        );
        for facet in &self.facets {
            vertex_set.extend(&facet.0);
        }

        vertex_set
    }

    fn edges(&self) -> HashSet<BTreeSet<u32>> {
        let first_len = self.first_len();
        if first_len == 0 {
            return HashSet::with_hasher(the_hasher());
        }
        let upper_bound = self.facets.len() * first_len * (first_len - 1);
        let mut edge_set: HashSet<BTreeSet<u32>> = HashSet::with_capacity_and_hasher(
            upper_bound, the_hasher()
        );

        for facet in &self.facets {
            let tuple = c![*v, for v in &facet.0];
            let len = tuple.len();
            for i in 0..len - 1 {
                for j in i + 1..len {
                    edge_set.insert(BTreeSet::from([tuple[i], tuple[j]]));
                }
            }
        }

        edge_set
    }

    fn intersection_with_simplex(&self, other: &Simplex) -> Self {
        let int_faces = c![f & other, for f in &self.facets];

        Self::from_check(int_faces)
    }

    fn intersection_with_complex(&self, other: &Self) -> Self {
        let mut int_faces: HashSet<BTreeSet<u32>> = HashSet::with_capacity_and_hasher(
            self.facets.len() * other.facets.len(), the_hasher()
        );
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
        while (return_base && (
            (nerve.first_len() < self.first_len() || nerve.facets.len() < base_vertex_count)
        )) || (!return_base && (
            (nerve.first_len() > self.first_len() || nerve.facets.len() > base_vertex_count)
        )) {
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

    fn is_contractible(&self) -> bool {
        if self.first_len() == 0 {
            return false;
        }
        let mut sc = self;
        let mut facet_count = sc.facets.len();
        let mut nerve: Self;
        loop {
            if facet_count == 1 {
                return true;
            }
            let first_len = sc.first_len();
            if first_len == 1 {
                return false;
            }
            let vertex_count = sc.vertex_set().len();
            if facet_count == 2 {
                return vertex_count != first_len + sc.facets[1].0.len();
            }
            nerve = sc.nerve();
            facet_count = nerve.facets.len();
            // This heuristic always works when there are four or fewer vertices but can give false
            // negatives for certain larger simplicial complexes. We use it anyway for efficiency.
            if vertex_count == facet_count {
                return false;
            }
            sc = &nerve;
        }
    }

    fn enlarge_in_supercomplex<'a>(&mut self, supercomplex: &'a Self) -> Vec<&'a Simplex> {
        let mut remainder = c![
            &facet, for facet in &supercomplex.facets, if !self.facets.contains(&facet)
        ];
        let mut remove_these: Vec<&Simplex>;
        let mut done = false;
        while !done {
            done = true;
            remove_these = Vec::with_capacity(remainder.len());
            for f in &remainder {
                if self.intersection_with_simplex(f).is_contractible() {
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

    fn is_deformation_retract(&mut self, supercomplex: &Self) -> bool {
        return self.enlarge_in_supercomplex(supercomplex).is_empty();
    }

    fn links(&self, faces: &Vec<Simplex>) -> Vec<Self> {
        let facets_len = self.facets.len();
        let mut facets_vec_vec = c![
            Vec::with_capacity(facets_len), for _f in faces
        ];
        let faces_len = faces.len();
        for facet in &self.facets {
            for n in 0..faces_len {
                let face = &faces[n];
                if face <= facet {
                    facets_vec_vec[n].push(facet - face);
                }
            }
        }

        c![Self { facets }, for facets in facets_vec_vec]
    }

    pub fn pinch(&mut self) -> bool {

        let mut moved = HashSet::<u32>::with_capacity_and_hasher(
            self.facets.len() * self.first_len(), the_hasher()
        );
        let mut pinched = false;

        for edge in c![edge.into_iter().collect::<Vec<u32>>(), for edge in self.edges()] {
            let new: u32 = edge[1];
            let old: u32 = edge[0];

            if moved.contains(&old) || moved.contains(&new) {
                continue;
            }

            let o_simp = Simplex::from(Vec::from([old]));
            let n_simp = Simplex::from(Vec::from([new]));
            let e_simp = Simplex::from(edge);

            if let [ref o_link, ref n_link, ref mut e_link] = self.links(
                &Vec::from([o_simp, n_simp, e_simp])
            )[..] {

                let intersection = o_link & n_link;

                if e_link.is_deformation_retract(&intersection) {
                    pinched = true;
                    moved.insert(old);

                    for facet in &mut self.facets {
                        if facet.remove(&old) {
                            facet.insert(&new);
                        }
                    }
                }
            }
        }

        if pinched {
            *self = Self::from_check(self.facets.clone());
        }

        pinched
    }

    fn first_facet_to_complex(&self) -> Self {
        Self { facets: Vec::from([self.facets[0].clone()]) }
    }

    pub fn contractible_subcomplex(&self) -> Self {
        let mut contractible = self.first_facet_to_complex();
        contractible.enlarge_in_supercomplex(self);

        contractible
    }

    pub fn minimal_pair(self) -> (Self, Self) {
        let mut contractible = self.first_facet_to_complex();
        let remainder_vec = contractible.enlarge_in_supercomplex(&self);
        let remainder = Self {
            facets: remainder_vec.into_iter().map(|s| (*s).clone()).collect()
        };
        let boundary = remainder.intersection_with_complex(&contractible);

        (remainder, boundary)
    }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vertex_dict: HashMap<u32, u32> = HashMap::with_capacity_and_hasher(
            vertex_set.len(), the_hasher()
        );
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
