//! Data structures and routines for a trie of tuple `(K,T,V,W)`.
//! 
//! We use a trie to represent a sorted sequence of tuples of the form `(K,T,V,W)`
//! where each of `K`, `T`, `V` implement `Ord`, and `W` is some accumulator type
//! such as `isize`. 
//! 
//! The trie is a compact *static* representation of such a list, but we want to be
//! able to add tuples to the trie. To accomplish this, we maintain a collection of
//! tries of (geometrically) varying sizes, and work to continually maintain this 
//! property as additional tuples are added. We must also provide an interface which
//! can merge the contents of multiple tries to provide the appearance of one ordered
//! sequence.

extern crate fnv;

pub mod merge;
pub mod arbor;
pub mod arbor_index;
pub mod trie;

pub use arbor::Arbor;
pub use merge::CursorMerger;

pub use trie::{TrieStorage, TrieRef};