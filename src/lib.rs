// Flags for the compiler
// #![feature(impl_trait_in_assoc_type)]
#![feature(once_cell_get_mut, thread_local, hash_set_entry, btree_cursors, iter_chain)]

// Resources shared between modules
// from the std library
use fmt::{Debug, Display};
use std::cmp::{Ordering, Reverse, max};
use std::default::Default;
use std::fmt;
use std::hash::Hash;
use std::ops::{AddAssign, SubAssign};
// use std::rc::Rc;

use indicatif::ProgressBar;
use num::Integer;

// Private modules
mod helpers;
mod partial_bijection_complex;
mod simplicial_complex;
mod style;

// Public resources
pub mod io;
pub use partial_bijection_complex::partial_bijection_complex;
pub use simplicial_complex::Face;
pub use simplicial_complex::SimplicialComplex;

pub trait Vertex:
    Default
    + Copy
    + Debug
    + Display
    + Hash
    + Integer
    + AddAssign
    + SubAssign
    + TryFrom<usize>
    + TryInto<usize>
    + Send
    + Sync
{
}

impl<T> Vertex for T where
    T: Default
        + Copy
        + Debug
        + Display
        + Hash
        + Integer
        + AddAssign
        + SubAssign
        + TryFrom<usize>
        + TryInto<usize>
        + Send
        + Sync
{
}
