use std::ops::BitAnd;
use std::default::Default;
use std::io::stdin;

use rustc_hash::FxBuildHasher;
use rustc_hash::{FxHashSet as HashSet};
fn the_hasher() -> FxBuildHasher { FxBuildHasher::default() }

mod simplex;
use simplex::Simplex;

mod simplicial_complex;
use simplicial_complex::SimplicialComplex;

use clap::Parser;
use cute::c;

mod cli;
use cli::Cli;

use std::io::BufRead;

fn read_input() -> SimplicialComplex {
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let mut facets: Vec<Simplex> = Vec::new();
    let first = true;
    while let Some(line) = lines.next() {
        let vertices = line
        .expect("A complex should have at least one facet.")
        .split(" ")
        .filter(|n| n.len() > 0)
        .map(|n| n.parse().expect("Vertices should be labeled by natural numbers less than 2^32."))
        .collect::<HashSet<u32>>();
        if vertices.len() > 0 || first {
            facets.push(Simplex(vertices));
        }
    }

    SimplicialComplex { facets }
}

fn write_sc(sc: &SimplicialComplex, xml: bool) {
    if xml {
        let xml_prefix = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<SimplicialComplexV2 type=\"SCSimplicialComplex\">\n	<SCFacetsEx type=\"SCArray\">[[".to_string();
        let xml_postfix = "]]</SCFacetsEx>\n</SimplicialComplexV2>".to_string();
        let facet_strings = c![
            c![v.to_string(), for v in &f.0].join(","), for f in &sc.facets
        ].join("],[");
        println!("{}", xml_prefix + &facet_strings + &xml_postfix);
    } else {
        for facet in &sc.facets {
            println!("{}", facet.to_string());
        }
    }
}

fn main() {
    let cli = Cli::parse();
    let xml = cli.xml;
    let mut sc = read_input();

    if cli.check_input {
        sc = SimplicialComplex::from_check(sc.facets);
    }

    if !cli.skip_nerve {
        sc.reduce();
    }

    let mut n = 0;
    while n < cli.max_pinch_loops && sc.pinch() {
        n += 1;
        if cli.write_each_pinch {
            write_sc(&sc, xml);
        }
    }

    sc.relabel_vertices();

    if cli.no_pair {
        write_sc(&sc, xml);
    } else if cli.minimize_pair {
        let (remainder, boundary) = sc.minimal_pair();
        write_sc(&remainder, xml);
        print!("\n");
        write_sc(&boundary, xml);
    } else {
        let contractible = sc.contractible_subcomplex();
        write_sc(&sc, xml);
        print!("\n");
        write_sc(&contractible, xml);
    }
}
