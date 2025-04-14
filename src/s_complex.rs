// use crate::Duration;
// use crate::VecDeque;

use crate::Rc;
// use crate::Default;

// use crate::{ProgressBar, update_style, info_style, info_number_style, the_sty};
use crate::{HashSet, new_hs, new_vec, new_vd};
// use crate::new_hm;

use crate::{Simplex, SimplicialComplex};


fn to_clones<T: Clone>(vec: &Vec<T>) -> impl Iterator<Item = T> + '_ {
    vec.iter().map(|t| t.clone())
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SComplex {
    pub cells: Vec<HashSet<Rc<Simplex>>>
}

impl From<SimplicialComplex> for SComplex {
    // Optimize this shit
    fn from(sc: SimplicialComplex) -> Self {
        let first_len = sc.first_len();
        let cells: Vec<HashSet<Rc<Simplex>>> = vec![new_hs::<Rc<Simplex>>(0); first_len + 2];
        let mut new = Self { cells };
        for facet in sc.facets {
            new.insert(Rc::new(facet));
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

        new
    }
}

impl SComplex {

    pub fn len(&self) -> usize { self.cells.len() }

    fn is_vacant(&self) -> bool { self.cells.iter().all(|ds| ds.is_empty()) }

    // fn contains(&self, s: &Simplex) -> bool { self.cells[s.len()].contains(s) }

    pub fn insert(&mut self, s: Rc<Simplex>) -> bool {
        if s.len() >= self.len() {
            let diff = 2 + s.len() - self.len();
            for _ in 0..diff {
                self.cells.push(new_hs::<Rc<Simplex>>(1));
            }
        }
        (*self).cells[s.len()].insert(s.clone())
    }

    fn remove(&mut self, s: &Simplex) -> bool { (*self).cells[s.len()].remove(s) }

    fn twin(&mut self) -> Self {
        let mut prev = SComplex { cells: new_vec(self.len()) };
        prev.cells.append(&mut self.cells);
        prev
    }

    fn cofaces(&self, s: &Simplex) -> Vec<Rc<Simplex>> {
        if s.len() + 1 == self.len() {
            new_vec(0)
        } else {
            self.cells[s.len() + 1].iter().filter(|c| s < c).map(|c| c.clone()).collect()
        }
    }

    fn faces(&self, s: &Simplex) -> Vec<Rc<Simplex>> {
        let s_faces = s.faces();
        let mut s_faces_kept = new_vec::<Rc<Simplex>>(s_faces.len());
        // We use this slightly verbose method to ensure we only have one Rc<Simplex> per face
        for face in s_faces {
            if let Some(f) = self.cells[face.len()].get(&face) {
                s_faces_kept.push(f.clone());
            }
        }

        s_faces_kept
    }

    pub fn vertex_set(&self) -> HashSet<u32> {
        let mut vertex_set = new_hs::<u32>(self.cells[0].len());
        for dim_set in &self.cells {
            for cell in dim_set {
                vertex_set.extend(&cell.0);
            }
        }

        vertex_set
    }

    // See Mischaikow, Nanda - Morse Theory for Filtrations... §5.1
    pub fn coreduce(&mut self) -> bool {
        let mut result = false;
        let mut prev = self.twin();
        let mut d = 0usize;

        while !prev.is_vacant() {
            let a = 'outer: loop {
                for a in &prev.cells[d] {
                    self.insert(a.clone());
                    break 'outer a.clone();
                }
                d += 1;
            };
            prev.remove(&a);

            let a_cofaces = prev.cofaces(&a);
            let mut queued = new_hs::<Rc<Simplex>>(a_cofaces.len());
            let mut que = new_vd::<Rc<Simplex>>(a_cofaces.len());
            queued.extend(to_clones(&a_cofaces));
            que.extend(a_cofaces);

            while let Some(s) = que.pop_front() {
                let s_faces = prev.faces(&s);
                if s_faces.is_empty() {
                    for c in prev.cofaces(&s) {
                        if queued.insert(c.clone()) {
                            que.push_back(c.clone());
                        }
                    }
                } else if s_faces.len() == 1 {
                    result = true;
                    let e = s_faces[0].clone();
                    prev.remove(&e);
                    que.extend(prev.cofaces(&s).into_iter().filter(
                        |f| queued.insert(f.clone())
                    ).map(|f| f.clone()));

                    prev.remove(&*s);
                }
            }
        }

        result
    }

    pub fn reduce(&mut self) -> bool {
        let md = self.len();
        if md == 0 { return false; }
        let mut result = false;
        let mut prev = self.twin();
        let mut d = md - 1;

        while !prev.is_vacant() {
            let a = 'outer: loop {
                for a in &prev.cells[d] {
                    self.insert(a.clone());
                    break 'outer a.clone();
                }
                d -= 1;
            };
            prev.remove(&a);

            let a_faces = prev.faces(&a);
            let mut queued = new_hs::<Rc<Simplex>>(a_faces.len());
            let mut que = new_vd::<Rc<Simplex>>(a_faces.len());
            queued.extend(to_clones(&a_faces));
            que.extend(a_faces);

            while let Some(s) = que.pop_front() {
                let s_cofaces = prev.cofaces(&s);
                if s_cofaces.is_empty() {
                    for c in prev.faces(&s) {
                        if queued.insert(c.clone()) {
                            que.push_back(c.clone());
                        }
                    }
                } else if s_cofaces.len() == 1 {
                    result = true;
                    let e = s_cofaces[0].clone();
                    prev.remove(&e);
                    que.extend(prev.faces(&s).into_iter().filter(
                        |f| queued.insert(f.clone())
                    ).map(|f| f.clone()));

                    prev.remove(&*s);
                }
            }
        }

        result
    }

    pub fn to_pair(self) -> (SimplicialComplex, SimplicialComplex) {
        let mut cells = new_hs::<Simplex>(0);
        for dim_set in self.cells {
            cells.extend(dim_set.into_iter().map(|f| (*f).clone()));
        }
        let main = SimplicialComplex::from_check(cells.iter().map(|f| f.clone()).collect());
        let mut subcells = new_hs::<Simplex>(0);
        for cell in &cells {
            for face in cell.faces() {
                add_missing(&face, &cells, &mut subcells);
            }
        }
        let sub = SimplicialComplex::from_check(subcells.into_iter().collect());

        (main, sub)
    }

    // pub fn relabel_vertices(&mut self) {
    //     if !self.is_vacant() {
    //         let prev = self.twin();
    //         let mut vertex_set = new_hs::<u32>(prev.cells[0].len());
    //         for dim_set in &prev.cells {
    //             for cell in dim_set {
    //                 vertex_set.extend(&cell.0);
    //             }
    //         }
    //         let mut vertex_dict = new_hm::<u32, u32>(vertex_set.len());
    //         let mut n = 0u32;
    //         for v in vertex_set {
    //             vertex_dict.insert(v, n);
    //             n += 1;
    //         }
    //         for dim_set in prev.cells {
    //             // Fix all this clone bullshit
    //             for mut cell in dim_set.clone() {
    //                 let mcell = Rc::make_mut(&mut cell);
    //                 *mcell = Simplex(
    //                     (*mcell).0.iter().map(|v| vertex_dict[v]).collect()
    //                 );
    //                 self.insert(Rc::new(mcell.clone()));
    //             }
    //         }
    //     }
    // }

}

fn add_missing(s: &Simplex, set: &HashSet<Simplex>, missing: &mut HashSet<Simplex>) {
    for face in s.faces() {
        if !set.contains(&face) {
            missing.insert(face);
        } else {
            for faceface in face.faces() {
                add_missing(&faceface, &set, missing);
            }
        }
    }
}
