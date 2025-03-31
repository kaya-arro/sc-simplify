# What it does

Reads a non-empty simplicial complex from stdin and print an equivalent simplified complex or pair.

The default behavior prints a pair X, C of simplicial complexes in the same format as the input in which X has the homotopy type of the input and C is a large contractible subcomplex of X. The complexes X and C are delineated by a blank line.

There are also options to remove the facets shared by the simplified complex and the subcomplex (`--minimize-pair`) or to forego calculating a subcomplex entirely (`--no-pair`).

# Why

The primary goal is to accelerate homology computations. I wrote `sc-simplify` as a tool for computing the homology of complexes relevant to pure mathematics research, but I expect it to be useful for persistence homology computations on very large datasets.

# Usage

The program reads its input from `stdin`. Each line is a facet presented as a space-separated list of vertices labeled by natural numbers less than 2<sup>32</sup>.

Examples:

```bash
$ sc-simplify < my-complex.sc [OPTIONS ...] > simplified.sc
```

```bash
$ sc-factory.sh | sc-simplify [OPTIONS ...] > simplified.sc
```

You can also enter `stdin` by hand, terminating the input with `^D` as usual.

By the default, the output has the same formatting as the input. Alternatively, the `--xml` flag can be enabled to yield a `.xml` file that can be loaded by GAP's `simpcomp` package with the `SCLoadXML` command.

If you would like to use the program with `sage`, small scripts for importing and exporting simplicial complexes in `sc-simplify`'s format to and from `sage` are provided in the file `sage/sc_io.sage`.

For furthere usage details, read the help text:

```bash
sc-simplify --help
```

# Algorithm

By default, `sc-simplify` takes the Čech nerve of the input iteratively until no more simplifications occur no more simplifications occur and the dimension of the complex is less than or equal to that of its nerve.

Next, the "pinch" algorithm is applied at most twice (configurable via the `--max-pinch-loops` flag). This algorithm is the central mathematical contribution of this package. Each edge is evaluated to determine if contracting it would change the homotopy type of the complex, and if not, it is contracted. For the sake of efficiency, a fast heuristic algorithm is used: this process does not generally yield a locally minimal simplicial complex, but it does yield a (potentially much) smaller complex very quickly.

Software like `sage` accelerates homology computations by find a large contractible subcomplex and calculating relative homology with respect to this subcomplex. Since Rust is faster than Python, `sc-simplify` does this for you automatically by default so that you can use e.g.

```python
sage: sc1, sc2 = read_sc_pair("simplified-pair.sc")
sage: sc1.homology(subcomplex=sc2, enlarge=False) 
```

# Installation

1. Intall `rust` or `rustup` through your distribution's package manager.

2. Clone the repo with `gh repo clone kaya-arro/sc-simplify`  or `git clone https://github.com/kaya-arro/sc-simplify.git` or through your browser.

3. Optional but encouraged: Specify the target architecture in `.cargo/config.toml` appropriate for your system. You can view a list of supported architectures with the command `rustup target list`.

4. Execute `cargo build -r` in the root directory of the package.

5. Optional: Copy `sc-simplify.1.gz` into `/usr/share/man/man1/`.

6. The binary can be found at `target/release/sc-simplify`. Copy it into your `PATH` (probably into `/usr/bin/`) or do whatever you want with it.

# To do

I have just written this program and am still trying out a couple of performance tweaks. I will probably also add some example files to the repo.

Although the program is focused on efficient calculations, I also have plans to implement a more computationally expensive algorithm to produce minimal simplicial complexes.

I may also add a progress bar, but that is a lower priority.

# License

Copyright 2025 Kaya Arro. Released under the Apache 2.0 license. See the `LICENSE` file or view the license [online](http://www.apache.org/licenses/LICENSE-2.0).
