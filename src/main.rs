use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use clap::Parser;
use ctrlc;

use sc_simplify::io::{SC, read_input, sc_info, write_sc};
use sc_simplify::{SimplicialComplex, Vertex};

mod for_main;
use for_main::{Cli, head_sty, info_sty_str};

fn pair_write<Point: Vertex>(sc: &SimplicialComplex<Point>, bnd: &SimplicialComplex<Point>) {
    write_sc(sc);
    println![];
    write_sc(bnd);
}

fn simplify<Point: Vertex>(mut sc: SimplicialComplex<Point>, cli: Cli) {
    let interrupted = Arc::new(AtomicBool::new(false));
    let intrpt = interrupted.clone();

    ctrlc::set_handler(move || {
        intrpt.store(true, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    let quiet = cli.quiet;

    if cli.skip_nerve || interrupted.load(Ordering::Relaxed) {
        // Check if taking nerves is actually faster than checking this way
        if cli.check_input {
            sc.maximalify();
        }
    } else {
        // There is no need to perform checks if we reduce.
        if !quiet {
            eprintln!["\n{}", head_sty("Applying Čech nerves:")];
        }
        let nerve_count = sc.nerve_reduce(quiet);
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
        while i > 0
            && !interrupted.load(Ordering::Relaxed)
            && sc.pinch(Some(interrupted.clone()), quiet)
        {
            i -= 1;
        }
        if !quiet {
            eprintln![];
            if i < cli.max_pinch_loops {
                eprintln![];
                sc_info(&sc, "After pinching, the complex");
            }
        }
    }

    if cli.no_pair || interrupted.load(Ordering::Relaxed) {
        write_sc(&sc);
    } else {
        if sc.height() > 0 {
            if !quiet {
                eprintln!["\n{}", head_sty("Accreting contractible subcomplex:")];
            }

            let contractible = sc.contractible_subcomplex(quiet);

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

            if cli.skip_minimize_pair {
                pair_write(&sc, &contractible);
            } else {
                if !quiet {
                    eprintln!["\n{}", head_sty("Minimizing pair:")];
                }
                let bnd = sc.minimize_pair(contractible);
                if !quiet {
                    eprintln!["\n"];
                    sc_info(&sc, "After minimizing, the complex");
                    sc_info(&bnd, "The subcomplex");
                }
                pair_write(&sc, &bnd);
            }
        }
    }
}

fn main() {
    let cli = Cli::parse();
    match read_input(cli.quiet) {
        SC::Small(sc) => simplify(sc, cli),
        SC::Large(sc) => simplify(sc, cli),
    }
}
