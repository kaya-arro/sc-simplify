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
use io::{the_sty, info_number_style, update_style, info_style, heading_style, sc_info, read_input, write_sc};

use rustc_hash::FxBuildHasher;
use rustc_hash::FxHashSet as HashSet;

fn the_hasher() -> FxBuildHasher {
    FxBuildHasher::default()
}


fn new_hs<T>(len: usize) -> HashSet<T> {
    HashSet::with_capacity_and_hasher(len, the_hasher())
}


fn new_v<T>(len: usize) -> Vec<T> {
    Vec::<T>::with_capacity(len)
}


fn to_v<T: Copy>(s: &HashSet<T>) -> Vec<T> {
    let mut v = new_v::<T>(s.len());
    v.extend(s);
    v
}


fn main() {
    let cli = Cli::parse();
    let quiet = cli.quiet;
    let xml = cli.xml;

    let mut sc = read_input();

    if !quiet { sc_info(&sc, "The original complex".to_string()); }

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
        let nerve_count = sc.reduce(quiet);
        if !quiet && nerve_count > 0 {
            eprint![
                "{} {} {}",
                info_style().apply_to("After reducing with Čech nerves"),
                info_number_style().apply_to(format!["{}", nerve_count]),
                info_style().apply_to("times, "),
            ];
            sc_info(&sc, "the complex".to_string());
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
            if !cli.skip_collapse {
                if !quiet { eprintln!["\n\n{}", heading_style().apply_to("Collapsing faces:")]; }
                while sc.collapse(quiet) { }
                if !quiet { eprintln!["\n"]; }
            } else {
                eprintln!["\n"];
            }
        }
    }

    if cli.no_pair {
        write_sc(&sc, xml);
        if !quiet { sc_info(&sc, "The simplified complex".to_string()); }
    } else {
        if !quiet {
            eprintln!["{}", heading_style().apply_to("Accreting subcomplex:")];
        }

        let contractible = sc.contractible_subcomplex(quiet);

        write_sc(&sc, xml);
        println![];
        write_sc(&contractible, xml);

        if !quiet {
            eprintln!["\n"];
            sc_info(&sc, "The simplified complex".to_string());
            sc_info(&contractible, "The contractible subcomplex".to_string());
        }
    }

}
