use std::io::{stdin, BufRead};

use indicatif::ProgressStyle;
use console::Style;

use crate::{HashSet, new_vec};
use crate::{SimplicialComplex, SComplex, Simplex, PrettySimplex};
use crate::{Rc, HashMap};

// A template for indicatif progress bars
pub fn the_sty() -> ProgressStyle {
    ProgressStyle::with_template(
        "[{elapsed_precise}]  {msg:<24} [{bar:50}] {pos:>7}/{len:<7} {eta:>5} left"
    )
    .unwrap()
    .progress_chars("=> ")
}


// Text styles for console output
pub fn heading_style() -> Style {
    let sty = Style::new().for_stderr().cyan().bold();

    sty
}

pub fn update_style() -> Style {
    let sty = Style::new().for_stderr().cyan().bright();

    sty
}

pub fn info_style() -> Style {
    let sty = Style::new().for_stderr().white().italic();

    sty
}

pub fn info_number_style() -> Style {
    let sty = Style::new().for_stderr().yellow().bright();

    sty
}


// Print formatted text to the console about the number of vertices and facets of a complex.
pub fn sc_info(sc: &SimplicialComplex, name: &str) {
    eprintln![
        "{} {} {} {} {}",
        info_style().apply_to(format!["{name} contains"]),
        info_number_style().apply_to(format!["{}", sc.vertex_set().len()]),
        info_style().apply_to("vertices and"),
        info_number_style().apply_to(format!["{}", sc.facets.len()]),
        info_style().apply_to("facets"),
    ];
}


pub fn read_input() -> SimplicialComplex {
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

pub fn write_sc(sc: &SimplicialComplex, xml: bool) {
    if xml {
        let xml_prefix = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<SimplicialComplexV2 type=\"SCSimplicialComplex\">\n	<SCFacetsEx type=\"SCArray\">[[".to_string();
        let xml_postfix = "]]</SCFacetsEx>\n</SimplicialComplexV2>".to_string();

        let mut facet_strings_vec = new_vec::<String>(sc.facets.len());
        for f in &sc.facets {
            let mut string_vec = new_vec::<String>(f.len());
            string_vec.extend(f.0.iter().map(u32::to_string));
            facet_strings_vec.push(string_vec.join(","));
        }

        let complex_string = facet_strings_vec.join("],[");

        println!("{}", xml_prefix + &complex_string + &xml_postfix);
    } else {
        let mut facet_vec: Vec<PrettySimplex> = sc.facets.iter().map(
            PrettySimplex::from
        ).collect();
        // Benchmark sorting unstable
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

pub fn write_s_complex(sc: SComplex) {
    let mut pretty_cells = new_vec::<Vec<PrettySimplex>>(sc.len());
    for dim_set in &sc.cells {
        let mut dim_pretty_vec = new_vec::<PrettySimplex>(dim_set.len());
        dim_pretty_vec.extend(dim_set.into_iter().map(|s| PrettySimplex::from(&**s)));
        dim_pretty_vec.sort_unstable();
        pretty_cells.push(dim_pretty_vec);
    }
    // One more than the greatest vertex label: we should subtract but only if legal
    if let Some(n) = sc.vertex_set().iter().max() {
        let mut l = *n;
        if l > 0 { l -= 1; }
        // The number of digits in the greatest vertex label
        let d = l.to_string().len();
        for dim_vec in pretty_cells {
            for pretty_cell in dim_vec {
                pretty_cell.print(d);
            }
        }
    }
}

pub fn write_chain_complex(del: HashMap<Rc<Simplex>, HashMap<Rc<Simplex>, i32>>) {
    let mut output = del.keys().into_iter().map(|s| s.clone()).collect::<Vec<Rc<Simplex>>>();
    output.sort_unstable_by_key(|s| s.len());
    for s in output {
        let s_bd = &del[&s];
        let mut bd_str = format!["∂{} =", s];
        for (t, val) in s_bd {
            let fmtted = format![" {}{}", val, t];
            bd_str.push_str(&fmtted);
        }

        println!["{}", bd_str];
    }
}


