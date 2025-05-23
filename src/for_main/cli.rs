use crate::Parser;

#[derive(Parser)]
#[command(version)]
/// Read a simplicial complex from stdin and print an equivalent simplified complex or pair.
///
/// Each line of the input is one facet, represented as a space-separated list of vertices.
/// The vertices should be labeled by natural numbers less than 2^32.
///
/// The default behavior prints a pair X, C of simplicial complexes in the same format as the input
/// in which X has the homotopy type of the input and C is a large contractible subcomplex of X.
/// The complexes X and C are delineated by a blank line.
pub struct Cli {
    /// Check that the faces in the input are maximal.
    ///
    /// Depending on other settings used, input including non-maximal faces may cause unexpected
    /// behavior.
    #[arg(short, long, default_value_t = false)]
    pub check_input: bool,

    /// Skip simplification by iterated Čech nerves.
    ///
    /// The Čech nerve of a complex has the same homotopy type as the complex and roughly exchanges
    /// the sets of facets and vertices, except certain "redundant" vertices are naturally removed.
    /// This can also change the dimension of the complex, sometimes substantially.
    ///
    /// Iterating this construction efficiently simplifies a complex when applicable.
    ///
    /// If you know that your input has no "redundant" vertices and that its dimension is less than
    /// the dimension of its nerve, you should skip this operation using this flag.
    #[arg(short = 'N', long, default_value_t = false)]
    pub skip_nerve: bool,

    /// Limit the "pinch" algorithm to this many runs.
    ///
    /// The pinch algorithm identifies edges which can be contracted without altering the
    /// homotopy type and contracts them. Successive runs provide diminishing returns. The program
    /// will always stop pinching once there are no longer any edges that can be contracted.
    ///
    /// It is highly unlikely that increasing this parameter beyond the default value of `2` will
    /// expedite homology calculations.
    ///
    /// Use `0` to disable pinching entirely.
    #[arg(short = 'P', long, default_value_t = 2, value_name = "MAX")]
    pub max_pinch_loops: usize,

    /// Only print the simplified input.
    ///
    /// If this flag is enabled, only one complex, equivalent to the input, will be printed.
    ///
    /// If this flag is not enabled, the tool prints a complex and subcomplex, X, Y, whose pair
    /// homotopy type -- i.e. the homotopy type of the mapping cone of the inclusion of Y into X --
    /// agrees with the homotopy type of the input.
    #[arg(short = 'p', long, default_value_t = false)]
    pub no_pair: bool,

    /// Skip minimizing the output pair by removing shared facets from both.
    ///
    /// If this flag is not enabled, the program will return a pair of complexes homotopy
    /// equivalent as a pair  to the input complex but such that the first complex in the pair
    /// on its own is unlikely to share the homotopy type of the input.
    ///
    /// If this flag is enabled, the program will instead return a simplicial complex that is
    /// homotopy equivalent to the input along with a large contractible subcomplex.
    #[arg(short = 'm', long, default_value_t = false)]
    pub skip_minimize_pair: bool,

    /// Suppress the progress indicators.
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}
