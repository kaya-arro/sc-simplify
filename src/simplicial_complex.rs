use crate::min;
use crate::{BitAnd, Default, Reverse};
use std::mem::take;

use crate::Simplex;
use crate::upd_sty;
use crate::{HashSet, new_hm, new_hs, new_vd, new_vec, to_sorted_vec};
use crate::{ProgressBar, new_pb, new_spnr};

pub struct SimplicialComplex {
    pub facets: Vec<Simplex>,
}

impl Default for SimplicialComplex {
    fn default() -> Self {
        Self {
            facets: vec![Simplex::default()],
        }
    }
}

impl From<&Simplex> for SimplicialComplex {
    fn from(simplex: &Simplex) -> Self {
        Self {
            facets: vec![simplex.clone()],
        }
    }
}

impl From<Vec<Simplex>> for SimplicialComplex {
    fn from(mut facets: Vec<Simplex>) -> Self {
        facets.select_nth_unstable_by_key(0, |f| Reverse(f.len()));
        Self { facets }
    }
}

impl BitAnd for &SimplicialComplex {
    type Output = SimplicialComplex;

    fn bitand(self, rhs: Self) -> SimplicialComplex {
        self.intersection_with_complex(rhs)
    }
}

impl SimplicialComplex {
    pub fn from_check(facets: Vec<Simplex>) -> Self {
        let mut sc = Self { facets };
        sc.maximalify();

        sc
    }

    pub fn maximalify(&mut self) {
        if self.facets.is_empty() {
            self.facets.push(Simplex::default());
        } else {
            let mut face_holder = take(&mut self.facets);
            face_holder.sort_by_key(|f| Reverse(f.len()));

            for face in face_holder {
                if !self
                    .facets
                    .iter()
                    .take_while(|f| f.len() >= face.len())
                    .any(|f| &face <= f)
                {
                    self.facets.push(face);
                }
            }

            self.facets.shrink_to_fit();
        }
    }

    pub fn height(&self) -> usize {
        self.facets[0].len()
    }

    pub fn facet_count(&self) -> usize {
        self.facets.len()
    }

    fn contains(&self, s: &Simplex) -> bool {
        self.facets.iter().any(|f| s <= f)
    }

    pub fn vertex_set(&self) -> HashSet<u32> {
        let fc = self.facet_count();
        let cap = (fc as f32).powf((self.height() as f32 - 1.0).recip()) as usize;

        let mut vertex_set = self.facets.iter().fold(new_hs::<u32>(cap), |mut f, g| {
            f.extend(g.vertices.iter());
            f
        });
        vertex_set.shrink_to_fit();

        vertex_set
    }

    fn intersection_with_simplex(&self, other: &Simplex) -> Self {
        Self::from_check(
            self.facets
                .iter()
                .map(|f| f & other)
                .collect::<HashSet<Simplex>>()
                .into_iter()
                .collect(),
        )
    }

    fn intersection_with_complex(&self, other: &Self) -> Self {
        let mut int_faces = new_hs::<Simplex>(min(self.facet_count(), other.facet_count()));
        for facet in &self.facets {
            for other_facet in &other.facets {
                if !facet.is_disjoint(&other_facet) {
                    int_faces.insert(facet & other_facet);
                }
            }
        }

        Self::from_check(int_faces.into_iter().collect())
    }

    fn nerve(&self) -> Self {
        let vertex_set = self.vertex_set();
        let facet_count = self.facet_count();

        let mut nerve_faces = new_hs::<Simplex>(vertex_set.len());
        for v in vertex_set {
            let mut nerve_simp_verts = new_hs::<u32>(facet_count);
            nerve_simp_verts.extend((0..facet_count).filter_map(|i| {
                if self.facets[i].contains(&v) {
                    Some(i as u32)
                } else {
                    None
                }
            }));
            nerve_simp_verts.shrink_to_fit();
            nerve_faces.insert(Simplex::from(nerve_simp_verts));
        }

        Self::from_check(nerve_faces.into_iter().collect())
    }

    // Take Čech nerves until both the dimension and the number of vertices are minimized.
    // Return the number of times the nerve was taken. It's not important, but it's nice for the
    // user's edification.
    pub fn reduce(&mut self, quiet: bool) -> usize {
        let mut n = 0usize;
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
        while (n % 2 == 0
            && (nerve.height() < self.height() || nerve.facet_count() < base_vertex_count))
            || (n % 2 != 0
                && (nerve.height() > self.height() || nerve.facet_count() > base_vertex_count))
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
        if self.height() == 0 {
            return false;
        }
        let mut sc = self;
        let mut facet_count = sc.facet_count();
        let mut height: usize;
        let mut nerve: Self;
        let mut nerve_facet_count: usize;
        loop {
            if facet_count == 1 {
                return true;
            }
            height = sc.height();
            if height == 1 {
                return false;
            }
            let vertex_count = sc.vertex_set().len();
            if facet_count == 2 {
                return vertex_count != height + sc.facets[1].vertices.len();
            }

            nerve = sc.nerve();
            nerve_facet_count = nerve.facet_count();
            if vertex_count == nerve_facet_count {
                if vertex_count < 5 || facet_count < 5 || height < 3 {
                    return false;
                }
                if height > nerve.height() {
                    // The nerve has lesser dimension: perform the check on it.
                    return nerve
                        .first_facet_to_complex()
                        .is_deformation_retract(&nerve);
                }
                // The nerve has greater dimension: revert to the previous complex by taking the
                // nerve again (necessary because we have already moved `sc` into `nerve`.
                let nsc = nerve.nerve();
                return nsc.first_facet_to_complex().is_deformation_retract(&nsc);
            }
            sc = &nerve;
            facet_count = nerve_facet_count;
        }
    }

    fn enlarge_in_supercomplex(&mut self, supercomplex: &Self, care: bool, quiet: bool) -> bool {
        let fc = supercomplex.facet_count();

        let mut n = self.facet_count();
        let spnr: ProgressBar;
        if quiet {
            spnr = ProgressBar::hidden();
        } else {
            spnr = new_spnr();
            spnr.set_message(upd_sty(format![
                "Added {n} of {fc} facets to the subcomplex"
            ]));
        }

        let check: bool = care && !self.facets.iter().all(|f| supercomplex.facets.contains(f));

        let mut queue = new_vd::<&Simplex>(fc);
        let mut rem = new_vd::<&Simplex>(fc);
        rem.extend(&supercomplex.facets);

        // The last `i` items of `rem` have already been checked, so we should not use
        // `while let Some(facet) = rem.pop_front()` here lest we loop forever.
        let mut i = 0;
        while i < rem.len() {
            let facet = rem.pop_front().unwrap();
            if self.facets.iter().any(|f| !facet.is_disjoint(f)) {
                queue.push_back(facet);
            } else {
                i += 1;
                rem.push_back(facet);
            }
        }

        // Refactor so that we can avoid cloning by say using `Rc`s
        // This requires allowing SimplicialComplex<T>
        while let Some(facet) = queue.pop_front() {
            let intrsct = self.intersection_with_simplex(facet);
            if intrsct.is_contractible() {
                self.facets.push(facet.clone());
                if !quiet {
                    n += 1;
                    spnr.set_message(upd_sty(format![
                        "Added {n} of {fc} facets to the subcomplex"
                    ]));
                }
                let mut i = 0;
                while i < rem.len() {
                    if let Some(nf) = rem.pop_front() {
                        // add a `rem.len() < 5 ||` or similar condition here to control the test
                        // strategy. Consider using !is_disjoint or some such to pre-filter or
                        // filter; consider taking all of rem when rem is sufficiently small.
                        if !intrsct.contains(&(facet & nf)) {
                            queue.push_back(nf);
                        } else {
                            rem.push_back(nf);
                            i += 1;
                        }
                    }
                }
            } else {
                rem.push_back(facet);
            }
        }
        spnr.finish();

        if check {
            self.maximalify();
        }

        rem.is_empty()
    }

    fn is_deformation_retract(&mut self, supercomplex: &Self) -> bool {
        self.enlarge_in_supercomplex(supercomplex, false, true)
    }

    fn links(&self, faces: Vec<Simplex>) -> Vec<Self> {
        let fc = self.facet_count();
        let mut link_sets = new_vec::<(Simplex, HashSet<Simplex>)>(faces.len());
        link_sets.extend(faces.into_iter().map(|f| (f, new_hs(fc))));
        for facet in &self.facets {
            for &mut (ref face, ref mut set) in &mut link_sets {
                if face <= facet {
                    set.insert(facet - face);
                }
            }
        }

        link_sets
            .into_iter()
            .map(|s| Self {
                facets: s.1.into_iter().collect(),
            })
            .collect()
    }

    fn pinch_check(&self, old: u32, new: u32) -> bool {
        let mut triple = self.links(vec![
            Simplex::from([old].iter().copied().collect::<HashSet<u32>>()),
            Simplex::from([new].iter().copied().collect::<HashSet<u32>>()),
            Simplex::from([old, new].iter().copied().collect::<HashSet<u32>>()),
        ]);
        let intersection = &triple[0] & &triple[1];

        triple[2].is_deformation_retract(&intersection)
    }

    fn edge_table(&self) -> Vec<(u32, Vec<u32>)> {
        let height = self.height();
        if height < 2 {
            let vs = self.vertex_set();
            let mut out = new_vec::<(u32, Vec<u32>)>(vs.len());
            out.extend(vs.into_iter().map(|i| (i, Vec::new())));

            return out;
        }

        // Set up edges_map
        let vertex_vec = to_sorted_vec(&self.vertex_set());
        let vert_count = vertex_vec.len();
        // Try collect here
        let mut edges_map = new_hm::<u32, HashSet<u32>>(vert_count);

        for v in &vertex_vec {
            edges_map.insert(*v, new_hs::<u32>(height));
        }

        for facet in &self.facets {
            let len = facet.len();
            let tuple = facet.tuple();
            for i in 1..len {
                if let Some(edge_set) = edges_map.get_mut(&tuple[i]) {
                    edge_set.extend(&tuple[0..i]);
                }
            }
        }

        let mut edge_vec = new_vec::<(u32, Vec<u32>)>(vert_count);
        for v in vertex_vec {
            let v_edge_set = &edges_map[&v];
            if !v_edge_set.is_empty() {
                let v_edge_vec = to_sorted_vec(v_edge_set);
                edge_vec.push((v, v_edge_vec));
            }
        }

        return edge_vec;
    }

    pub fn pinch(&mut self, quiet: bool) -> bool {
        let mut pinched = false;

        let mut edges = self.edge_table();
        let vert_count: usize = edges.len();

        let mut n: usize;
        n = 0;

        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = new_pb(vert_count);
            pb.set_message(upd_sty(format!["Pinched {n} edges"]));
        }

        while let Some((old, mut adj)) = edges.pop() {
            pb.inc(1);

            while let Some(new) = adj.pop() {
                if self.pinch_check(old, new) {
                    for facet in &mut self.facets {
                        if facet.remove(&old) {
                            facet.insert(&new);
                        }
                    }

                    n += 1;
                    pb.set_message(upd_sty(format!["Pinched {n} edges"]));

                    pinched = true;
                    break;
                }
            }
        }
        pb.finish();

        self.maximalify();
        pinched
    }

    fn first_facet_to_complex(&self) -> Self {
        Self {
            facets: vec![self.facets[0].clone()],
        }
    }

    pub fn contractible_subcomplex(&self, quiet: bool) -> Self {
        let mut contractible = self.first_facet_to_complex();
        contractible.enlarge_in_supercomplex(self, false, quiet);

        contractible
    }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vertex_dict = new_hm::<u32, u32>(vertex_set.len());
        let mut n = 0u32;
        for v in vertex_set {
            vertex_dict.insert(v, n);
            n += 1;
        }
        for facet in &mut self.facets {
            facet.vertices = facet.vertices.iter().map(|v| vertex_dict[v]).collect();
        }
    }
}

pub fn minimize_pair(
    (SimplicialComplex { facets: sup_facets }, sub): (SimplicialComplex, SimplicialComplex),
) -> (SimplicialComplex, SimplicialComplex) {
    let sub_facet_set: HashSet<&Simplex> = sub.facets.iter().collect();
    let rem_facets: Vec<Simplex> = sup_facets
        .into_iter()
        .filter(|f| !sub_facet_set.contains(f))
        .collect();

    let rem = SimplicialComplex { facets: rem_facets };
    let bnd = &rem & &sub;

    (rem, bnd)
}
