# What it does

Reads a non-empty simplicial complex from `stdin` and prints an equivalent simplified complex or pair.

The primary motivation is to accelerate homology computations for highly non-minimal complexes.

The default behavior prints a pair X, C of simplicial complexes in the same format as the input in which X has the homotopy type of the input and C is a large contractible subcomplex of X. The complexes X and C are delineated by a blank line.

 The `--no-pair` flag causes the program to skip calculating a contractible subcomplex.

# Usage

The program reads its input from `stdin`. Each line is a facet presented as a space-separated list of vertices labeled by natural numbers less than 2<sup>32</sup>.

Examples:

```bash
sc-simplify < my-complex.sc [OPTIONS ... ] > simplified.sc
sc-factory.sh | sc-simplify [OPTIONS ... ] > simplified.sc
```

You can also enter `stdin` by hand, terminating the input with `^D`.

By the default, the output has the same formatting as the input. Alternatively, the `--xml` flag can be enabled to yield a `.xml` file that can be loaded by GAP's `simpcomp` package with the `SCLoadXML` command:

```
$ sc-simplify < my-complex.sc -px > simplified.xml
$ gap
gap> LoadPackage("simpcomp");;
gap> x := SCLoadXML("simplified.xml");;
gap> SCHomologyInternal(x);
```

If you would like to use the program with `sage`, small scripts for importing and exporting simplicial complexes in `sc-simplify`'s format to and from `sage` are provided in the file `sage/sc_io.sage`.

For further usage details, read the help text: `sc-simplify --help`

# Algorithm

By default, `sc-simplify` takes the Čech nerve of the input iteratively until no more simplifications occur and the dimension of the complex is less than or equal to that of its nerve.

Next, the "pinch" algorithm is applied. This algorithm is the central mathematical contribution of this package. Each edge is evaluated to determine if contracting it would change the homotopy type of the complex, and if not, it is contracted. 

Software like `sage` accelerates homology computations by finding a large contractible subcomplex and calculating relative homology with respect to this subcomplex. Since Rust is faster than Python, `sc-simplify` does this for you automatically by default so that you can use e.g.

```python
sc1, sc2 = read_sc_pair("simplified-pair.sc")
sc1.homology(subcomplex=sc2, enlarge=False)
```

# Installation

1. If necessary, install Rust.
   
   - You can your operating system's package manager or Rust's [official installation shell script](https://www.rust-lang.org/tools/install):
   
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

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

- [x] Make tweaks to improve performance.

- [ ] Make more tweaks to improve performance.

- [ ] Implement optional multithreading to accelerate the `enlarge_in_supercomplex` algorithm.

- [ ] Implement a "pair mode" that enlarges a given subcomplex.

- [x] Implement a flag to switch to a slower but more thorough internal contractibility test that could achieve more simplification at the cost of slower run time.

- [ ] Include shell scripts to facilitate interfacing with GAP and Sage.

- [ ] Implement a flag to enable a much slower mode that tries to find a truly minimal version of a complex or pair.

- [ ] Implement a flag to enable a careful mode that prints partial progress while enlarging a subcomplex.

- [ ] Include examples in the repository.

- [ ] (Maybe) implement a flag to enable a status bar to give a sense of the program's progress.
  
  - This would require some refactoring of the program's I/O system. It would be kind of nice to have but isn't a high priority.

- [ ] The `simplicial_topology` crate implements mod 2 homology for simplicial complexes. Maybe write an interface with that crate.

- [ ] Maybe someday implement integral simplicial homology in Rust.
  
  - It would be quite a while before I would have time to get to this.

# Acknowledgements

The [`enlarge_in_supercomplex`]([sc-simplify/src/simplicial_complex.rs at 18f794887aeee89266d0038d1942aaa945ec8938 · kaya-arro/sc-simplify · GitHub](https://github.com/kaya-arro/sc-simplify/blob/18f794887aeee89266d0038d1942aaa945ec8938/src/simplicial_complex.rs#L170)) method in this program is largely ported from the [`_enarlge_subcomplex`]([sage/src/sage/topology/simplicial_complex.py at 871ba9daed15374d6b2ff1c533970f44b70f21e9 · sagemath/sage · GitHub](https://github.com/sagemath/sage/blob/871ba9daed15374d6b2ff1c533970f44b70f21e9/src/sage/topology/simplicial_complex.py#L3901)) method used by Sage and written by John Palmieri.

# License

Copyright 2025 Kaya Arro. Released under the Apache 2.0 license. See the `LICENSE` file or view the license [online](http://www.apache.org/licenses/LICENSE-2.0).
