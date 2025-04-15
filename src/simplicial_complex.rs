use crate::min;
use crate::Duration;
use crate::{BitAnd, Default};

use crate::{ProgressBar, update_style, info_style, info_number_style, the_sty};
use crate::{HashSet, new_hs, new_hm, new_vec, new_vd, to_rev_sorted_vec};
use crate::{Rc, HashMap, VecDeque};
use crate::{Simplex, SComplex};


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
        let facets = &mut self.facets;
        match facets.len() {
            0 => facets.push(Simplex::default()),
            _ => {
                facets.sort_by(|a, b| b.len().cmp(&a.len()));
                let mut face_holder = new_vec::<Simplex>(facets.len());
                face_holder.append(facets);
                for face in face_holder.into_iter() {
                    if !facets.iter().any(
                        |facet| face <= *facet
                    ) {
                        facets.push(face);
                    }
                }
                facets.shrink_to_fit();
            }
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

                if !quiet {
                    pb.set_message(
                        format![
                            "{} {} {}",
                            info_style().apply_to("Simplified with Čech nerves"),
                                   info_number_style().apply_to(format!["{}", n]),
                                   info_style().apply_to("times."),
                        ]
                    );
                }
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

    fn enlarge_in_supercomplex(&mut self, supercomplex: &Self, care: bool, quiet: bool) -> bool {
        let mut n = 0usize;
        let pb: ProgressBar;
        if quiet {
            pb = ProgressBar::hidden();
        } else {
            pb = ProgressBar::new_spinner();
            pb.enable_steady_tick(Duration::from_millis(100));
            pb.set_message(
                format![
                    "{} {} {}",
                    info_style().apply_to("Added"),
                           info_number_style().apply_to(format!["{}", n]),
                           info_style().apply_to("facets to the subcomplex."),
                ]
            );
        }

        let check: bool = care && !self.facets.iter().all(|f| supercomplex.facets.contains(f));

        let fc = supercomplex.facets.len();
        let mut queue = new_vd::<&Simplex>(fc);
        let mut rem = new_vd::<&Simplex>(fc);
        rem.extend(&supercomplex.facets);

        // Alternatively, try using swap_take. Benchmark.
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
                    pb.set_message(
                        format![
                            "{} {} {}",
                            info_style().apply_to("Added"),
                                   info_number_style().apply_to(format!["{}", n]),
                                   info_style().apply_to("facets to the subcomplex."),
                        ]
                    );
                }
                let mut i = 0;
                while i < rem.len() {
                    if let Some(nf) = rem.pop_front() {
                        if !facet.is_disjoint(nf) && !intrsct.contains(&(facet & nf)) {
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

        pb.finish();

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

    // Returns vec holding (v, s) where v is a vertex and s is vec of edges less than v connected
    // to v.
    fn edge_table(&self) -> Vec<(u32,Vec<u32>)> {
        let first_len = self.first_len();
        if first_len < 2 {
            let vs = self.vertex_set();
            let mut out = new_vec::<(u32, Vec<u32>)>(vs.len());
            out.extend(vs.into_iter().map(|i| (i, Vec::new())));

            return out;
        }

        // Set up edges_map
        let vertex_vec = to_rev_sorted_vec(&self.vertex_set());
        let vert_count = vertex_vec.len();
        let mut edges_map = new_hm::<u32, HashSet<u32>>(vert_count);

        for v in &vertex_vec {
            edges_map.insert(*v, new_hs::<u32>(first_len));
        }

        for facet in &self.facets {
            let len = facet.len();
            let tuple = facet.tuple();
            for i in 0..len - 1 {
                if let Some(edge_set) = edges_map.get_mut(&tuple[i]) {
                    edge_set.extend((i + 1..len).map(|j| tuple[j]));
                };
            }
        }
        let mut edge_vec = new_vec::<(u32, Vec<u32>)>(vert_count);
        for v in vertex_vec {
            let v_edge_set = &edges_map[&v];
            let v_edge_vec = to_rev_sorted_vec(v_edge_set);
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

            for new in adj_edges {

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
                // When working with a codimension 1 face, it isn't really necessary to calculate
                // the link, but this code will make the method more extensible to smaller cells
                // down the line.
                if self.links(vec![face.clone()])[0].is_contractible() {
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
        contractible.enlarge_in_supercomplex(self, false, quiet);

        contractible
    }

    pub fn cofaces(&self, s: &Simplex) -> Vec<Simplex> {
        // Is there a good heuristic for the capacity to use?
        let mut cb = new_hs::<u32>(1);
        for f in self.facets.iter().filter(|f| s <= f) {
            cb.extend(f.0.iter().filter(|v| !s.contains(v)));
        }

        let mut res = new_vec::<Simplex>(cb.len());
        res.extend(cb.into_iter().map(|v| s.add_vertex(&v)));
        res
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
            facet.0 = facet.0.iter().map(|v| vertex_dict[v]).collect();
        }
    }

    // DMT stuff from Mischaikow, Nanda: Morse Theory for Filtrations...

    pub fn add_face(&mut self, s: Simplex) {
        self.facets.push(s);
        self.maximalify()
    }

    // Optimize this shit
    pub fn cells(&self) -> Vec<HashSet<Rc<Simplex>>> {
        let first_len = self.first_len();
        let cells: Vec<HashSet<Rc<Simplex>>> = vec![new_hs::<Rc<Simplex>>(0); first_len + 2];
        let mut new = SComplex { cells };
        for facet in &self.facets {
            new.insert(Rc::new(facet.clone()));
        }
        for i in (2..=first_len).rev() {
            for s in new.cells[i].clone() {
                for v in &(*s).0 {
                    let mut cell = (*s).0.clone();
                    cell.remove(&v);
                    new.insert(Rc::new(Simplex(cell)));
                }
            }
        }

        new.cells
    }

    // Buggy: note, when something is removed from N, it is also removed from all cbs and del^Ns!!! No indication that it is removed from g however.
    pub fn morse_reduce(
        self
    ) -> HashMap<Rc<Simplex>, HashMap<Rc<Simplex>, i32>> {
        let mut cells = self.cells();
        let mut crits = new_hs::<Rc<Simplex>>(1);
        let mut del = new_hm::<Rc<Simplex>, HashMap<Rc<Simplex>, i32>>(1);
        let mut d = 0usize;
        while !cells.iter().all(|set| set.is_empty()) {

            // make_critical
            let a_crit = 'outer: loop {
                // Similar: figure out how to not clone!
                for a in &cells[d] {
                    crits.insert(a.clone());
                    // update_gradient_chain_crit
                    for zr in self.cofaces(&*a).into_iter().filter(
                        // |f| cells[f.len()].contains(f) || crits.contains(f)
                        |f| true
                    ) {
                        let z = Rc::new(zr);
                        let sgn = z.sgn(&a);
                        if let Some(z_bd) = del.get_mut(&z) {
                            if let Some(val) = z_bd.get_mut(a) {
                                *val += sgn;
                            } else {
                                z_bd.insert(a.clone(), sgn);
                            }
                        } else {
                            let mut z_bd = new_hm::<Rc<Simplex>, i32>(1);
                            z_bd.insert(a.clone(), sgn);
                            del.insert(z, z_bd);
                        }
                    }

                    break 'outer a.clone();
                }
                d += 1;
            };
            cells[d].remove(&a_crit);

            let a_cofaces = self.cofaces(&a_crit);
            let acl = a_cofaces.len();
            let mut queued = new_hs::<Rc<Simplex>>(acl);
            let mut que = VecDeque::<Rc<Simplex>>::with_capacity(acl);
            queued.extend(a_cofaces.iter().map(|f| Rc::new(f.clone())));
            que.extend(a_cofaces.into_iter().map(|f| Rc::new(f)));
            while let Some(s) = que.pop_front() {
                let s_faces = s.faces();
                let mut s_faces_kept = new_vec::<Rc<Simplex>>(s_faces.len());
                for face in s_faces {
                    if let Some(f) = cells[face.len()].get(&face) {
                        s_faces_kept.push(f.clone());
                    }
                }

                if s_faces_kept.is_empty() {
                    for cr in self.cofaces(&*s) {
                        let c = Rc::new(cr);
                        if queued.insert(c.clone()) && cells[c.len()].contains(&c) {
                            que.push_back(c.clone());
                        }
                    }
                } else if s_faces_kept.len() == 1 {
                    let e = s_faces_kept[0].clone();
                    drop(s_faces_kept);

                    // remove_pair
                    // Swap e and s??? double check
                    cells[s.len()].remove(&s);
                    que.extend(self.cofaces(&e).into_iter().filter(
                        |f| queued.insert(Rc::new(f.clone())) && cells[f.len()].contains(f)
                    ).map(|f| Rc::new(f)));

                    if e.len() == d {
                        if let Some(s_bd) = del.get(&s) {
                            let sgn = s.sgn(&e) * (-1);
                            let mut e_bd = new_hm::<Rc<Simplex>, i32>(s_bd.len());
                            for (e, c) in s_bd.iter() {
                                e_bd.insert(s.clone(), c * sgn);
                            }
                            del.insert(e.clone(), e_bd);
                        }

                        // update_gradient_chain_rem
                        // eprintln!["R"];
                        for zr in self.cofaces(&e).into_iter().filter(
                            // |f| cells[f.len()].contains(f) || crits.contains(f)
                            |f| true
                        ) {
                            let z = Rc::new(zr);
                            let sgn = z.sgn(&e);
                            if let Some(e_bd_r) = del.get(&*e) {
                                let e_bd = e_bd_r.clone();
                                if let Some(z_bd) = del.get_mut(&z) {
                                    for t in e_bd.keys() {
                                        let update = sgn * e_bd[t];
                                        if let Some(z_bd_t_val) = z_bd.get_mut(t) {
                                            *z_bd_t_val += update;
                                        } else {
                                            z_bd.insert(t.clone(), update);
                                        }
                                    }
                                } else {
                                    let mut z_bd = new_hm::<Rc<Simplex>, i32>(e_bd.len());
                                    for t in e_bd.keys() {
                                        z_bd.insert(t.clone(), sgn * e_bd[t]);
                                    }
                                    del.insert(z, z_bd);
                                }
                            }
                        }

                    }
                    cells[e.len()].remove(&*e);

                }
            }
        }
        // del.retain(|s, _| crits.contains(s));
        // Can't use filter because we mutate del
        for a in crits {
            if !del.contains_key(&a) {
                del.insert(a.clone(), new_hm::<Rc<Simplex>, i32>(0));
            }
        }

        // It might be better to use some `new_hm`s here for capacity reasons, but it's probably
        // not a high priority. Check that out later.
        // See about moving instead of cloning??? Will very probably need to clone the keys for the
        // second layer `HashMap` unless I come up with something clever... Maybe a Cow.
        // Possibly much much better idea: create dictionaries to label the cells in each dimension
        // with integers. Could be tedious but would probably really really really speed things up.
        // No: `Rc`s are even better: cheap and more descriptive with less lookup overhead
        del
    }

}
