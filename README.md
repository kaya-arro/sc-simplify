# What it does

## Behavior

Reads a non-empty simplicial complex from `stdin` and prints an equivalent simplified complex or pair.

The primary motivation is to accelerate homology computations for extremely large and highly non-minimal complexes.

The default behavior prints a pair $X, C$ of simplicial complexes in which $X$ has the homotopy type of the input and $C$ is a large contractible subcomplex of $X$. The complexes $X$ and $C$ are delineated by a blank line.

## Niche

`sc-simplify` is fast, but so are other programs for homology computations. Even more than speed, `sc-simplify` aims for *memory efficiency*.

Since the mid-to-late-2000s, the fastest algorithms for calculating the homology of simpicial complexes have preprocessed their input using the [discrete Morse theory](https://www.emis.de/journals/SLC/wpapers/s48forman.pdf) (DMT) developed by Forman and built upon by many others. `sc-simplify` does not use DMT and is consequently (in my testing) on the order of three to six times slower than programs like [Perseus](https://people.maths.ox.ac.uk/nanda/perseus/) that do.

However, to use DMT, a program must typically store the cells of a complex in memory along with the values of a discrete Morse function or at least its gradient discrete vector field. As a consequence, it can be intractible to use such algorithms with exceedingly large complexes unless one has access to very large amounts of memory. This is especially true for programs geared toward persistence homology.

`sc-simplify`, in contast, only needs to store the facets and edges of a complex in memory along with the facets of some short-lived and usually-not-gigantic subcomplexes. As a result, it consumes orders of magnitude less memory than programs that utilize DMT while still enabling homology computations hundreds and even thousands of times faster than calculating the homology directly without preprocessing.

# Usage

## Options

For a comprehensive explanation of the available options, see `sc-simplify --help` or `man ./sc-simplify.1.gz`.

## Input

### Formatting input

The program reads its input from `stdin`. Each line is a facet presented as a space-separated list of vertices labeled by natural numbers less than $2^{32}$. The program is tolerant of excess whitespace. If non-empty non-maximal faces are included in the input, bugs may or may not arise; in this case, you should enable the`-c`/`--check-input` flag to ensure correct behavior.

Example input not requiring `--check-input`:

```text
0   1     3 2
4 1    3


2 5
```

 Example input requiring `--check-input`:

```text
0 1 2 3
1 2 3 4
2 3 4 0
1 3 4
0 1 4
```

Input that contains characters other than numerals, spaces, and newlines will cause `sc-simplify` to panic.

### Loading input

For those unused to the terminal: since `sc-simplify` reads fr`stdin`om `stdin`, you can redirect redirect your input from a file with `<`, pipe the input from the output of another command with `|`, or enter the input by hand, signalling its termination with `^D` (Ctrl + D) after a newline:

```shell
sc-simplify -NP1 < my-complex.simp > pinched.simp
sc-factory.sh | sc-simplify -c > checked.simp
sc-simplify -p > circle.simp
0 1
1 2
2 3
0 3
^D
```

## Output

`sc-simplify` prints its output to `stdout`, so if you wish to save the output as a file, you should redirect `stdout` using `>` (see the examples above and below). If `stderr` is a terminal and the  `-q`/`--quiet` flag is not enabled, `sc-simplify` prints progress indicators to `stderr`.

By default, the output has the same formatting as the input with the simplified complex and its contractible subcomplex delineated by a blank line.

```shell
sc-simplify -p < my-complex.simp > simplified.simp
chomp-simplicial simplified.simp
```

Alternatively, the `-x`/`--xml` flag can be enabled to yield a `.xml` file that can be loaded by GAP's `simpcomp` package with the `SCLoadXML` command:

```console
$ sc-simplify < my-complex.simp -px > simplified.xml.sc
$ gap
gap> LoadPackage("simpcomp");;
gap> x := SCLoadXML("simplified.xml.sc");;
gap> SCHomologyClassic(x);
```

Since GAP `simpcomp` does not compute relative homology, you should use the `-p`/`--no-pair` flag when preparing a complex as input for `simpcomp`.

If you would like to use the program with Sage, functions for importing and exporting simplicial complexes in `sc-simplify`'s format to and from Sage are provided in the file `Python/sc_io.py`. For example, the `read_sc_pair` function can be used like so:

```python
X, C = read_sc_pair("simplified.simp")
X.homology(subcomplex=C, enlarge=False, base_ring=QQ)
```

# Algorithm

The following steps are the default algorithm; they can be adjusted by passing various flags to the program. See `sc-simplify --help` for details.

1. Take the Čech nerve of the input iteratively until no more simplifications occur and the dimension of the complex is less than or equal to that of its nerve.

2. Identify edges that can be contracted without changing the homotopy type of the complex and contract them.

3. (Disabled by default) identify collapsible faces and collapse them.
   
   - This process does not generally expedite calculation of homotopy invariants, so you must enable the `--max-collapse-loops` flag with a positive number if you wish to use this functionality.

4. Construct a large contractible subcomplex.

# Installation

## Dependencies

- Rust

## Install process

1. If necessary, install Rust.
   
   - You can your operating system's package manager or Rust's [official installation shell script](https://www.rust-lang.org/tools/install):
   
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. If necessary (e.g. if you did not use the official installation script), update Rust to the latest edition with `rustup upgrade`.

3. Clone/download this repository.

4. Execute `cargo build -r` in the root directory of the repository.

5. Optional: copy or link `sc-simplify.1.gz` into `/usr/share/man/man1/` so that `man sc-simplify` brings up the manual (see below for how to do this from the terminal).

6. The binary can be found at `target/release/sc-simplify`. Link to it from your `PATH` with
   
   ```bash
   sudo ln -s $(pwd)/target/release/sc-simplify /usr/bin/
   ```
   
   If you plan to delete or move `target/release/sc-simplify`, make a hardlink by omitting the `-s` flag or copy the binary with
   
   ```bash
   sudo cp $(pwd)/target/release/sc-simplify /usr/bin/
   ```
   
   or generally do whatever you want with the binary: it's yours!

## Portability

Since the purpose of this program is efficiency, the `-C target-cpu=native`  `rustc` compile flag is enabled in `.cargo/config.toml`. This means that you may not be able to run the produced binary on a machine other than the one on which you built it (or at least a machine with the same processor). If you would like a more portable but possibly slower version of the program, comment out this flag.

# Development plans and hopes and dreams

- [x] Make tweaks to improve performance.

- [x] Make more tweaks to improve performance.

- [x] Make a progress bar.

- [ ] Improve the documentation of individual methods and publish the crate as a library on crates.io.

- [ ] Compile a binary to wasm so that users who do not wish to compile the program can use it.

- [ ] Include examples in the repository.

- [ ] Remove more excess cells created by `--thorough` by applying the collapse algorithm to remove cells of greater codimension than one.

- [ ] Maybe implement a flag to use a DMT algorithm.
  
  - The main motivation simply be the fact that a number of software packages implementing DMT algorithms seem no longer to be maintained. Perseus is usable for me, but I am unable to get CHomP or RedHom to compile on my machine, and the DMT functions of `simpcomp` also give me errors.

- [ ] Maybe someday implement integral simplicial homology in Rust, or use an existing crate that does?
  
  - Sage is pretty quick when taking homology with coefficients in a field. The main motivations to do this would be:
    
    - to accelerate the calculation of integral homology
    
    - to allow the use of a DMT algorithm that produces a Morse complex directly without having to hassle with exporting it to other software.

# Limitations

This package is not intended to be a general toolkit for working with simplicial complexes. It is focused on the single goal of filling what I perceived as a gap in the functionality of available software: efficiently reducing complexes to accelerate calculations of homotopy invariants (the functionality of the `--thorough` flag is the one exception to this).

Other tools exist for more general manipulations of simplicial complexes. A highly non-exhausitive list of these includes the `simplicial_topology` Rust crate, the `simpcomp` GAP package, and the `sage.topology.simplicial_complex` Sage/Python module.

I do not currently have plans to extend this package into something more ambitious, but I am open to feature requests that generally align with this package's aim of accelerating the computation of homotopy invariants and which are not implemented in other software.

# Acknowledgements

The [`enlarge_in_supercomplex`](https://github.com/kaya-arro/sc-simplify/blob/18f794887aeee89266d0038d1942aaa945ec8938/src/simplicial_complex.rs#L170) method in this program is largely ported from the [`_enarlge_subcomplex`](https://github.com/sagemath/sage/blob/871ba9daed15374d6b2ff1c533970f44b70f21e9/src/sage/topology/simplicial_complex.py#L3901) method used by Sage and written by John Palmieri.

# License

Copyright 2025 Kaya Arro. Released under the Apache 2.0 license. See the `LICENSE` file or view the license [online](http://www.apache.org/licenses/LICENSE-2.0).
