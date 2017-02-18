//! A collection of `Trie<K,T,V>` tries.
//! 
//! An `Arbor` is backed by multiple `Trie` structures of varying sizes, 
//! designed to provide both efficient enumeration of its contents and 
//! addition of new tuples. 
//! 
//! The main functionality of the `Arbor` is to continually merge tries 
//! whose sizes are the same order of magnitude. This keeps a bounded 
//! number of tries, so that enumeration remains efficient, while doing
//! an amortized logarithmic amount of work for each introduced tuple, 
//! which should be asymptotically optimal as the product of the `Arbor`
//! is an ordered representation of its contents.

use std::collections::HashMap;
use std::hash::Hash;
use std::hash::BuildHasherDefault;
use std::collections::hash_map::Entry;

use fnv::FnvHasher;

use {TrieRef, TrieStorage, CursorMerger};
use ::trie::TrieLayer;

struct KeyLocation {
	index: usize,
	offset: usize,
	next: Option<usize>,
}

impl KeyLocation {
	fn new(index: usize, offset: usize, next: Option<usize>) -> KeyLocation {
		KeyLocation {
			index: index,
			offset: offset,
			next: next,
		}
	}
}

#[derive(Debug)]
pub struct ArborIndex<K: Ord+Hash, L: TrieStorage> {

	// storage for keyed trie values, from largest to smallest.
	// the `usize` is the number of spilled KeyLocation entries.
	tries: Vec<(TrieLayer<K, L>, usize)>,

	// indicates a smallest entry in `spill`.
	//
	// TODO : put the first entry of `spill` here, and only successive entries
	// into `spill`. This complicates the logic for updating the index, but makes
	// for fewer dereferences when a key doesn't spill. All keys are guaranteed 
	// to have an entry, so it isn't wasted space.
	// index: HashMap<K, usize>,
	index: HashMap<K, (usize, usize, Option<usize>), BuildHasherDefault<FnvHasher>>,

	// indicates a (layer, offset) and optionally a next entry in `spill`.
	spill: Vec<(usize, usize, Option<usize>)>,	
}

impl<K: Ord+Hash+Clone, L: TrieStorage> ArborIndex<K, L> {

	/// Allocates a new empty arbor.
	pub fn new() -> ArborIndex<K, L> {

		// let map: HashMap<K, usize, BuildHasherDefault<FnvHasher>> = Default::default();
		ArborIndex { 
			tries: Vec::new(),
			index: ::std::default::Default::default(),
			spill: Vec::new(),
		}
	}

	/// Reports the number of tuples across all managed tries.
	///
	/// Note that this number may be greater than the number of distinct elements
	/// enumerated by `cursor`, which has the opportunity to merge like elements.
	pub fn size(&self) -> usize {
		let mut count = 0;
		for trie in &self.tries {
			count += trie.0.tuples();
		}
		count
	}

	/// Adds an ordered sequence of tuples to the collection.
	///
	/// If the tuples aren't in order, something horrible probably happens.
	/// Nothing memory unsafe, but ... why would you do this?
	pub fn extend_ordered<I: Iterator<Item=(K, L::Item)>>(&mut self, iterator: I) {
		self.append(TrieLayer::<K, L>::from_ordered(iterator));
	}

	/// Adds an entire trie into the collection.
	///
	/// This method can be helpful if the resources required for the trie
	/// representation are already available, and avoids re-allocating them
	/// in `extend_ordered`. The method can be quite fast in this case, as
	/// it does not need to re-process every tuple in the input batch.
	///
	/// The method will perform merging of tries if the introduced trie has
	/// a size within a factor of two of the smallest trie the arbor currently
	/// manages. This can be quite *not fast*, but it should be improved with
	/// progressive merging.
	pub fn append(&mut self, mut trie: TrieLayer<K, L>) {

		while self.tries.last().map(|x| x.0.tuples() <= 2 * trie.tuples()) == Some(true) {
			
			let (other, count) = self.tries.pop().unwrap();
			
			// pop entries from self.index.
			//
			// TODO : merge could track a list of discarded keys, as we can then
			// update them, followed by the keys in the merged results, rather than
			// all of the indexing we do here. Measure this, then implement that.
			for &(ref key, _) in &other.keys {
				match self.index.entry(key.clone()) {
					Entry::Occupied(mut entry) => { 
						if let Some(next) = entry.get().2 {
							*entry.get_mut() = self.spill[next];
						}
						else {
							entry.remove();
						}
					},
					Entry::Vacant(mut entry) => { 
						unreachable!();
					},
				}
			}

			for _ in 0 .. count { self.spill.pop(); }

			trie = trie.merge(&other);
		}

		// update index for all keys in the result of the merge.
		let mut spill_len = self.spill.len();
		for (pos, key) in trie.keys.iter().map(|x| &x.0).enumerate() {
			match self.index.entry(key.clone()) {
				Entry::Occupied(mut entry) => { 
					self.spill.push(*entry.get());
					*entry.get_mut() = (self.tries.len(), pos, Some(self.spill.len() - 1));
				},
				Entry::Vacant(mut entry) => { 
					entry.insert((self.tries.len(), pos, None));
				},
			}
		}

		let count = self.spill.len() - spill_len;
		self.tries.push((trie, count));
	}
}

impl<'a, K: Ord+Hash, L: TrieStorage+TrieRef<'a>> ArborIndex<K, L> {
	/// Provides a cursor for traversing the arbor's contents.
	pub fn cursor(&'a self) -> CursorMerger<'a, ::trie::TrieCursor<'a, K, L>> {
		CursorMerger::from(self.tries.iter().map(|x| x.0.cursor(0, x.0.keys_cnt())))
	}

	/// Populates an existing cursor merger with cursors for values for a given key.
	///
	/// If the key does not exist in the collection, the merger will simply be empty.
	pub fn get_into(&'a self, key: &K, cursor: &mut CursorMerger<'a, <L as TrieRef<'a>>::Cursor>) {
		cursor.clear();

		let mut next = self.index.get(key).map(|&x| x);
		while let Some((index, offset, spill)) = next {
			let lower = if offset == 0 { 0 } else { self.tries[index].0.keys[offset - 1].1 };
			let upper = self.tries[index].0.keys[offset].1;
			cursor.push(self.tries[index].0.vals.cursor(lower, upper));
			next = spill.map(|next| self.spill[next]);
		}

		cursor.cursors.sort_by(|x,y| (x.0).0.cmp(&(y.0).0));
	}
}