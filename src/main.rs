use std::cmp::{Ordering, Reverse, min};
use std::collections::VecDeque;
use std::default::Default;
use std::fmt::Display;
use std::io::{IsTerminal, stderr};
use std::ops::BitAnd;
use std::time::Duration;

mod simplex;
use simplex::Simplex;

mod simplicial_complex;
use simplicial_complex::{SimplicialComplex, minimize_pair};

use clap::Parser;
mod cli;
use cli::Cli;

use indicatif::ProgressBar;
mod io;
use io::{head_sty, info_sty_str, upd_sty};
use io::{new_pb, new_spnr};
use io::{read_input, sc_info, write_sc};

use rustc_hash::FxBuildHasher;
use rustc_hash::{FxHashMap as HashMap, FxHashSet as HashSet};

fn new_hs<T>(len: usize) -> HashSet<T> {
    HashSet::with_capacity_and_hasher(len, FxBuildHasher::default())
}

fn new_hm<S, T>(len: usize) -> HashMap<S, T> {
    HashMap::with_capacity_and_hasher(len, FxBuildHasher::default())
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

fn to_sorted_vec(set: &HashSet<u32>) -> Vec<u32> {
    let mut vec = to_vec(&set);
    vec.sort_unstable();

    vec
}

fn main() {
    let cli = Cli::parse();

    let mut sc = read_input();

    let quiet = cli.quiet || !stderr().is_terminal();

    let xml = cli.xml;

    if !quiet {
        sc_info(&sc, "The original complex");
    }

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
        if !quiet {
            eprintln!["\n{}", head_sty("Applying Čech nerves:")];
        }
        let nerve_count = sc.reduce(quiet);
        if !quiet {
            eprintln![];
            if nerve_count > 0 {
                eprintln![];
                sc_info(&sc, "After reducing, the complex");
            }
        }
    }

    let mut i = cli.max_pinch_loops;
    if i > 0 {
        if !quiet {
            eprintln!["\n{}", head_sty("Pinching edges:")];
        }
        while i > 0 && sc.pinch(quiet) {
            i -= 1;
            // The pinch algorithm uses an ordering of the vertices for efficiency. Relabeling the
            // vertices helps shake things up and allow further pinches.
            sc.relabel_vertices();
        }
        if !quiet {
            eprintln![];
            if i < cli.max_pinch_loops {
                eprintln![];
                sc_info(&sc, "After pinching, the complex");
            }
        }
    }

    if cli.no_pair {
        write_sc(&sc, xml);
    } else {
        let contractible: SimplicialComplex;
        if sc.height() > 0 {
            if !quiet {
                eprintln!["\n{}", head_sty("Accreting contractible subcomplex:")];
            }

            contractible = sc.contractible_subcomplex(quiet);

            if !quiet {
                eprintln!["\n"];
                if sc.height() > 0 {
                    sc_info(&contractible, "The contractible subcomplex");
                } else {
                    eprintln![
                        "{}",
                        info_sty_str("The empty complex contains no contractible subcomplex.")
                    ];
                }
            }

            let first: SimplicialComplex;
            let second: SimplicialComplex;

            if cli.skip_minimize_pair {
                (first, second) = (sc, contractible);
            } else {
                (first, second) = minimize_pair((sc, contractible));
            }

            write_sc(&first, xml);
            println![];
            write_sc(&second, xml);
        }
    }
}
