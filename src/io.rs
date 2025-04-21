use std::io::{BufRead, stdin};
use std::sync::LazyLock;

use console::{Style, StyledObject};
use indicatif::{ProgressBar, ProgressStyle};

use crate::Display;
use crate::Duration;
use crate::Ordering;
use crate::{HashSet, new_vec};
use crate::{Simplex, SimplicialComplex};

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

// Text styles for console output
const HEAD_STY: LazyLock<Style> = LazyLock::new(|| Style::new().for_stderr().cyan().bold());

const UPD_STY: Style = Style::new().for_stderr().cyan().bright();

const INFO_STR_STY: LazyLock<Style> = LazyLock::new(|| Style::new().for_stderr().white().italic());

const INFO_NUM_STY: Style = Style::new().for_stderr().yellow().bright();

pub fn head_sty<S: Display>(text: S) -> StyledObject<String> {
    HEAD_STY.apply_to(text.to_string())
}

pub fn upd_sty<S: Display>(text: S) -> String {
    format!["{}", UPD_STY.apply_to(text.to_string())]
}

pub fn info_sty_str<S: Display>(text: S) -> StyledObject<String> {
    INFO_STR_STY.apply_to(text.to_string())
}

pub fn info_sty_num<S: Display>(n: S) -> StyledObject<String> {
    INFO_NUM_STY.apply_to(n.to_string())
}

// Print formatted text to the console about the number of vertices and facets of a complex.
pub fn sc_info(sc: &SimplicialComplex, name: &str) {
    eprintln![
        "{} {} {} {} {}",
        info_sty_str(format!["{name} contains"]),
        info_sty_num(sc.vertex_set().len()),
        info_sty_str("vertices and"),
        info_sty_num(sc.facet_count()),
        info_sty_str("facets"),
    ];
}

// Refactor this to handle the check
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
            facets.push(Simplex::from(vertices));
        }
    }

    if facets.is_empty() {
        SimplicialComplex::default()
    } else {
        SimplicialComplex { facets }
    }
}

// pub fn read_pair() -> (SimplicialComplex, SimplicialComplex) {
//     let stdin = stdin();
//     let mut lines = stdin.lock().lines();
//
//     let mut facets = new_vec::<Simplex>(1);
//     while let Some(line) = lines.next() {
//         let vertices = line
//         .expect("A complex should have at least one facet.")
//         .split(" ")
//         .filter(|v| !v.is_empty())
//         .map(|n| {
//             n.parse()
//             .expect("Vertices should be labeled by natural numbers less than 2^32.")
//         })
//         .collect::<HashSet<u32>>();
//         if !vertices.is_empty() {
//             facets.push(Simplex::from(vertices));
//         } else {
//             break
//         }
//     }
//
//     let mut sec_facets =new_vec::<Simplex>(1);
//     while let Some(line) = lines.next() {
//             let vertices = line
//             .expect("A complex should have at least one facet.")
//             .split(" ")
//             .filter(|v| !v.is_empty())
//             .map(|n| {
//                 n.parse()
//                 .expect("Vertices should be labeled by natural numbers less than 2^32.")
//             })
//             .collect::<HashSet<u32>>();
//         if !vertices.is_empty() {
//             sec_facets.push(Simplex::from(vertices));
//         } else {
//             break
//         }
//     }
//
//     (SimplicialComplex { facets }, SimplicialComplex { facets: sec_facets })
// }

pub fn write_sc(sc: &SimplicialComplex, xml: bool) {
    if xml {
        let xml_prefix = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<SimplicialComplexV2 type=\"SCSimplicialComplex\">\n	<SCFacetsEx type=\"SCArray\">[[".to_string();
        let xml_postfix = "]]</SCFacetsEx>\n</SimplicialComplexV2>".to_string();

        let mut facet_strings_vec = new_vec::<String>(sc.facet_count());
        for f in &sc.facets {
            let mut string_vec = new_vec::<String>(f.len());
            string_vec.extend(f.vertices.iter().map(u32::to_string));
            facet_strings_vec.push(string_vec.join(","));
        }

        let complex_string = facet_strings_vec.join("],[");

        println!("{}", xml_prefix + &complex_string + &xml_postfix);
    } else {
        let mut facets_vec: Vec<Vec<u32>> = sc
            .facets
            .iter()
            .map(|s| {
                let mut vec = s.vertices.iter().copied().collect::<Vec<u32>>();
                vec.sort_unstable_by(|a, b| b.cmp(a));

                vec
            })
            .collect();
        // Benchmark sorting unstable instead
        // `sort_by_key` should work here with Reverse, but the sad facet is that it doesn't. It
        // gives lifetime issues.
        facets_vec.sort_by(|a, b| {
            if a.len() < b.len() {
                Ordering::Greater
            } else if a.len() > b.len() {
                Ordering::Less
            } else if a < b {
                Ordering::Greater
            } else {
                Ordering::Less
            }
        });
        // facets_vec.sort_by_key(|f| (Reverse(f.len()), Reverse(f)));
        // One more than the greatest vertex label: we should subtract but only if legal
        let mut l = facets_vec[0][0];
        if l > 0 {
            l -= 1;
        }
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
}
