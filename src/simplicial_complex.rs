use std::cmp::min;
use std::time::Duration;
use rustc_hash::FxHashMap as HashMap;

use crate::{BitAnd, Default, ProgressBar, update_style, info_style, info_number_style, the_sty, HashSet, simplex::Simplex, the_hasher, new_hs, new_v, to_v};


fn len_sort(vec: &mut Vec<Simplex>) {
    vec.sort_by(|a, b| b.len().cmp(&a.len()));
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimplicialComplex {
    pub facets: Vec<Simplex>,
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

    pub fn from_check(mut faces: Vec<Simplex>) -> Self {
        match faces.len() {
            0 => Self::default(),
            _ => {
                len_sort(&mut faces);
                let mut facets = new_v::<Simplex>(faces.len());
                for old_facet in faces {
                    if !facets.iter().any(
                        |new_facet| old_facet <= *new_facet
                    ) {
                        facets.push(old_facet);
                    }
                }
                facets.shrink_to_fit();

                Self { facets }
            }
        }
    }

    pub fn first_len(&self) -> usize {
        return self.facets[0].len();
    }

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
                if !facet.0.is_disjoint(&other_facet.0) {
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
    pub fn reduce(&mut self, quiet: bool) -> u8 {
        let mut n = 0u8;
        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_message(
                format![
                    "{} {} {}",
                    info_style().apply_to("Simplified with Čech nerves"),
                    info_number_style().apply_to(format!["{}", n]),
                    info_style().apply_to("times."),
                ]
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
        }
        if n % 2 != 0 {
            *self = nerve;
        }
        pb.finish_and_clear();

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

    fn enlarge_in_supercomplex(&mut self, supercomplex: &Self, quiet: bool) -> Vec<Simplex> {
        let check: bool;
        if !self.facets.iter().all(|f| supercomplex.facets.contains(f)) {
            check = true;
        } else {
            check = false;
        }

        let mut remainder = new_v::<&Simplex>(supercomplex.facets.len());
        remainder.extend(supercomplex.facets.iter().filter(|f| !self.facets.contains(f)));
        remainder.shrink_to_fit();

        let mut remove_these = new_hs::<&Simplex>(remainder.len());
        let mut done = false;
        while !done {
            done = true;

            let mut n = 0;

            let r_len = remainder.len();
            let pb: ProgressBar;
            if quiet {
                pb = ProgressBar::hidden();
            } else {
                pb = ProgressBar::new(r_len as u64);
                pb.set_style(the_sty());
                pb.set_message(
                    format!["{}", update_style().apply_to(format!["Added {n} facets"])]
                );
            }

            for f in &remainder {
                pb.inc(1);
                if self.intersection_with_simplex(f).is_contractible() {
                    self.facets.push((*f).clone());
                    remove_these.insert(f);

                    done = false;

                    n += 1;
                    pb.set_message(
                        format!["{}", update_style().apply_to(format!["Added {n} facets"])]
                    );
                }
            }

            remainder.retain(|s| !remove_these.contains(s));
            remainder.shrink_to_fit();
            remove_these.clear();
            remove_these.shrink_to(remainder.len());

            pb.finish();
        }

        self.facets.sort_by_key(Simplex::len);
        let mut facets = new_v::<Simplex>(self.facets.len());
        for old_facet in &self.facets {
            if !facets.iter().any(|new_facet| old_facet <= new_facet) {
                facets.push(old_facet.clone());
            }
        }
        facets.shrink_to_fit();

        if check {
            *self = Self::from_check(facets);
        } else {
            *self = Self { facets };
        }
        return remainder.into_iter().map(|f| f.clone()).collect();
    }

    fn is_deformation_retract(&mut self, supercomplex: &Self) -> bool {
        return self.enlarge_in_supercomplex(supercomplex, true).is_empty();
    }

    fn links(&self, faces: Vec<Simplex>) -> Vec<Self> {
        let facets_len = self.facets.len();
        let mut link_sets = new_v::<(Simplex, HashSet<Simplex>)>(faces.len());
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

    // Returns vec holding (v, s) where v is a vertex and s is vec of edges less than v connected
    // to v.
    fn edge_table(&self) -> Vec<(u32,Vec<u32>)> {
        let first_len = self.first_len();
        if first_len < 2 {
            let vs = self.vertex_set();
            let mut out = new_v::<(u32, Vec<u32>)>(vs.len());
            out.extend(vs.into_iter().map(|i| (i, Vec::new())));

            return out;
        }

        // Set up edges_map
        let mut vertex_vec = to_v(&self.vertex_set());
        vertex_vec.sort_unstable_by(|a, b| b.cmp(a));
        let vert_count = vertex_vec.len();
        let mut edges_map = HashMap::<u32, HashSet<u32>>::with_capacity_and_hasher(
            vert_count, the_hasher()
        );

        for v in &vertex_vec {
            edges_map.insert(*v, new_hs::<u32>(first_len));
        }

        for facet in &self.facets {
            let len = facet.len();
            let mut tuple = to_v(&facet.0);
            tuple.sort_unstable_by(|a, b| b.cmp(a));
            for i in 0..len - 1 {
                if let Some(edge_set) = edges_map.get_mut(&tuple[i]) {
                    edge_set.extend((i + 1..len).map(|j| tuple[j]));
                };
            }
        }
        let mut edge_vec = new_v::<(u32, Vec<u32>)>(vert_count);
        for v in vertex_vec {
            let v_edge_set = &edges_map[&v];
            let mut v_edge_vec = to_v(v_edge_set);
            v_edge_vec.sort_unstable_by(|a, b| b.cmp(a));
            edge_vec.push((v, v_edge_vec));
        }

        return edge_vec
    }

    pub fn pinch(&mut self, quiet: bool) -> bool {
        let mut pinched = false;

        let edges = self.edge_table();
        let vert_count: usize = edges.len();

        let mut n: usize;
        n = 0;

        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = ProgressBar::new(vert_count as u64);
            pb.set_style(the_sty());
            pb.set_message(
                format!["{}", update_style().apply_to(format!["Pinched {n} edges"])]
            );
        }

        for entry in edges {
            pb.inc(1);

            let old = entry.0;
            let adj_edges = entry.1;

            for &new in &adj_edges {

                // let o_s = Simplex::from([old]);
                // let n_s = Simplex::from([new]);
                // let e_s = Simplex::from([old, new]);

                let triple = self.links(vec![
                    Simplex::from([old]),
                    Simplex::from([new]),
                    Simplex::from([old, new]),
                ]);
                let o_link = &triple[0];
                let n_link = &triple[1];
                let mut e_link = triple[2].clone();
                let intersection = o_link & n_link;
                if e_link.is_deformation_retract(&intersection) {
                    for facet in &mut self.facets {
                        if facet.remove(&old) {
                            facet.insert(&new);
                        }
                    }

                    n += 1;
                    pb.set_message(
                        format!["{}", update_style().apply_to(format!["Pinched {n} edges"])]
                    );

                    pinched = true;
                    break;
                }
            }
        }
        pb.finish();

        *self = Self::from_check(self.facets.clone());
        pinched
    }

    pub fn collapse(&mut self, quiet: bool) -> bool {
        let mut collapsed = false;
        let facets = &self.facets;
        let mut active_faces = new_hs::<Simplex>(facets.len());
        for facet in facets {
            active_faces.extend(facet.faces());
        }
        // let p_len = active_faces.len();
        let p_len = facets.len();

        let mut n = 0u32;
        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = ProgressBar::new(p_len as u64);
            pb.set_style(the_sty());
            pb.set_message(
                format!["{}", update_style().apply_to(format!["Collapsed {n} faces"])]
            );
        }

        for facet in facets.clone() {
            pb.inc(1);
            if !self.facets.contains(&facet) { continue; }
            for face in facet.faces() {
                if self.links(vec![face.clone()])[0].is_contractible() {
                    active_faces.remove(&face);
                    self.facets.retain(|f| !(*f <= facet));
                    self.facets.extend(facet.faces().into_iter().filter(|f| *f != face));
                    *self = Self::from_check(self.facets.clone());

                    collapsed = true;
                    n += 1;
                    pb.set_message(
                        format!["{}", update_style().apply_to(format!["Collapsed {n} faces"])]
                    );

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
        contractible.enlarge_in_supercomplex(self, quiet);

        contractible
    }

    pub fn relabel_vertices(&mut self) {
        let vertex_set = self.vertex_set();
        let mut vertex_dict: HashMap<u32, u32> =
        HashMap::with_capacity_and_hasher(vertex_set.len(), the_hasher());
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
