use std::default::Default;
use std::io::{stdin, BufRead};
use std::ops::BitAnd;
use std::collections::BTreeSet;

mod simplex;
use simplex::{Simplex, PrettySimplex};

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

use clap::Parser;

use console::Style;

mod cli;
use cli::Cli;

use indicatif::{ProgressBar, ProgressStyle};

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


#[inline]
fn the_sty() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}]  {msg:<25} [{bar:50}] {pos:>7}/{len}"
    )
    .unwrap()
    .progress_chars("=> ")
}


fn heading_style() -> Style {
    let sty = Style::new().for_stderr().cyan().bold();

    sty
}


#[inline]
fn read_input() -> SimplicialComplex {
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let mut facets = Vec::<Simplex>::new();
    while let Some(line) = lines.next() {
        let vertices = line
            .expect("A complex should have at least one facet.")
            .split(" ")
            .filter(|v| !v.is_empty())
            .map(|n| {
                n.parse()
                    .expect("Vertices should be labeled by natural numbers less than 2^32.")
            })
            .collect::<HashSet<u32>>();
        if !vertices.is_empty() {
            facets.push(Simplex(vertices));
        }
    }

    if facets.is_empty() {
        SimplicialComplex::default()
    } else {
        SimplicialComplex { facets }
    }
}

fn write_sc(sc: &SimplicialComplex, xml: bool) {
    if xml {
        let xml_prefix = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<SimplicialComplexV2 type=\"SCSimplicialComplex\">\n	<SCFacetsEx type=\"SCArray\">[[".to_string();
        let xml_postfix = "]]</SCFacetsEx>\n</SimplicialComplexV2>".to_string();

        let mut facet_strings_vec = new_v::<String>(sc.facets.len());
        for f in &sc.facets {
            let mut string_vec = new_v::<String>(f.len());
            string_vec.extend(f.0.iter().map(u32::to_string));
            facet_strings_vec.push(string_vec.join(","));
        }

        let complex_string = facet_strings_vec.join("],[");

        println!("{}", xml_prefix + &complex_string + &xml_postfix);
    } else {
        let mut facet_vec: Vec<PrettySimplex> =
            sc.facets.iter().map(PrettySimplex::from).collect();
        facet_vec.sort();
        // One more than the greatest vertex label: we should subtract but only if legal
        let mut l = sc.vertex_set().len();
        if l > 0 { l -= 1; }
        // The number of digits in the greatest vertex label
        let d = l.to_string().len();
        for f in facet_vec {
            f.print(d);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let mut sc = read_input();

    let quiet = cli.quiet;

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
            eprintln!["{}", heading_style().apply_to("\nPinching the complex:\n")];
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
    } else {
        if !quiet {
            eprintln!["{}", heading_style().apply_to("\n\nAccreting a contractible subcomplex:\n")];
        }
        let contractible = sc.contractible_subcomplex(quiet);
        write_sc(&sc, xml);
        print!("\n");
        write_sc(&contractible, xml);
    }
}
