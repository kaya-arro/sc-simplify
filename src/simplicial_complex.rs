use crate::min;
use crate::{BitAnd, Default};

use crate::{ProgressBar, new_pb, new_spnr};
use crate::upd_sty;
use crate::{HashSet, new_hs, new_hm, new_vec, new_vd, to_sorted_vec};
// use crate::{Rc, HashMap, VecDeque};
use crate::Simplex;


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimplicialComplex {
    pub facets: Vec<Simplex>
}

impl Default for SimplicialComplex {
    fn default() -> Self {
        Self { facets: vec![Simplex::default()] }
    }
}

impl From<&Simplex> for SimplicialComplex {
    fn from(simplex: &Simplex) -> Self {
        Self { facets: vec![simplex.clone()] }
    }
}

impl From<Vec<Simplex>> for SimplicialComplex {
    fn from(mut facets: Vec<Simplex>) -> Self {
        facets.select_nth_unstable_by_key(0, Simplex::len);
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
            self.facets.push(Simplex::default())
        } else {
            self.facets.sort_by(|a, b| b.len().cmp(&a.len()));
            let mut face_holder = new_vec::<Simplex>(self.facets.len());
            face_holder.append(&mut self.facets);
            for face in face_holder.into_iter() {
                if !self.facets.iter().any(
                    |facet| face <= *facet
                ) {
                    self.facets.push(face);
                }
            }
            self.facets.shrink_to_fit();
        }
    }

    pub fn first_len(&self) -> usize {
        self.facets[0].len()
    }

    pub fn contains(&self, s: &Simplex) -> bool { self.facets.iter().any(|f| s <= f) }

    pub fn vertex_set(&self) -> HashSet<u32> {
        let mut vertex_set = new_hs::<u32>(self.facets.len());
        for facet in &self.facets {
            vertex_set.extend(&facet.0);
        }
        vertex_set.shrink_to_fit();

        vertex_set
    }

    fn intersection_with_simplex(&self, other: &Simplex) -> Self {
        let mut int_faces = new_hs::<Simplex>(self.facets.len());
        int_faces.extend(self.facets.iter().map(|f| f & other));

        Self::from_check(int_faces.into_iter().collect())
    }

    fn intersection_with_complex(&self, other: &Self) -> Self {
        let mut int_faces = new_hs::<Simplex>(min(self.facets.len(), other.facets.len()));
        for facet in &self.facets {
            for other_facet in &other.facets {
                if !facet.is_disjoint(&other_facet) {
                    int_faces.insert(facet & other_facet);
                }
            }
        }

        Self::from_check(int_faces.into_iter().collect())
    }

    pub fn nerve(&self) -> Self {
        let vertex_set = self.vertex_set();
        let facet_count = self.facets.len();

        let mut nerve_faces = new_hs::<Simplex>(vertex_set.len());
        for v in vertex_set {
            let mut nerve_simp_verts = new_hs::<u32>(facet_count);
            nerve_simp_verts.extend(
                (0..facet_count).filter(|i| self.facets[*i].contains(&v)).map(|i| i as u32)
            );
            nerve_simp_verts.shrink_to_fit();
            nerve_faces.insert(Simplex(nerve_simp_verts));
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
            spnr.set_message(
                upd_sty(format!["Reduced with Čech nerves {n} times"])
            );
        }

        let mut base_vertex_count = self.vertex_set().len();
        if base_vertex_count == 0 {
            return 0;
        }
        let mut nerve = self.nerve();
        while (n % 2 == 0
            && (nerve.first_len() < self.first_len() || nerve.facets.len() < base_vertex_count))
            || (n % 2 != 0
            && (nerve.first_len() > self.first_len() || nerve.facets.len() > base_vertex_count))
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
                if vertex_count < 5 || facet_count < 5 || first_len < 3 { return false; }
                if first_len > nerve.first_len() {
                    // The nerve has lesser dimension: perform the check on it.
                    return nerve.first_facet_to_complex().is_deformation_retract(&nerve);
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
        let fc = supercomplex.facets.len();
        let mut n = self.facets.len();
        let spnr: ProgressBar;
        if quiet {
            spnr = ProgressBar::hidden();
        } else {
            spnr = new_spnr();
            spnr.set_message(upd_sty(format!["Added {n} of {fc} facets to the subcomplex"]));
        }

        let check: bool = care && !self.facets.iter().all(|f| supercomplex.facets.contains(f));

        let mut queue = new_vd::<&Simplex>(fc);
        let mut rem = new_vd::<&Simplex>(fc);
        rem.extend(&supercomplex.facets);

        let mut i = 0;
        while i < rem.len() {
            if let Some(facet) = rem.pop_front() {
                let mut was_queued = false;
                for s_facet in &self.facets {
                    if !s_facet.is_disjoint(facet) {
                        was_queued = true;
                        queue.push_back(facet);
                        break
                    }
                }
                if !was_queued {
                    i += 1;
                    rem.push_back(facet);
                }
            }
        }

        while let Some(facet) = queue.pop_front() {
            let intrsct = self.intersection_with_simplex(facet);
            if intrsct.is_contractible() {
                self.facets.push(facet.clone());
                if !quiet {
                    n += 1;
                    spnr.set_message(
                        upd_sty(format!["Added {n} of {fc} facets to the subcomplex"])
                    );
                }
                let mut i = 0;
                while i < rem.len() {
                    if let Some(nf) = rem.pop_front() {
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
        return self.enlarge_in_supercomplex(supercomplex, false, true);
    }

    fn links(&self, faces: Vec<Simplex>) -> Vec<Self> {
        let facets_len = self.facets.len();
        let mut link_sets = new_vec::<(Simplex, HashSet<Simplex>)>(faces.len());
        link_sets.extend(faces.into_iter().map(|f| (f, new_hs(facets_len))));
        for facet in &self.facets {
            for &mut (ref face, ref mut set) in &mut link_sets {
                if face <= facet {
                    set.insert(facet - face);
                }
            }
        }

        link_sets.into_iter().map(|s| Self { facets: s.1.into_iter().collect() } ).collect()
    }

    fn pinch_check(&self, old: u32, new: u32) -> bool {
        let mut triple = self.links(vec![
            Simplex::from([old]),
                                    Simplex::from([new]),
                                    Simplex::from([old, new]),
        ]);
        let intersection = &triple[0] & &triple[1];

        triple[2].is_deformation_retract(&intersection)
    }

    fn edge_table(&self) -> Vec<(u32,Vec<u32>)> {
        let first_len = self.first_len();
        if first_len < 2 {
            let vs = self.vertex_set();
            let mut out = new_vec::<(u32, Vec<u32>)>(vs.len());
            out.extend(vs.into_iter().map(|i| (i, Vec::new())));

            return out;
        }

        // Set up edges_map
        let vertex_vec = to_sorted_vec(&self.vertex_set());
        let vert_count = vertex_vec.len();
        let mut edges_map = new_hm::<u32, HashSet<u32>>(vert_count);

        for v in &vertex_vec {
            edges_map.insert(*v, new_hs::<u32>(first_len));
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

        return edge_vec
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
                // eprintln!["old: {old}; new: {new}"];
                if self.pinch_check(old, new) {
                    for facet in &mut self.facets {
                        if facet.remove(&old) { facet.insert(&new); }
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

    // To do: write a version that collapses smaller cells too, not just codimension 1 faces.
    pub fn collapse(&mut self, quiet: bool) -> bool {
        let mut collapsed = false;
        let facets = &self.facets;
        let p_len = facets.len();

        let mut n = 0u32;
        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = new_pb(p_len);
            pb.set_message(upd_sty(format!["Collapsed {n} faces"]));
        }

        for facet in facets.clone() {
            pb.inc(1);
            if !self.facets.contains(&facet) { continue; }
            for face in facet.faces() {
                // When working with a codimension 1 face, it isn't really necessary to calculate
                // the link, but this code will make the method more extensible to smaller cells
                // down the line.
                if self.links(vec![face.clone()])[0].is_contractible() {
                    self.facets.retain(|f| !(*f <= facet));
                    self.facets.extend(facet.faces().into_iter().filter(|f| *f != face));
                    *self = Self::from_check(self.facets.clone());

                    collapsed = true;
                    n += 1;
                    pb.set_message(upd_sty(format!["Collapsed {n} faces"]));

                    break;
                }
            }
        }
        pb.finish();

        collapsed
    }

    fn first_facet_to_complex(&self) -> Self {
        Self { facets: vec![self.facets[0].clone()] }
    }

    pub fn contractible_subcomplex(&self, quiet: bool) -> Self {
        let mut contractible = self.first_facet_to_complex();
        contractible.enlarge_in_supercomplex(self, false, quiet);

        contractible
    }

    // pub fn cofaces(&self, s: &Simplex) -> Vec<Simplex> {
    //     // Is there a good heuristic for the capacity to use?
    //     let mut cb = new_hs::<u32>(1);
    //     for f in self.facets.iter().filter(|f| s <= f) {
    //         cb.extend(f.0.iter().filter(|v| !s.contains(v)));
    //     }
    //
    //     let mut res = new_vec::<Simplex>(cb.len());
    //     res.extend(cb.into_iter().map(|v| s.add_vertex(&v)));
    //     res
    // }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vertex_dict = new_hm::<u32, u32>(vertex_set.len());
        let mut n = 0u32;
        for v in vertex_set {
            vertex_dict.insert(v, n);
            n += 1;
        }
        for facet in &mut self.facets {
            facet.0 = facet.0.iter().map(|v| vertex_dict[v]).collect();
        }
    }

}
