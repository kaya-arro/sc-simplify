// Benchmark replacing usize with u8
use std::cmp::min;

use itertools::Itertools;

// use crate::Rc;
use crate::Reverse;
use crate::helpers::{SCHashMap, SCHashSet, new_hs, new_vec};
use crate::max;
use crate::{Face, SimplicialComplex};
use std::rc::Rc;

#[derive(Clone, PartialEq, Eq, Hash)]
struct PartialBijection(Vec<(u8, u8)>);

impl PartialBijection {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn reduce(&self, i: usize) -> Self {
        let mut reduced = self.clone();
        reduced.0.remove(i);

        reduced
    }
}

struct FacetGenerator(Vec<(u8, usize)>);

impl FacetGenerator {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn vals(&self) -> Vec<u8> {
        self.0.iter().map(|(b, _)| *b).collect()
    }

    fn perm(&self) -> Vec<usize> {
        let mut perm: Vec<usize> = self.0.iter().map(|(_, i)| *i).collect();

        perm.truncate(perm.len() - 1);
        for i in 0..perm.len() - 1 {
            for j in i + 1..perm.len() {
                if perm[i] < perm[j] {
                    perm[j] -= 1;
                }
            }
        }

        perm
    }

    fn generate(&self, n: u8, check_len: bool) -> Vec<PartialBijection> {
        let m = self.len();
        let mut facet_vec = new_vec::<PartialBijection>(m);
        let mut vert = PartialBijection((0..m as u8).zip(self.vals()).collect());

        facet_vec.push(vert.clone());
        for i in self.perm() {
            vert = vert.reduce(i);
            if !(check_len && m as u8 == n && vert.len() == m - 1) {
                facet_vec.push(vert.clone());
            }
        }

        facet_vec
    }
}

impl From<(Vec<u8>, Vec<usize>)> for FacetGenerator {
    fn from(v: (Vec<u8>, Vec<usize>)) -> Self {
        Self(v.0.iter().copied().zip(v.1.iter().copied()).collect())
    }
}

pub fn partial_bijection_complex(a: u8, b: u8) -> SimplicialComplex<u32> {
    let (m, n) = (min(a, b), max(a, b));

    let facet_gens = (0..n)
        .permutations(m.into())
        .cartesian_product((0..m as usize).permutations(m.into()))
        .map(|g| g.into());

    let mut verts_vec: Vec<PartialBijection> = facet_gens
        .clone()
        .fold(
            new_hs::<PartialBijection>(n.into()),
            |mut set, fg: FacetGenerator| {
                set.extend(fg.generate(n, false));

                set
            },
        )
        .into_iter()
        .collect();
    verts_vec.sort_by_key(|bj| Reverse(bj.len()));

    let mut sorted_vec = new_vec::<Rc<PartialBijection>>(verts_vec.len());
    let mut sorted_set = new_hs::<Rc<PartialBijection>>(verts_vec.len());
    let mut i = 0;

    // Inefficient but who cares
    while !verts_vec.is_empty() {
        let vert = &verts_vec[i];
        let sub_verts_set: SCHashSet<Rc<PartialBijection>> = if vert.len() > 1 {
            (0..vert.len()).map(|j| Rc::new(vert.reduce(j))).collect()
        } else {
            new_hs(0)
        };
        if sub_verts_set.is_subset(&sorted_set) {
            let vert = Rc::new(verts_vec.remove(i));
            if !(m == n && vert.len() == (m - 1).into()) {
                sorted_vec.push(vert.clone());
            }
            sorted_set.insert(vert);
            i = 0;
        } else {
            i += 1;
        }
    }
    drop(sorted_set);

    let vert_dict: SCHashMap<Rc<PartialBijection>, u32> =
        sorted_vec.into_iter().zip(0u32..).collect();
    let facets = facet_gens
        .map(|fg| Face::<u32>::from_iter(fg.generate(n, true).into_iter().map(|v| vert_dict[&v])));

    if m == n {
        SimplicialComplex::from(
            facets
                .collect::<SCHashSet<Face<u32>>>()
                .into_iter()
                .collect::<Vec<Face<u32>>>(),
        )
    } else {
        SimplicialComplex::from(facets.collect::<Vec<Face<u32>>>())
    }
}
