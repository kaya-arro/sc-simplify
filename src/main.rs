use std::default::Default;
use std::io::stdin;
use std::ops::BitAnd;
use std::collections::BTreeSet;

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

mod simplex;
use simplex::{Simplex, PrettySimplex};

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

use clap::Parser;

mod cli;
use cli::Cli;

use std::io::BufRead;

#[inline]
fn read_input() -> SimplicialComplex {
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let mut facets: Vec<Simplex> = Vec::new();
    // We ignore empty lines except/unless the first line is empty, in which case it is the empty
    // simplex.
    let mut first = true;
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
        if !vertices.is_empty() || first {
            facets.push(Simplex(vertices));
        }
        first = false;
    }

    SimplicialComplex { facets }
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
        let d = (sc.vertex_set().len() - 1).to_string().len();
        for f in facet_vec {
            f.print(d);
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let xml = cli.xml;
    let mut sc = read_input();

    if cli.skip_nerve {
        if cli.check_input {
            sc = SimplicialComplex::from_check(sc.facets);
        } else {
            sc.facets.select_nth_unstable_by_key(0, Simplex::len);
        }
    } else {
        sc.reduce();
    }

    if !cli.skip_pinch {
        sc.pinch();
    }

    sc.relabel_vertices();

    if cli.no_pair {
        write_sc(&sc, xml);
    } else {
        let contractible = sc.contractible_subcomplex();
        write_sc(&sc, xml);
        print!("\n");
        write_sc(&contractible, xml);
    }
}
