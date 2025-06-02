use std::slice::{Iter, IterMut};
use std::sync::{Arc, atomic};

use rayon::prelude::*;

use super::BitAnd;

use crate::ProgressBar;
use crate::Vertex;
use crate::helpers::{SCHashMap, SCHashSet, new_hm, new_hs, new_vd, new_vec, to_sorted_vec};
use crate::io::{new_pb, new_spnr};
use crate::style::upd_sty;
use crate::{Debug, Default, Reverse, fmt, max};

use crate::Face;

#[derive(Clone, PartialEq, Eq)]
pub struct SimplicialComplex<Point: Vertex> {
    facets: Vec<Face<Point>>,
}

impl<Point: Vertex> Debug for SimplicialComplex<Point> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.facets.fmt(f)
    }
}

impl<Point: Vertex> Default for SimplicialComplex<Point> {
    fn default() -> Self {
        Self {
            facets: vec![Face::<Point>::default()],
        }
    }
}

impl<Point: Vertex> From<&Face<Point>> for SimplicialComplex<Point> {
    fn from(simplex: &Face<Point>) -> Self {
        Self {
            facets: vec![simplex.clone()],
        }
    }
}

impl<Point: Vertex> From<Vec<Face<Point>>> for SimplicialComplex<Point> {
    fn from(facets: Vec<Face<Point>>) -> Self {
        Self { facets }
    }
}

impl<Point: Vertex> FromIterator<Face<Point>> for SimplicialComplex<Point> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = Face<Point>>,
    {
        Self {
            facets: iter.into_iter().collect(),
        }
    }
}

impl<'a, Point: Vertex> IntoIterator for &'a SimplicialComplex<Point> {
    type Item = &'a Face<Point>;
    type IntoIter = Iter<'a, Face<Point>>;

    fn into_iter(self) -> Self::IntoIter {
        self.facets.iter()
    }
}

impl<Point: Vertex> IntoIterator for SimplicialComplex<Point> {
    type Item = Face<Point>;
    type IntoIter = <Vec<Self::Item> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.facets.into_iter()
    }
}

impl<'a, Point: Vertex> IntoIterator for &'a mut SimplicialComplex<Point> {
    type Item = &'a mut Face<Point>;
    type IntoIter = IterMut<'a, Face<Point>>;

    fn into_iter(self) -> Self::IntoIter {
        self.facets.iter_mut()
    }
}

impl<Point: Vertex> BitAnd for &SimplicialComplex<Point> {
    type Output = SimplicialComplex<Point>;

    fn bitand(self, rhs: Self) -> SimplicialComplex<Point> {
        self.intersection_with_complex(rhs, true)
    }
}

// Add from_check_sorted function
// Add sort_facets function
impl<Point: Vertex> SimplicialComplex<Point> {
    pub fn from_check_unique<T>(facets: T) -> Self
    where
        T: IntoIterator<Item = Face<Point>>,
    {
        let unique_facets = SCHashSet::<Face<Point>>::from_iter(facets);
        let mut sc = Self::from_iter(unique_facets);
        sc.maximalify();

        sc
    }

    pub fn from_check_maximal<T>(faces: T) -> Self
    where
        T: IntoIterator<Item = Face<Point>>,
    {
        let mut sc = Self::from_iter(faces);
        sc.maximalify();

        sc
    }

    pub fn from_check_sorted<T>(facets: T) -> Self
    where
        T: IntoIterator<Item = Face<Point>>,
    {
        let mut sc = Self::from_iter(facets);
        sc.sortify();

        sc
    }

    // Make privateable

    pub fn sortify(&mut self) {
        self.facets.sort_unstable_by_key(|s| Reverse(s.len()))
    }

    // Make privateable

    fn uniqueify(&mut self) {
        let unique_facets: SCHashSet<Face<Point>> = self.facets.drain(..).collect();
        self.facets.extend(unique_facets);
        self.shrink_to_fit();
    }

    // Make privateable
    pub fn maximalify(&mut self) {
        if self.facets.is_empty() {
            self.facets.push(Face::<Point>::default());
        } else {
            self.sortify();
            if self.facets[0].len() == self.facets.last().unwrap().len() {
                return;
            }

            let mut i = 1;
            while i < self.len() {
                let face: Vec<Point> = self.facets[i].iter().cloned().collect();
                let len = face.len();
                if self
                    .iter()
                    .take_while(|f| f.len() > len)
                    .any(|f| face.iter().all(|v| f.contains(v)))
                {
                    self.facets.remove(i);
                } else {
                    i += 1;
                }
            }

            self.shrink_to_fit();
        }
    }

    pub fn height(&self) -> usize {
        self.facets[0].len()
    }

    pub fn is_empty(&self) -> bool {
        self.height() == 0
    }

    pub fn len(&self) -> usize {
        self.facets.len()
    }

    pub fn shrink_to_fit(&mut self) {
        self.facets.shrink_to_fit();
    }

    pub fn has_face(&self, simplex: &Face<Point>) -> bool {
        has_face(self, &simplex.to_vec())
    }

    pub fn iter(&self) -> Iter<Face<Point>> {
        self.into_iter()
    }

    pub fn iter_mut(&mut self) -> IterMut<Face<Point>> {
        self.into_iter()
    }

    pub fn vertex_set(&self) -> SCHashSet<Point> {
        let facet_count = self.len();
        let cap = (facet_count as f32).powf((self.height().min(2) as f32 - 1.0).recip()) as usize;

        let mut vertex_set = self.facets.iter().fold(new_hs::<Point>(cap), |mut f, g| {
            f.extend(g.iter());
            f
        });
        vertex_set.shrink_to_fit();

        vertex_set
    }

    fn intersection_with_simplex(&self, simplex: &Face<Point>) -> Self {
        Self::from_check_maximal(
            self.iter()
                .filter_map(|f| f.maybe_intersection(simplex))
                .collect::<SCHashSet<Face<Point>>>(),
        )
    }

    // Refactor the pb to be an optional parameter
    fn intersection_with_complex(&self, other: &Self, quiet: bool) -> Self {
        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = new_pb(self.len());
            pb.set_message(upd_sty(format!["Intersecting facets"]));
        }

        let mut int_faces = new_hs::<Face<Point>>(max(self.len(), other.len()));

        for facet in &self.facets {
            int_faces.extend(
                other
                    .facets
                    .iter()
                    .filter_map(|g| g.maybe_intersection(facet)),
            );
            pb.inc(1);
        }
        pb.finish();

        Self::from_check_maximal(int_faces)
    }

    fn nerve(&self) -> Self {
        Self::from_check_maximal(self.vertex_set().into_iter().map(|v| {
            (0..self.len())
                .filter(|&i| self.facets[i].contains(&v))
                .map(|i| {
                    i.try_into()
                        .ok()
                        .expect("The number of facets has overflowed.")
                })
                .collect::<Face<Point>>()
        }))
    }

    // Take Čech nerves until both the dimension and the number of vertices are minimized.
    // Return the number of times the nerve was taken. It's not important, but it's nice for the
    // user's edification.
    pub fn nerve_reduce(&mut self, quiet: bool) -> usize {
        let mut n = 0;
        let spnr: ProgressBar;
        if quiet {
            spnr = ProgressBar::hidden();
        } else {
            spnr = new_spnr();
            spnr.set_message(upd_sty(format!["Reduced with Čech nerves {n} times"]));
        }

        let mut base_vertex_count = self.vertex_set().len();
        if base_vertex_count == 0 {
            return 0;
        }
        let mut nerve = self.nerve();
        while (n % 2 == 0 && (nerve.height() < self.height() || nerve.len() < base_vertex_count))
            || (n % 2 != 0 && (nerve.height() > self.height() || nerve.len() > base_vertex_count))
        {
            if n % 2 == 0 {
                *self = nerve.nerve();
                base_vertex_count = self.vertex_set().len();
            } else {
                nerve = self.nerve();
            }
            n += 1;

            if !quiet {
                spnr.set_message(upd_sty(format!["Simplified with Čech nerves {n} times"]));
            }
        }
        if n % 2 != 0 {
            *self = nerve;
        }
        spnr.finish();

        n
    }

    fn is_contractible(&self) -> bool {
        // The empty complex is not contractible
        if self.height() == 0 {
            return false;
        }

        let mut sc = self;
        let mut facet_count = sc.len();
        let mut height;
        let mut nerve;
        let mut nerve_facet_count;
        loop {
            // A single simplex is contractible
            if facet_count == 1 {
                return true;
            }

            // A discrete complex with multiple facets is not contractible
            height = sc.height();
            if height == 1 {
                return false;
            }

            // A complex with two facets is contractible iff the facets overlap
            let vertex_count = sc.vertex_set().len();
            if facet_count == 2 {
                return vertex_count != height + sc.facets[1].len();
            }

            // Test if taking the nerve accomplishes a reduction
            nerve = sc.nerve();
            nerve_facet_count = nerve.len();
            if vertex_count == nerve_facet_count {
                // There exist contractible, non-nerve-reducible complexes, but none with the
                // following properties
                if vertex_count < 5 || facet_count < 5 || height < 3 {
                    return false;
                }

                // If the preceding check was not decisive, we find a contractible subcomplex
                if height > nerve.height() {
                    // The nerve has lesser dimension: perform the check on it.
                    return nerve
                        .first_facet_to_complex()
                        .is_deformation_retract(&mut nerve);
                }

                // The nerve has greater dimension: revert to the previous complex by taking the
                // nerve again (necessary because we have already moved `sc` into `nerve`.
                let mut nsc = nerve.nerve();
                return nsc
                    .first_facet_to_complex()
                    .is_deformation_retract(&mut nsc);
            }

            // Simplify by taking the nerve
            sc = &nerve;
            facet_count = nerve_facet_count;
        }
    }

    // Make the pb an optional parameter
    fn enlarge_from_complex(&mut self, other: &mut Self, care: bool, quiet: bool) -> bool {
        let facet_count = other.len();
        if self.height() == 0 {
            return other.height() == 0;
        }

        let mut n = 0;
        let spnr: ProgressBar;
        if quiet {
            spnr = ProgressBar::hidden();
        } else {
            spnr = new_spnr();
            spnr.set_message(upd_sty(format![
                "Added {n} of {facet_count} facets to the subcomplex"
            ]));
        }

        let check: bool = care && self.facets.par_iter().any(|f| other.has_face(f));
        let other_facets = &mut other.facets;

        let mut queue = new_vd::<Face<Point>>(facet_count);
        queue.extend(
            other_facets.extract_if(.., |of| !self.facets.iter().all(|sf| of.is_disjoint(sf))),
        );

        while let Some(facet) = queue.pop_front() {
            let intrsct = self.intersection_with_simplex(&facet);
            if intrsct.is_contractible() {
                self.facets.push(facet);
                let facet = self.facets.last().unwrap();

                // Parallelize
                queue.extend(other_facets.extract_if(.., |of| {
                    facet
                        .maybe_intersection(of)
                        .is_some_and(|f| !intrsct.has_face(&f))
                }));

                if !quiet {
                    n += 1;
                    spnr.set_message(upd_sty(format![
                        "Added {n} of {facet_count} facets to the subcomplex"
                    ]));
                }
            } else {
                other_facets.push(facet);
            }
        }
        spnr.finish();

        if check {
            self.uniqueify();
            self.maximalify();
        }

        other_facets.is_empty()
    }

    fn is_deformation_retract(&mut self, other: &mut Self) -> bool {
        self.enlarge_from_complex(other, false, true)
        // retract_test(self.iter().collect(), other.iter().collect())
    }

    fn edge_table(&self) -> Vec<(Point, Vec<Point>)> {
        let height = self.height();
        if height < 2 {
            let vs = self.vertex_set();
            let mut out = new_vec::<(Point, Vec<Point>)>(vs.len());
            out.extend(vs.into_iter().map(|i| (i, Vec::new())));

            return out;
        }

        // Set up edges_map
        let mut edges_map: SCHashMap<Point, SCHashSet<Point>> = self
            .vertex_set()
            .into_par_iter()
            .map(|v| (v, new_hs(height.pow(2))))
            .collect();

        self.iter().map(|f| f.tuple()).for_each(|tuple| {
            (1..tuple.len()).for_each(|i| {
                edges_map.get_mut(&tuple[i]).unwrap().extend(&tuple[0..i]);
            })
        });

        let mut edges_vec: Vec<(Point, Vec<Point>)> = edges_map
            .into_iter()
            .filter(|(_, s)| !s.is_empty())
            .map(|(v, s)| (v, to_sorted_vec(&s)))
            .collect();

        edges_vec.sort_unstable_by_key(|(v, _)| Reverse(*v));

        edges_vec
    }

    fn first_facet_to_complex(&mut self) -> Self {
        Self::from_iter([self.facets.remove(0)])
    }

    pub fn contractible_subcomplex(&mut self, quiet: bool) -> Self {
        let mut contractible = Self::from_iter([self.facets.remove(0)]);
        contractible.enlarge_from_complex(self, false, quiet);

        contractible
    }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vert_dict = new_hm::<Point, Point>(vertex_set.len());
        let mut n = Point::zero();
        for v in vertex_set {
            vert_dict.insert(v, n);
            n += Point::one();
        }
        for facet in &mut self.facets {
            facet.replace_verts_from_map(&vert_dict);
        }
    }

    pub fn relabel_vertices_reverse(&mut self) {
        let mut vert_vec: Vec<Point> = self.vertex_set().into_iter().collect();
        vert_vec.sort_unstable_by(|a, b| b.cmp(a));
        let mut vert_dict = new_hm::<Point, Point>(vert_vec.len());
        let mut n = Point::zero();
        for v in vert_vec {
            vert_dict.insert(v, n);
            n += Point::one();
        }
        for facet in &mut self.facets {
            facet.replace_verts_from_map(&vert_dict);
        }
    }

    pub fn minimize_pair(&self, sub: Self) -> Self {
        (&*self).intersection_with_complex(&sub, false)
    }

    pub fn pinch(&mut self, intrpt: Option<Arc<atomic::AtomicBool>>, quiet: bool) -> bool {
        if self.is_empty()
            || intrpt
                .as_ref()
                .is_some_and(|s| s.load(atomic::Ordering::Relaxed))
        {
            return false;
        }
        let mut interrupted = false;
        let mut pinched = false;

        let edges = self.edge_table();
        let vertex_count = edges.len();
        let mut n: usize = 0;

        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = new_pb(vertex_count);
            pb.set_message(upd_sty(format!["Pinched {n} edges"]));
        }

        rayon::ThreadPoolBuilder::new()
            .stack_size(1024 * 1024 * 2)
            .build()
            .unwrap()
            .install(|| {
                'outer: for (old, adj) in edges {
                    pb.inc(1);

                    for new in adj {
                        if intrpt
                            .as_ref()
                            .is_some_and(|s| s.load(atomic::Ordering::Relaxed))
                        {
                            interrupted = true;
                            break 'outer;
                        }

                        let cap = self.len().isqrt();
                        let mut edge_link = new_vec::<usize>(cap);
                        let mut old_link_ext = new_vec::<usize>(cap);
                        let mut new_link_ext = new_vec::<usize>(cap);
                        let mut relevant = new_vec::<(usize, bool, bool)>(2 * cap);

                        relevant.par_extend(
                            self.facets
                                .par_iter_mut()
                                .enumerate()
                                .map(|(i, facet)| (i, facet.remove(&old), facet.remove(&new)))
                                .filter(|(_, old_bool, new_bool)| *old_bool || *new_bool),
                        );

                        for (i, old_bool, new_bool) in relevant {
                            match (old_bool, new_bool) {
                                (true, true) => edge_link.push(i),
                                (true, _) => old_link_ext.push(i),
                                _ => new_link_ext.push(i),
                            }
                        }

                        edge_link.sort_by_key(|i| Reverse(self.facets[*i].len()));
                        let pre_int_faces: SCHashSet<Face<Point>> = new_link_ext
                            .par_iter()
                            .copied()
                            .map(|new_idx| {
                                old_link_ext
                                    .iter()
                                    .filter_map(|old_idx| {
                                        self.facets[*old_idx]
                                            .maybe_intersection(&self.facets[new_idx])
                                    })
                                    .filter(|int_face| {
                                        !has_face(
                                            edge_link.iter().map(|i| &self.facets[*i]),
                                            &int_face.to_vec(),
                                        )
                                    })
                                    .collect::<SCHashSet<Face<Point>>>()
                            })
                            .reduce(
                                || new_hs(0),
                                |mut a, b| {
                                    a.extend(b);
                                    a
                                },
                            );

                        if pre_int_faces.is_empty() || {
                            let mut pre_int_facets: Vec<&Face<Point>> =
                                pre_int_faces.iter().collect();
                            maximalify(&mut pre_int_facets);
                            retract_test(
                                edge_link.iter().map(|i| &self.facets[*i]).collect(),
                                pre_int_facets,
                            )
                        } {
                            new_link_ext.sort_by_key(|i| Reverse(self.facets[*i].len()));
                            old_link_ext.extend(edge_link);
                            let mut rem_or_ins = new_vec::<(usize, bool)>(old_link_ext.len());
                            rem_or_ins.par_extend(old_link_ext.into_par_iter().map(|i| {
                                let face = &self.facets[i];

                                (
                                    i,
                                    pre_int_faces.contains(face)
                                        || has_face(
                                            new_link_ext.iter().map(|j| &self.facets[*j]),
                                            &face.to_vec(),
                                        ),
                                )
                            }));

                            drop(pre_int_faces);

                            new_link_ext
                                .into_iter()
                                .chain(rem_or_ins.extract_if(.., |(_, b)| !*b).map(|p| p.0))
                                .for_each(|i| {
                                    self.facets[i].insert(new);
                                });

                            rem_or_ins.sort_unstable_by_key(|p| Reverse(p.0));
                            rem_or_ins.into_iter().for_each(|(i, _)| {
                                self.facets.swap_remove(i);
                            });

                            n += 1;
                            pb.set_message(upd_sty(format!["Pinched {n} edges"]));

                            pinched = true;
                            break;
                        } else {
                            drop(pre_int_faces);

                            let edge: Vec<Point> = vec![old, new];

                            edge_link.into_iter().for_each(|i| {
                                self.facets[i].extend(edge.iter().copied());
                            });

                            old_link_ext.into_iter().for_each(|i| {
                                self.facets[i].insert(old);
                            });

                            new_link_ext.into_iter().for_each(|i| {
                                self.facets[i].insert(new);
                            });
                        }
                    }
                }
            });

        self.sortify();

        if !interrupted {
            pb.finish();

            // The pinch algorithm is sensitive to the ordering of the vertices. Relabeling the
            // vertices shakes things up to facilitate further pinches.
            self.relabel_vertices();
        }

        pinched
    }
}

fn retract_test<'a, Point: Vertex>(
    mut sc: Vec<&'a Face<Point>>,
    mut rem: Vec<&'a Face<Point>>,
) -> bool {
    let mut queue = new_vd::<&Face<Point>>(rem.len());

    queue.extend(rem.extract_if(.., |facet| sc.iter().any(|f| !facet.is_disjoint(f))));

    while let Some(facet) = queue.pop_front() {
        let intrsct = intersection_with_simplex(&sc, &facet);
        if intrsct.is_contractible() {
            sc.push(facet);
            let facet = sc.last().unwrap();

            queue.extend(rem.extract_if(.., |nf| {
                facet
                    .maybe_intersection(nf)
                    .is_some_and(|f| !intrsct.has_face(&f))
            }));
        } else {
            rem.push(facet);
        }
    }

    rem.is_empty()
}

fn maximalify<Point: Vertex>(sc: &mut Vec<&Face<Point>>) {
    sc.sort_unstable_by_key(|f| Reverse(f.len()));
    if sc[0].len() == sc.last().unwrap().len() {
        return;
    }

    let mut i = 1;
    while i < sc.len() {
        let face: Vec<Point> = sc[i].to_vec();
        let len = face.len();
        if sc
            .iter()
            .take_while(|f| f.len() > len)
            .any(|f| face.iter().all(|v| f.contains(v)))
        {
            sc.remove(i);
        } else {
            i += 1;
        }
    }
}

fn intersection_with_simplex<Point: Vertex>(
    sc: &Vec<&Face<Point>>,
    facet: &Face<Point>,
) -> SimplicialComplex<Point> {
    SimplicialComplex::from_check_maximal(
        sc.into_iter()
            .filter_map(|f| f.maybe_intersection(facet))
            .collect::<SCHashSet<Face<Point>>>(),
    )
}

// &Face<Point> should become Item = &S where S: Simplex<Point = Point>
fn has_face<'a, Point: Vertex + 'a>(
    facets: impl IntoIterator<Item = &'a Face<Point>>,
    face: &Vec<Point>,
) -> bool {
    let len = face.len();
    facets
        .into_iter()
        .take_while(|facet| facet.len() >= len)
        .any(|facet| face.into_iter().all(|v| facet.contains(v)))
}
