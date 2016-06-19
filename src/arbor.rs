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

use TrieStorage;
use TrieRef;
use CursorMerger;

#[derive(Debug, PartialEq, Eq)]
pub struct Arbor<T: TrieStorage> {
	tries: Vec<T>,
}

impl<T: TrieStorage+::std::fmt::Debug+Sized> Arbor<T> {

	pub fn new() -> Arbor<T> {
		Arbor { tries: vec![] }
	}

	/// Adds a single tuple to the collection.
	///
	/// This method should be called rarely if possible. It performs 
	/// allocation for each invocation, which can be avoided by using
	/// batch insertion methods like `extend`, `extend_ordered`, and
	/// `append`.
	pub fn push(&mut self, tuple: T::Item) {
		self.append(T::from_ordered(Some(tuple).into_iter()));
	}

	// /// Adds a unordered sequence of tuples to the collection.
	// pub fn extend<I: Iterator<Item=(K,T,V,isize)>>(&mut self, iterator: I) {
	// 	let mut vector = iterator.collect::<Vec<_>>();
	// 	self.append(Trie::from_vector(&mut vector));
	// }

	/// Adds an ordered sequence of tuples to the collection.
	pub fn extend_ordered<I: Iterator<Item=T::Item>>(&mut self, iterator: I) {
		self.append(T::from_ordered(iterator));
	}

	/// Adds an entire trie into the collection.
	///
	/// This method can be helpful if the resources required for the trie
	/// representation are already available, and avoids re-allocating them
	/// in `extend_ordered`. The method is also quite fast in this case, as
	/// it does not need to re-process every tuple in the input batch.
	///
	/// The method will perform merging of tries if the introduced trie has
	/// a size within a factor of two of the smallest trie the arbor currently
	/// manages. This can be quite *not fast*, but it should be improved with
	/// progressive merging.
	#[inline(never)]
	pub fn append(&mut self, trie: T) {

		// println!("appending length: {}", trie.total_length());

		// This method could be optimized to search out an empty location where
		// the trie can be inserted. It presently accumulates up any small tries
		// as it goes, which ensures the sizing invariant but also performs work
		// it may not have needed to do just yet.

		self.tries.push(trie);
		while self.tries.len() > 1 {

			// acquire the last two elements
			let trie1 = self.tries.pop().unwrap();
			let trie2 = self.tries.pop().unwrap();

			// if trie1 is within 2x of trie2 merge, ...
			if trie1.tuples() > trie2.tuples() / 2 {
				let mut result = T::with_capacity(&trie1, &trie2);
				result.extend_merge((&trie1, 0, trie1.keys()), (&trie2, 0, trie2.keys()));
				self.tries.push(result);
			}
			// ... otherwise push them back and return.
			else {
				self.tries.push(trie2);
				self.tries.push(trie1);
				return;
			}
		}
	}

	pub fn cursor<'a>(&'a self) -> CursorMerger<<&'a T as TrieRef>::Cursor> where &'a T : TrieRef {
		let mut result = CursorMerger::<<&'a T as TrieRef>::Cursor>::new();
		for trie in &self.tries {
			result.push(trie.cursor(0, trie.keys()));
		}
		result.sort();
		result
	}
}