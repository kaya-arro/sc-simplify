use std::default::Default;
use std::ops::BitAnd;
use std::collections::BTreeSet;
use std::io::{stderr, IsTerminal};
use std::collections::VecDeque;
use std::cmp::min;
use std::time::Duration;

mod simplex;
use simplex::{Simplex, PrettySimplex};

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

use clap::Parser;
mod cli;
use cli::Cli;

use indicatif::ProgressBar;
mod io;
use io::the_sty;
use io::{heading_style, update_style, info_style, info_number_style};
use io::{sc_info, read_input, write_sc};

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


fn main() {
    let cli = Cli::parse();

    let mut sc = read_input();

    let quiet = cli.quiet || !stderr().is_terminal();
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
        let nerve_count = sc.reduce(quiet);
        if !quiet && nerve_count > 0 {
            eprint![
                "{} {} {}",
                info_style().apply_to("After reducing with Čech nerves"),
                info_number_style().apply_to(format!["{}", nerve_count]),
                info_style().apply_to("times, "),
            ];
            sc_info(&sc, "the complex");
        }
    }

    if !quiet { eprintln![]; }

    if cli.thorough {
        let mut repeat = true;
        while repeat {
            repeat = false;

            if !quiet { eprintln!["{}", heading_style().apply_to("Pinching edges:")]; }
            while sc.pinch(quiet) {
                repeat = true;
                sc.relabel_vertices();
            }

            sc = sc.nerve();
            if !quiet {
                eprintln!["\n"];
                eprintln!["{}", heading_style().apply_to("Pinching edges of Čech nerve:")];
            }
            while sc.pinch(quiet) {
                repeat = true;
                sc.relabel_vertices();
            }
            if !quiet { eprintln!["\n"]; }

            sc = sc.nerve();
        }

        sc.reduce(true);
        if !quiet { eprintln!["{}", heading_style().apply_to("Collapsing faces:")]; }
        while sc.collapse(quiet) { }
        if !quiet { eprintln!["\n"]; }
    } else {
        let mut i = cli.max_pinch_loops;
        if i > 0 {
            if !quiet { eprintln!["{}", heading_style().apply_to("Pinching edges:")]; }
            while i > 0 && sc.pinch(quiet) {
                i -= 1;
                // The pinch algorithm uses an ordering of the vertices for efficiency. Relabeling the
                // vertices helps shake things up and allow further pinches.
                sc.relabel_vertices();
            }
            if !quiet { eprintln!["\n"] }
        }
        i = cli.max_collapse_loops;
        if i > 0 {
            if !quiet { eprintln!["{}", heading_style().apply_to("Collapsing faces:")]; }
            while i > 0 && sc.collapse(quiet) { i -= 1; }
            if !quiet { eprintln!["\n"] }
        }
    }

    if cli.no_pair {
        write_sc(&sc, xml);
        if !quiet { sc_info(&sc, "The simplified complex"); }
    } else {
        write_sc(&sc, xml);

        let mut contractible = SimplicialComplex::default();
        if sc.first_len() > 0 {
            if !quiet {
                eprintln!["{}", heading_style().apply_to("Accreting subcomplex:")];
            }

            contractible = sc.contractible_subcomplex(quiet);

            println![];
            write_sc(&contractible, xml);
        }

        if !quiet {
            eprintln!["\n"];
            sc_info(&sc, "The simplified complex");
            if sc.first_len() > 0 {
                sc_info(&contractible, "The contractible subcomplex");
            } else {
                eprintln!["{}", info_style().apply_to("The empty complex contains no contractible subcomplex.")]
            }
        }
    }
}
