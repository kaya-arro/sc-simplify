use crate::Parser;

#[derive(Parser)]
#[command(version)]
/// Read a non-empty complex from stdin and print an equivalent simplified complex or pair.
///
/// Each line of the input is one facet, represented as a space-separated list of vertices.
/// The vertices should be labeled by natural numbers less than 2^32.
///
/// The default behavior prints a pair X, C of simplicial complexes in the same format as the input
/// in which X has the homotopy type of the input and C is a large contractible subcomplex of X.
/// The complexes X and C are delineated by a blank line.
pub struct Cli {
    /// Check that the faces in the input are all maximal.
    ///
    /// Depending on other settings used, input including non-maximal faces may yield unexpected
    /// results unless this flag is enabled.
    #[arg(short, long, default_value_t = false)]
    pub check_input: bool,

    /// Skip taking iterated Čech nerves.
    ///
    /// The Čech nerve of a complex has the same homotopy type as the complex and roughly exchanges
    /// the sets of facets and vertices, except certain "redundant" vertices are naturally removed.
    /// This can also change the dimension of the complex, sometimes substantially.
    ///
    /// Iterating this construction efficiently simplifies a complex when applicable.
    ///
    /// If you know that your input has no "redundant" vertices and that its dimension is less than
    /// the dimension of its nerve, you should skip this operation using this flag.
    #[arg(short, long, default_value_t = false)]
    pub skip_nerve: bool,

    /// Limit the number of pinch loops. Set to 0 to skip pinching entirely.
    ///
    /// The pinch algorithm identifies edges which can be contracted without altering the
    /// homotopy type and contracts them.
    ///
    /// The tool will never continue pinching once no edges can be contracted. Nevertheless, the
    /// efficiency and utility of the algorithm diminish quickly with each iteration, so large
    /// values for this setting are discouraged.
    #[arg(short = 'p', long, value_name = "MAX", default_value_t = 2)]
    pub max_pinch_loops: usize,

    /// Only print the simplified input.
    ///
    /// If this flag is enabled, only one complex, equivalent to the input, will be printed.
    ///
    /// If this flag is not enabled, the tool prints a complex and subcomplex, X, Y, whose pair
    /// homotopy type -- i.e. the homotopy type of the mapping cone of the inclusion of Y into X --
    /// agrees with the homotopy type of the input.
    #[arg(short, long, default_value_t = false)]
    pub no_pair: bool,

    /// Minimize the output pair by removing shared facets from both.
    ///
    /// Instead of printing a complex and a large contractible subcomplex, remove shared facets
    /// from both and print the resulting pair.
    ///
    /// The pair homotopy type will agree with the homotopy type of the input, but the first member
    /// of the pair will probably not have the same homotopy type as the input and the second
    /// member will probably not be contractible.
    ///
    /// If your complex is contractible and you enable this flag, the output will be empty. This is
    /// not a bug: the mapping cone of the identity map of the empty complex is contractible.
    #[arg(short, long, default_value_t = false)]
    pub minimize_pair: bool,

    /// Print the current complex after each pinch loop.
    ///
    /// Useful if you are simplifying a very large complex and would like to guard against
    /// unexpected interruptions, e.g. power outages. Be sure to redirect stdout to a file for this
    /// flag to be useful.
    #[arg(short = 'w', long, default_value_t = false)]
    pub write_each_pinch: bool,

    /// Print the output in XML format for parsing by the GAP simpcomp package.
    ///
    /// If you are using simpcomp to calculate properties of your complex, you can enable this flag
    /// to print the output in a format that can then be loaded into GAP:
    ///
    /// gap> my_simplified_complex := SCLoadXML("my-simplified-complex.sc");
    #[arg(short, long, default_value_t = false)]
    pub xml: bool,
}
