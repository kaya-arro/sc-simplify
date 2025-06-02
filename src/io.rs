use std::io::{BufRead, stdin};
use std::time::Duration;

use crate::Vertex;
use crate::style::{info_sty_num, info_sty_str};
use crate::{Face, SimplicialComplex};

use crate::ProgressBar;
use indicatif::ProgressStyle;

pub fn new_pb(n: usize) -> ProgressBar {
    let pb = ProgressBar::new(n as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}]  {msg:<24} [{bar:50}] {pos:>7}/{len:<7} {eta:>5} left",
        )
        .expect("The progress indicator template is invalid")
        .progress_chars("=> "),
    );

    pb
}

pub fn new_spnr() -> ProgressBar {
    let spnr = ProgressBar::new_spinner();
    spnr.enable_steady_tick(Duration::from_millis(100));
    spnr.set_style(
        ProgressStyle::with_template("[{elapsed_precise}]  {msg:<24} {spinner}")
            .expect("The progress indicator template is invalid"),
    );

    spnr
}

// Print formatted text to the console about the number of vertices and facets of a complex.
// pub fn sc_info<Point: Vertex>(sc: &SimplicialComplex<Point>, name: &str) {
pub fn sc_info<Point: Vertex>(sc: &SimplicialComplex<Point>, name: &str) {
    eprintln![
        "{} {} {} {} {}",
        info_sty_str(format!["{name} contains"]),
        info_sty_num(sc.vertex_set().len()),
        info_sty_str("vertices and"),
        info_sty_num(sc.len()),
        info_sty_str("facets"),
    ];
}

pub enum SC {
    Small(SimplicialComplex<u16>),
    Large(SimplicialComplex<u32>),
}

fn ambi_sc_info(asc: &SC, s: &str) {
    if let SC::Small(sc) = asc {
        sc_info(sc, s);
    } else if let SC::Large(sc) = asc {
        sc_info(sc, s);
    }
}

// Refactor this to handle the check
pub fn read_input(quiet: bool) -> SC {
    let stdin = stdin();
    let mut lines = stdin.lock().lines();
    let mut max: u32 = 0;
    let mut facets = Vec::<Face<u32>>::new();
    while let Some(line) = lines.next() {
        let facet = line
            .expect("A complex should have at least one facet.")
            .split(" ")
            .filter(|v| !v.is_empty())
            .map(|n| {
                let m = n
                    .parse()
                    .expect("Pointices should be labeled by natural numbers less than 2^32.");
                max = max.max(m);

                m
            })
            .collect::<Face<u32>>();
        if !facet.is_empty() {
            facets.push(facet);
        }
    }

    drop(stdin);

    max = max.max(
        facets
            .len()
            .try_into()
            .expect("There should be fewer than 2^32 facets"),
    );

    let sc: SC = if facets.is_empty() {
        SC::Small(SimplicialComplex::<u16>::default())
    } else if max > u16::MAX.into() {
        SC::Large(SimplicialComplex::<u32>::from(facets))
    } else {
        SC::Small(SimplicialComplex::<u16>::from(
            facets
                .into_iter()
                .map(|s| {
                    s.into_iter()
                        .map(|v| v.try_into().unwrap())
                        .collect::<Face<u16>>()
                })
                .collect::<Vec<Face<u16>>>(),
        ))
    };

    if !quiet {
        ambi_sc_info(&sc, "The original complex");
    }

    sc
}

pub fn write_sc<Point: Vertex>(sc: &SimplicialComplex<Point>) {
    let mut facets_vec: Vec<Vec<Point>> = sc.into_iter().map(|s| s.to_vec()).collect();
    // Benchmark sorting unstable instead
    // `sort_by_key` should work here with Reverse, but the sad facet is that it doesn't. It
    // gives lifetime issues.
    facets_vec.sort_by(|a, b| {
        if a.len() != b.len() {
            b.len().cmp(&a.len())
        } else {
            b.cmp(a)
        }
    });

    let l = facets_vec
        .first()
        .expect("Even empty complexes should have one facet")
        .first()
        .copied()
        .unwrap_or(Point::zero());
    // The number of digits in the greatest vertex label
    let d = l.to_string().len();
    for f in facets_vec {
        println![
            "{}",
            f.into_iter()
                .map(|v| format!["{:>d$}", v])
                .collect::<Vec<String>>()
                .join(" "),
        ];
    }
}
