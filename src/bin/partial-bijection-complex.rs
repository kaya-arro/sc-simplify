use clap::Parser;
use sc_simplify::io::write_sc;
use sc_simplify::partial_bijection_complex;

#[derive(Parser)]
#[command(version)]
/// Generate the simplicial complex of non-empty partial bijections between two finite sets
pub struct Cli {
    #[clap(num_args = 2)]
    cards: Vec<u8>,
}

fn main() {
    let [a, b] = Cli::parse().cards[0..2] else {
        panic!["There should be exactly two arguments"]
    };
    write_sc(&partial_bijection_complex(a, b));
}
