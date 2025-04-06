use std::default::Default;
use std::ops::BitAnd;
use std::collections::BTreeSet;

mod simplex;
use simplex::{Simplex, PrettySimplex};

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

use clap::Parser;

use indicatif::ProgressBar;

mod cli;
use cli::Cli;

mod io;
use io::{the_sty, update_style, heading_style, sc_info, read_input, write_sc};

use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashSet as HashSet;

fn the_hasher() -> FxBuildHasher {
    FxBuildHasher::default()
}

#[inline]
fn new_hs<T>(len: usize) -> HashSet<T> {
    HashSet::with_capacity_and_hasher(len, the_hasher())
}

#[inline]
fn new_v<T>(len: usize) -> Vec<T> {
    Vec::<T>::with_capacity(len)
}

#[inline]
fn to_v<T: Copy>(s: &HashSet<T>) -> Vec<T> {
    let mut v = new_v::<T>(s.len());
    v.extend(s);
    v
}


fn main() {
    let cli = Cli::parse();
    let mut sc = read_input();

    let quiet = cli.quiet;

    if !quiet { sc_info(&sc, "The original complex"); }

    if cli.skip_nerve {
        if cli.check_input {
            sc = SimplicialComplex::from_check(sc.facets);
        } else {
            // Even if we don't check the input, we check the first facet because it's cheap and
            // important.
            sc.facets.select_nth_unstable_by_key(0, Simplex::len);
        }
    } else {
        // There is no need to perform checks if we reduce.
        sc.reduce();
    }

    let mut i = cli.max_pinch_loops;
    if i > 0 {
        if !quiet {
            eprintln!["{}", heading_style().apply_to("\nPinching complex:")];
        }
        while i > 0 && sc.pinch(quiet) {
            i -= 1;
            sc.relabel_vertices();
        }
    } else {
        sc.relabel_vertices();
    }

    let xml = cli.xml;
    if cli.no_pair {
        write_sc(&sc, xml);
        if !quiet { sc_info(&sc, "The simplified complex"); }
    } else {
        if !quiet {
            eprintln!["{}", heading_style().apply_to("\n\nAccreting subcomplex:")];
        }

        let contractible = sc.contractible_subcomplex(quiet);

        write_sc(&sc, xml);
        print!["\n"];
        write_sc(&contractible, xml);

        if !quiet {
            eprintln!["\n"];
            sc_info(&sc, "The simplified complex");
            sc_info(&contractible, "The contractible subcomplex");
        }
    }
}
