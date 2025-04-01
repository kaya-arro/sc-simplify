# What it does

Reads a non-empty simplicial complex from stdin and print an equivalent simplified complex or pair.

The default behavior prints a pair X, C of simplicial complexes in the same format as the input in which X has the homotopy type of the input and C is a large contractible subcomplex of X. The complexes X and C are delineated by a blank line.

The `--minimize-pair` flag instructs the program to removes the facets shared by the simplified complex and the subcomplex, while the `--no-pair` flag causes the program to skip calculating a contractible subcomplex entirely.

# Why

The primary goal is to accelerate homology computations. I wrote `sc-simplify` as a tool for computing the homology of complexes relevant to pure mathematics research, but I expect it to be useful for persistence homology computations on very large datasets.

# Usage

The program reads its input from `stdin`. Each line is a facet presented as a space-separated list of vertices labeled by natural numbers less than 2<sup>32</sup>.

Examples:

```bash
sc-simplify < my-complex.sc [OPTIONS ... ] > simplified.sc
sc-factory.sh | sc-simplify [OPTIONS ... ] > simplified.sc
```

You can also enter `stdin` by hand, terminating the input with `^D` as usual.

By the default, the output has the same formatting as the input. Alternatively, the `--xml` flag can be enabled to yield a `.xml` file that can be loaded by GAP's `simpcomp` package with the `SCLoadXML` command:

```bash

```

If you would like to use the program with `sage`, small scripts for importing and exporting simplicial complexes in `sc-simplify`'s format to and from `sage` are provided in the file `sage/sc_io.sage`.

For further usage details, read the help text: `sc-simplify --help`

# Algorithm

By default, `sc-simplify` takes the Čech nerve of the input iteratively until no more simplifications occur and the dimension of the complex is less than or equal to that of its nerve.

Next, the "pinch" algorithm is applied at most twice (configurable via the `--max-pinch-loops` flag). This algorithm is the central mathematical contribution of this package. Each edge is evaluated to determine if contracting it would change the homotopy type of the complex, and if not, it is contracted. For the sake of efficiency, a fast heuristic algorithm is used: this process does not generally yield a locally minimal simplicial complex, but it does yield a (potentially much) smaller complex very quickly.

Software like `sage` accelerates homology computations by finding a large contractible subcomplex and calculating relative homology with respect to this subcomplex. Since Rust is faster than Python, `sc-simplify` does this for you automatically by default so that you can use e.g.

```python
sage: sc1, sc2 = read_sc_pair("simplified-pair.sc")
sage: sc1.homology(subcomplex=sc2, enlarge=False) 
```

# Installation

1. If necessary, install Rust.

   - You can use Rust's [official installation shell script](https://www.rust-lang.org/tools/install):

   ```bash
      curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

   or, if you prefer, your operating system's package manager.

2. If necessary (e.g. if you did not use the official installation script), update Rust to the latest edition with `rust update`.

3. Clone this repository.

4. *Optional but strongly encouraged*: Specify the target architecture in `.cargo/config.toml` appropriate for your system. You can view a list of supported architectures with the command `rustup target list`.

   - Note: `.cargo/config.toml` has `rustflags = ['-Ctarget-cpu=native']` enabled. This means that the binary you build may not work on machines other than the one on which you build it. This lack of portability buys you the with the advantage of shorter run times because the compiler will optimize the binary for your specific CPU, but you can comment out this flag if you wish.

   - Because the goal of the project is efficient computation, you are encouraged to build the binary on each machine you will run it on.

5. Execute `cargo build -r` in the root directory of the repository.

6. Optional: Copy or link `sc-simplify.1.gz` into `/usr/share/man/man1/` so that `man sc-simplify` brings up the manual.

7. The binary can be found at `target/release/sc-simplify`. Link to it from your `PATH` with
   ```bash
      sudo ln -s $(pwd)/target/release/sc-simplify /usr/bin/
   ```
   If you plan to delete or move `target/release/sc-simplify`, make a hardlink by omitting the `-s` flag or copy the binary with
   ```bash
   sudo cp $(pwd)/target/release/sc-simplify /usr/bin/
   ```
   or generally do whatever you want with the binary: it's yours!

# Development plans and hopes and dreams

- [ ] Make tweaks to improve performance

- [ ] Implement optional multithreading to accelerate the "enlarge subcomplex" algorithm

- [ ] Implement a "pair mode" that enlarges a given subcomplex (and optionally "minifies" the pair by removing shared facets)

- [ ] Implement a flag to switch to a slower but more thorough internal contractibility test that could accomplish for more simplification at the cost of a slower run time.

- [ ] Implement a flag to enable a much slower mode that tries to find a truly minimal version of a complex or pair

- [ ] Implement a flag to enable an even more careful mode that prints partial progress while enlarging a subcomplex

- [ ] Include examples in the repository

- [ ] (Maybe) implement a flag to enable a status bar to give a sense of the program's progress.

   - This would require some refactoring of the program's I/O system. It would be kind of nice to have but isn't a high priority.

- [ ] Maybe someday implement integral simplicial homology in Rust

   - I am aware of a crate the implements mod 2 homology but not integral homology.

   - It would be a while before I would have time to get to this.

# License

Copyright 2025 Kaya Arro. Released under the Apache 2.0 license. See the `LICENSE` file or view the license [online](http://www.apache.org/licenses/LICENSE-2.0).
