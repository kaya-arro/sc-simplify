# What it does

Reads a non-empty simplicial complex from `stdin` and prints an equivalent simplified complex or pair.

The primary motivation is to accelerate homology computations for highly non-minimal complexes.

The default behavior prints a pair X, C of simplicial complexes in the same format as the input in which X has the homotopy type of the input and C is a large contractible subcomplex of X. The complexes X and C are delineated by a blank line.

# Usage

The program reads its input from `stdin`. Each line is a facet presented as a space-separated list of vertices labeled by natural numbers less than 2<sup>32</sup>. The program is tolerant of excess whitespace, but if non-maximal faces are included in the input, you should enable the `--check-input` flag.

Examples:

```bash
sc-simplify < my-complex.slcx [OPTIONS ... ] > simplified.slcx
sc-factory.sh | sc-simplify [OPTIONS ... ] > simplified.slcx
```

You can also enter `stdin` by hand, terminating the input with `^D`.

By default, the output has the same formatting as the input. Alternatively, the `--xml` flag can be enabled to yield a `.xml` file that can be loaded by GAP's `simpcomp` package with the `SCLoadXML` command:

```
$ sc-simplify < my-complex.slcx -px > simplified.xml
$ gap
gap> LoadPackage("simpcomp");;
gap> x := SCLoadXML("simplified.xml");;
gap> SCHomologyInternal(x);
```

If you would like to use the program with Sage, small scripts for importing and exporting simplicial complexes in `sc-simplify`'s format to and from Sage are provided in the file `Python/sc_io.py`. For example, the `read_sc_pair` script can be used like so:

```python
X, C = read_sc_pair("simplified-pair.slcx")
X.homology(subcomplex=C, enlarge=False, base_ring=QQ)
```

For further usage details, see the help text: `sc-simplify --help`

# Algorithm

The following steps are the default algorithm; they can be adjusted by passing various flags to the program. See `sc-simplify --help` for details.

1. Take the Čech nerve of the input iteratively until no more simplifications occur and the dimension of the complex is less than or equal to that of its nerve.

2. Identify edges that can be contracted without changing the homotopy type of the complex and contract them.

3. Identify collapsible faces and collapse them.

4. Construct a large contractible subcomplex.

# Installation

1. If necessary, install Rust.
   
   - You can your operating system's package manager or Rust's [official installation shell script](https://www.rust-lang.org/tools/install):
   
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. If necessary (e.g. if you did not use the official installation script), update Rust to the latest edition with `rustup upgrade`.

3. Clone this repository.

4. Execute `cargo build -r` in the root directory of the repository.

5. Optional: copy or link `sc-simplify.1.gz` into `/usr/share/man/man1/` so that `man sc-simplify` brings up the manual (see below for how to do this from the terminal).

6. The binary can be found at `target/release/sc-simplify`. Link to it from your `PATH` with
   
   ```bash
   sudo ln -s $(pwd)/target/release/sc-simplify /usr/local/bin/
   ```
   
   If you plan to delete or move `target/release/sc-simplify`, make a hardlink by omitting the `-s` flag or copy the binary with
   
   ```bash
   sudo cp $(pwd)/target/release/sc-simplify /usr/local/bin/
   ```
   
   or generally do whatever you want with the binary: it's yours!

# Development plans and hopes and dreams

- [x] Make tweaks to improve performance.

- [x] Make more tweaks to improve performance.

- [x] Make a progress bar.

- [ ] Improve the documentation of individual methods and publish the crate as a library on crates.io.

- [ ] Include examples in the repository.

- [ ] Remove more excess cells created by `--thorough` by applying the collapse algorithm to cells of greater codimension.

- [ ] Maybe someday implement integral simplicial homology in Rust??
  
  - It would be quite a while before I would have time to get to this.
  - Sage is pretty quick if you take coefficients in a field. Integral homology is much slower.

# Limitations

This package is not intended to be a general toolkit for working with simplicial complexes in Rust. It is focused on the single goal of filling what I perceived as a gap in the functionality of available software: efficiently reducing complexes to accelerate calculations of homotopy invariants (the functionality of the `--thorough` flag is the one expection to this).

Other tools exist for more general manipulations of simplicial complexes. A highly non-exhausitive list of these includes the `simplicial_topology` Rust crate, the `simpcomp` GAP package, and the `sage.topology.simplicial_complex` Sage/Python module.

I do not currently have plans to extend this package into something more ambitious, but I am open to feature requests that generally align with this package's aim of accelerating the computation of homotopy invariants and which are not implemented in other software.

# Acknowledgements

The [`enlarge_in_supercomplex`](https://github.com/kaya-arro/sc-simplify/blob/18f794887aeee89266d0038d1942aaa945ec8938/src/simplicial_complex.rs#L170) method in this program is largely ported from the [`_enarlge_subcomplex`](https://github.com/sagemath/sage/blob/871ba9daed15374d6b2ff1c533970f44b70f21e9/src/sage/topology/simplicial_complex.py#L3901) method used by Sage and written by John Palmieri.

# License

Copyright 2025 Kaya Arro. Released under the Apache 2.0 license. See the `LICENSE` file or view the license [online](http://www.apache.org/licenses/LICENSE-2.0).
