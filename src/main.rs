#![allow(unused)]

use std::default::Default;
use std::ops::BitAnd;
use std::collections::BTreeSet;
use std::io::{stderr, IsTerminal};
use std::collections::VecDeque;
use std::rc::Rc;
use std::cmp::min;
use std::fmt::Display;

mod simplex;
use simplex::{Simplex, PrettySimplex};

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

mod s_complex;
use s_complex::SComplex;

use clap::Parser;
mod cli;
use cli::Cli;

use indicatif::ProgressBar;
mod io;
use io::{new_pb, new_spnr};
use io::{head_sty, upd_sty, info_sty_str, info_sty_num};
use io::{sc_info, read_input, write_sc, write_s_complex, write_chain_complex};

use rustc_hash::FxBuildHasher;
use rustc_hash::{FxHashSet as HashSet, FxHashMap as HashMap};

fn the_hasher() -> FxBuildHasher {
    FxBuildHasher::default()
}

fn new_hs<T>(len: usize) -> HashSet<T> {
    HashSet::with_capacity_and_hasher(len, the_hasher())
}

fn new_hm<S, T>(len: usize) -> HashMap<S, T> {
    HashMap::with_capacity_and_hasher(len, the_hasher())
}


fn new_vec<T>(len: usize) -> Vec<T> {
    Vec::<T>::with_capacity(len)
}

fn new_vd<T>(len: usize) -> VecDeque<T> {
    VecDeque::<T>::with_capacity(len)
}

fn to_vec<T: Copy>(s: &HashSet<T>) -> Vec<T> {
    let mut v = new_vec::<T>(s.len());
    v.extend(s);

    v
}

fn to_rev_sorted_vec(set: &HashSet<u32>) -> Vec<u32> {
    let mut vec = to_vec(&set);
    vec.sort_unstable_by(|a, b| b.cmp(a));

    vec
}

fn to_sorted_vec(set: &HashSet<u32>) -> Vec<u32> {
    let mut vec = to_vec(&set);
    vec.sort_unstable();

    vec
}


fn main() {
    let cli = Cli::parse();

    let mut sc = read_input();

    let quiet = cli.quiet || !stderr().is_terminal();

    if cli.morse_complex {
        // let mut s_complex = SComplex::from(sc);
        // let mut repeat = true;
        // while repeat {
        //     repeat = false;
        //     if s_complex.coreduce() {
        //         repeat = true;
        //     }
        //     if s_complex.reduce() {
        //         repeat = true;
        //     }
        // }
        // // s_complex.relabel_vertices();
        // write_s_complex(s_complex.clone());
        // // println![];
        // // let (main, sub) = s_complex.to_pair();
        // // write_sc(&main, false);
        // // println![];
        // // write_sc(&sub, false);

        write_chain_complex(sc.morse_reduce());
    } else {
        let xml = cli.xml;

        if !quiet { sc_info(&sc, "The original complex"); }

        if cli.skip_nerve {
            // Check if taking nerves is actually faster than checking this way
            if cli.check_input {
                sc = SimplicialComplex::from_check(sc.facets);
            } else {
                // Even if we don't check the input, we check the first facet because it's cheap and
                // important.
                sc.facets.select_nth_unstable_by_key(0, Simplex::len);
            }
        } else {
            // There is no need to perform checks if we reduce.
            if !quiet { eprintln!["\n{}", head_sty("Applying Čech nerves:")]; }
            let nerve_count = sc.reduce(quiet);
            if !quiet && nerve_count > 0 {
                eprintln!["\n"];
                sc_info(&sc, "After reducing, the complex");
            }
        }

        if !quiet { eprintln![]; }

        if cli.thorough {
            let mut repeat = true;
            while repeat {
                repeat = false;

                if !quiet { eprintln!["{}", head_sty("Pinching edges:")]; }
                while sc.pinch(quiet) {
                    repeat = true;
                    sc.relabel_vertices();
                }

                sc = sc.nerve();
                if !quiet {
                    eprintln!["\n"];
                    eprintln!["{}", head_sty("Pinching edges of Čech nerve:")];
                }
                while sc.pinch(quiet) {
                    repeat = true;
                    sc.relabel_vertices();
                }
                if !quiet { eprintln!["\n"]; }

                sc = sc.nerve();
            }

            sc.reduce(true);
            if !quiet { eprintln!["{}", head_sty("Collapsing faces:")]; }
            while sc.collapse(quiet) { }
            if !quiet { eprintln!["\n"]; }
        } else {
            let mut i = cli.max_pinch_loops;
            if i > 0 {
                if !quiet { eprintln!["{}", head_sty("Pinching edges:")]; }
                while i > 0 && sc.pinch(quiet) {
                    i -= 1;
                    // The pinch algorithm uses an ordering of the vertices for efficiency. Relabeling the
                    // vertices helps shake things up and allow further pinches.
                    sc.relabel_vertices();
                }
                if !quiet && i < cli.max_pinch_loops {
                    eprintln!["\n"];
                    sc_info(&sc, "After pinching, the complex");
                }
            }
            i = cli.max_collapse_loops;
            if i > 0 {
                if !quiet { eprintln!["\n{}", head_sty("Collapsing faces:")]; }
                while i > 0 && sc.collapse(quiet) { i -= 1; }
                if !quiet && i < cli.max_collapse_loops {
                    eprintln!["\n"];
                    sc_info(&sc, "After collapsing, the complex");
                }
            }
        }

        if cli.no_pair {
            write_sc(&sc, xml);
        } else {
            write_sc(&sc, xml);

            let mut contractible = SimplicialComplex::default();
            if sc.first_len() > 0 {
                if !quiet { eprintln!["\n{}", head_sty("Accreting subcomplex:")]; }

                contractible = sc.contractible_subcomplex(quiet);

                println![];
                write_sc(&contractible, xml);
            }

            if !quiet {
                eprintln!["\n"];
                if sc.first_len() > 0 {
                    sc_info(&contractible, "The contractible subcomplex");
                } else {
                    eprintln!["{}", info_sty_str("The empty complex contains no contractible subcomplex.")]
                }
            }
        }
    }
}
