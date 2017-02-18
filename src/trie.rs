//! Traits and types supporting general tuple trie implementations.

/// An iterator replacement for ordered sequences with random access.
///
/// Types implementing `Cursor` are able to both iterate through elements in
/// sequence, and to seek to specific keys (or the first key after them, if
/// the target key does not exist). There is no specific promise that seeking
/// elements will be fast, but the cursors implemented in this crate all have
/// the property that they take at most a number of steps logarithmic in the 
/// distance to the target key.
pub trait Cursor<'a> {
 	/// A strictly increasing key for enumerated items.
	type Key: Ord+'a;
	/// An arbitrary payload for each item.
	type Val;
	/// Advances the cursor and returns the next item. 
	fn next(&mut self) -> Option<(&'a Self::Key, Self::Val)>;
	/// Advances the cursor to the first element with key greater or equal to `key`.
	fn seek(&mut self, key: &Self::Key);
	/// Returns the key of the next item, if one exists.
	fn peek(&self) -> Option<&'a Self::Key>;
	/// Returns the number of items remaining.
	fn size(&self) -> usize;
}

/// A reference to a trie, capable of enumerating ranges of values.
pub trait TrieRef<'a> : 'a {
	/// The type of cursor the trie reference uses to navigate its elements.
	type Cursor: Cursor<'a>;
	/// The number of keys in this layer of the trie.
	fn keys_cnt(&self) -> usize;
	/// Returns a cursor for a range of elements in the trie.
	fn cursor(&'a self, lower: usize, upper: usize) -> Self::Cursor;
	/// Returns a cursor for the full range of elements in the trie.
	fn enumerate(&'a self) -> Self::Cursor {
		self.cursor(0, self.keys_cnt())
	}
}

/// A trie with owned data that may be pushed into. 
pub trait TrieStorage : Sized {
	/// Type of the item stored in the trie.
	type Item;
	/// Allocates a new empty trie.
	fn new() -> Self;
	/// Allocates a new empty trie sized to hold both `other1` and `other2`.
	fn with_capacity(other1: &Self, other2: &Self) -> Self;
	/// Reports the number of distinct keys at this level.
	fn keys(&self) -> usize;
	/// Reports the number of tuples in the trie.
	fn tuples(&self) -> usize;
	/// Extends the trie by the range of the supplied trie.
	fn extend_trie(&mut self, other: &Self, lower: usize, upper: usize);
	/// Merges two other tries, with supplied lower and upper indices, into this trie.
	fn extend_merge(&mut self, other1: (&Self, usize, usize), other2: (&Self, usize, usize));
	/// Pushes one tuple on; used for trie construction.
	fn extend_tuple(&mut self, tuple: Self::Item, is_new: bool);

	fn merge(&self, other: &Self) -> Self {
		let mut result = Self::with_capacity(self, other);
		result.extend_merge((self, 0, self.keys()), (other, 0, other.keys()));
		result
	}

	/// Creates a new trie from an ordered sequence of items.
	fn from_ordered<I: Iterator<Item=Self::Item>>(iter: I) -> Self {
		let mut result = Self::new();
		for item in iter {
			result.extend_tuple(item, false);
		}
		result
	}
}	

/// A layer of a trie wrapped around another trie.
///
/// A `TrieLayer` contains a list of `(K, usize)` elements indicating key values
/// of type `K` and the offset in `vals` where their corresponding range *ends*.
/// Their corresponding range starts either at zero, or at the end of the range 
/// of the immediately preceding key.
#[derive(Debug)]
pub struct TrieLayer<K:Ord, L> {
	pub keys: Vec<(K, usize)>,
	pub vals: L,
}

impl<K:Ord+Clone, L: TrieStorage> TrieStorage for TrieLayer<K, L> {
	type Item = (K, L::Item);
	fn new() -> Self { TrieLayer { keys: vec![], vals: L::new() }}
	fn with_capacity(other1: &Self, other2: &Self) -> Self {
		TrieLayer { 
			keys: Vec::with_capacity(other1.keys.len() + other2.keys.len()),
			vals: L::with_capacity(&other1.vals, &other2.vals),
		}
	}
	fn keys(&self) -> usize { self.keys.len() }
	fn tuples(&self) -> usize { self.vals.tuples() }
	fn extend_trie(&mut self, other: &Self, lower: usize, upper: usize) {

		// not sure that this is critical, but we will access upper-1.
		assert!(lower < upper);

		// a memcpy would be nice here, but all of the offsets need to be corrected.
		// in principle we could re-think this so that all offsets are relative to 
		// the restriction defined by parent keys, which would mean MEMCPY HO!
		//
		// Not yet.

		// we want to capture the keys but update all of their offsets appropriately,
		// based on vals.length().
		let other_basis = if lower == 0 { 0 } else { other.keys[lower-1].1 };
		let self_basis = self.vals.keys();
		self.keys.reserve(upper - lower);
		self.keys.extend(other.keys[lower .. upper]
							  .iter()
							  .map(|&(ref k, c)| (k.clone(), (c + self_basis) - other_basis)));
		// move all of the values over ...
		self.vals.extend_trie(&other.vals, other_basis, other.keys[upper-1].1);

		assert!(self.vals.keys() == self.keys[self.keys.len()-1].1);	
	}
	fn extend_merge(&mut self, other1: (&Self, usize, usize), other2: (&Self, usize, usize)) {
		let (trie1, mut lower1, upper1) = other1;
		let (trie2, mut lower2, upper2) = other2;

		self.keys.reserve(upper1 + upper2 - lower1 - lower2);

		// while both mergees are still active
		while lower1 < upper1 && lower2 < upper2 {
			match (trie1.keys[lower1].0).cmp(&(trie2.keys[lower2].0)) {
				::std::cmp::Ordering::Less => {
					// determine how far we can advance lower1 until we reach/pass lower2
					let step = 1 + advance(&trie1.keys[(1+lower1)..upper1], |x| x.0 < trie2.keys[lower2].0);
					assert!(step > 0);
					self.extend_trie(trie1, lower1, lower1 + step);
					lower1 += step;
				}
				::std::cmp::Ordering::Equal => {
					// need to merge vals and then push the key if the merge pushed vals.
					let v_lower1 = if lower1 == 0 { 0 } else { trie1.keys[lower1-1].1 };
					let v_lower2 = if lower2 == 0 { 0 } else { trie2.keys[lower2-1].1 };
					let v_upper1 = trie1.keys[lower1].1;
					let v_upper2 = trie2.keys[lower2].1;

					// record vals_length so we can tell if anything was pushed.
					let v_len = self.vals.keys();
					self.vals.extend_merge(
						(&trie1.vals, v_lower1, v_upper1), 
						(&trie2.vals, v_lower2, v_upper2)
					);
					if self.vals.keys() > v_len {
						self.keys.push((trie1.keys[lower1].0.clone(), self.vals.keys()));
					}

					lower1 += 1;
					lower2 += 1;
				} 
				::std::cmp::Ordering::Greater => {
					// determine how far we can advance lower2 until we reach/pass lower1
					let step = 1 + advance(&trie2.keys[(1+lower2)..upper2], |x| x.0 < trie1.keys[lower1].0);
					assert!(step > 0);
					self.extend_trie(trie2, lower2, lower2 + step);
					lower2 += step;
				}
			}
		}

		if lower1 < upper1 { self.extend_trie(trie1, lower1, upper1); }
		if lower2 < upper2 { self.extend_trie(trie2, lower2, upper2); }
	}
	fn extend_tuple(&mut self, tuple: (K, L::Item), is_new: bool) {
		// if is_new or the key is not the same as the last key, advance.
		let is_new = if is_new || self.keys.last().map(|x| x.0 != tuple.0).unwrap_or(true) {
			self.keys.push((tuple.0, 0));
			true
		}
		else {
			false
		};
		self.vals.extend_tuple(tuple.1, is_new);
		let len = self.keys.len();
		self.keys[len-1].1 = self.vals.keys();
	}
}

impl<'a, K:Ord+'a, L:'a> TrieRef<'a> for TrieLayer<K,L> where L: TrieRef<'a> {
	type Cursor = TrieCursor<'a, K, L>;
	fn keys_cnt(&self) -> usize { self.keys.len() }
	fn cursor(&'a self, lower: usize, upper: usize) -> Self::Cursor {
		// type annotations apparently important to keep Rust from asploding.
		TrieCursor::<'a,K,L>::new(&self.keys[lower .. upper], &self.vals)
	}
}

pub struct TrieCursor<'a, K:Ord+'a, L:'a> {
	pub index: usize,
	pub keys: &'a [(K, usize)],
	pub vals: &'a L,
}

impl<'a, K:Ord+'a, L> TrieCursor<'a,K,L> where L: TrieRef<'a> {
	pub fn new(keys: &'a [(K, usize)], vals: &'a L) -> TrieCursor<'a,K,L> {
		TrieCursor::<'a,K,L> {
			index: 0, 
			keys: keys,
			vals: vals,
		}
	}
}

impl<'a, K:Ord+'a, L> Cursor<'a> for TrieCursor<'a,K,L> where L: TrieRef<'a> {

	type Key = K;
	type Val = <L as TrieRef<'a>>::Cursor;

	fn next(&mut self) -> Option<(&'a Self::Key, Self::Val)> {
		if self.index < self.keys.len() {
			let current = self.index;
			self.index += 1;

			let v_lower = if current == 0 { 0 } else { self.keys[current-1].1 };
			let v_upper = self.keys[current].1;

			Some((
				&self.keys[current].0,
				self.vals.cursor(v_lower, v_upper),
			))
		}
		else {
			None
		}
	}

	#[inline(never)]
	fn seek(&mut self, key: &Self::Key) {
		self.index += advance(&self.keys[self.index ..], |x| &x.0 < key);
		// assert!(self.index >= self.keys.len() || &self.keys[self.index].0 >= key);
	}
	fn peek(&self) -> Option<&'a Self::Key> {
		if self.index < self.keys.len() { Some(&self.keys[self.index].0) } else { None }
	}
	fn size(&self) -> usize {
		self.keys.len() - self.index
	}
}

impl<'a, K:Ord+'a, L:'a> Clone for TrieCursor<'a,K,L> {
	fn clone(&self) -> Self {
		TrieCursor::<'a,K,L> {
			index: self.index,
			keys: self.keys,
			vals: self.vals,
		}
	}
}

/// A trie with owned data that may be pushed into. 
impl<K:Ord+Clone> TrieStorage for Vec<(K, i32)> {
	type Item = (K, i32);
	fn new() -> Self { vec![] }
	fn with_capacity(other1: &Self, other2: &Self) -> Self { 
		Vec::with_capacity(other1.len() + other2.len()) 
	}
	fn keys(&self) -> usize { self.len() }
	fn tuples(&self) -> usize { self.len() }
	fn extend_trie(&mut self, other: &Self, lower: usize, upper: usize) {
		debug_assert!(lower < upper);
		self.reserve(upper - lower);
		self.extend_from_slice(&other[lower .. upper]);

	// unsafe {
	//     let position = self.len();
	//     let slice = &other[lower .. upper];
	//     ::std::ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr().offset(position as isize), slice.len());
	//     self.set_len(position + slice.len());
	// }
	}
	fn extend_merge(&mut self, other1: (&Self, usize, usize), other2: (&Self, usize, usize)) {

		let (vec1, mut lower1, upper1) = other1;
		let (vec2, mut lower2, upper2) = other2;

		// perhaps overly aggressive
		self.reserve(upper1 + upper2 - lower1 - lower2);

		while lower1 < upper1 && lower2 < upper2 {
			match vec1[lower1].0.cmp(&vec2[lower2].0) {
				::std::cmp::Ordering::Less => {
					let step = 1 + advance(&vec1[(1+lower1)..upper1], |x| x.0 < vec2[lower2].0);
					assert!(step > 0);
					self.extend_trie(&vec1, lower1, lower1 + step);
					lower1 += step;
				}
				::std::cmp::Ordering::Equal => {
					let count = vec1[lower1].1 + vec2[lower2].1;
					if count != 0 {
						self.push((vec1[lower1].0.clone(), count));
					}
					lower1 += 1;
					lower2 += 1;
				}
				::std::cmp::Ordering::Greater => {
					let step = 1 + advance(&vec2[(1+lower2)..upper2], |x| x.0 < vec1[lower1].0);
					assert!(step > 0);
					self.extend_trie(&vec2, lower2, lower2 + step);
					lower2 += step;
				}
			}
		}

		if lower1 < upper1 { self.extend_trie(&vec1, lower1, upper1); }
		if lower2 < upper2 { self.extend_trie(&vec2, lower2, upper2); }
	}
	fn extend_tuple(&mut self, tuple: Self::Item, _is_new: bool) {
		self.push(tuple);
	}
}	


/// A trie with owned data that may be pushed into. 
impl<K:Ord+Clone> TrieStorage for Vec<(K, isize)> {
	type Item = (K, isize);
	fn new() -> Self { vec![] }
	fn with_capacity(other1: &Self, other2: &Self) -> Self { 
		Vec::with_capacity(other1.len() + other2.len()) 
	}
	fn keys(&self) -> usize { self.len() }
	fn tuples(&self) -> usize { self.len() }
	fn extend_trie(&mut self, other: &Self, lower: usize, upper: usize) {
		debug_assert!(lower < upper);
		self.reserve(upper - lower);
		self.extend_from_slice(&other[lower .. upper]);

	// unsafe {
	//     let position = self.len();
	//     let slice = &other[lower .. upper];
	//     ::std::ptr::copy_nonoverlapping(other.as_ptr(), self.as_mut_ptr().offset(position as isize), slice.len());
	//     self.set_len(position + slice.len());
	// }
	}
	fn extend_merge(&mut self, other1: (&Self, usize, usize), other2: (&Self, usize, usize)) {

		let (vec1, mut lower1, upper1) = other1;
		let (vec2, mut lower2, upper2) = other2;

		// perhaps overly aggressive
		self.reserve(upper1 + upper2 - lower1 - lower2);

		while lower1 < upper1 && lower2 < upper2 {
			match vec1[lower1].0.cmp(&vec2[lower2].0) {
				::std::cmp::Ordering::Less => {
					let step = advance(&vec1[lower1..upper1], |x| x.0 < vec2[lower2].0);
					assert!(step > 0);
					self.extend_trie(&vec1, lower1, lower1 + step);
					lower1 += step;
				}
				::std::cmp::Ordering::Equal => {
					let count = vec1[lower1].1 + vec2[lower2].1;
					if count != 0 {
						self.push((vec1[lower1].0.clone(), count));
					}
					lower1 += 1;
					lower2 += 1;
				}
				::std::cmp::Ordering::Greater => {
					let step = advance(&vec2[lower2..upper2], |x| x.0 < vec1[lower1].0);
					assert!(step > 0);
					self.extend_trie(&vec2, lower2, lower2 + step);
					lower2 += step;
				}
			}
		}

		if lower1 < upper1 { self.extend_trie(&vec1, lower1, upper1); }
		if lower2 < upper2 { self.extend_trie(&vec2, lower2, upper2); }
	}
	fn extend_tuple(&mut self, tuple: Self::Item, _is_new: bool) {
		self.push(tuple);
	}
}

impl<'a, K:Ord+'a, V:'a> TrieRef<'a> for Vec<(K,V)> {
	type Cursor = SliceCursor<'a,K,V>;
	fn keys_cnt(&self) -> usize { self.len() }
	fn cursor(&'a self, lower: usize, upper: usize) -> Self::Cursor {
		SliceCursor::<'a,K,V>::new(&self[lower .. upper])
	}
}

pub struct SliceCursor<'a, K:Ord+'a, V:'a> {
	index: usize,
	slice: &'a [(K, V)],
}

impl<'a, K:Ord+'a, V:'a> SliceCursor<'a, K, V> {
	pub fn new(slice: &'a [(K, V)]) -> SliceCursor<'a,K,V> {
		SliceCursor {
			index: 0,
			slice: slice,
		}
	}
}

impl<'a, K:Ord+'a, V:'a> Cursor<'a> for SliceCursor<'a,K,V> {
	type Key = K;
	type Val = &'a V;

	fn next(&mut self) -> Option<(&'a Self::Key, Self::Val)> {
		if self.index < self.slice.len() {
			let result = (&self.slice[self.index].0, &self.slice[self.index].1);
			self.index += 1;
			Some(result)
		}
		else {
			None
		}
	}

	#[inline(never)]
	fn seek(&mut self, key: &Self::Key) {
		self.index += advance(&self.slice[self.index ..], |x| &x.0 < key)
	}

	fn peek(&self) -> Option<&'a Self::Key> {
		if self.index < self.slice.len() { Some(&self.slice[self.index].0) } else { None }
	}
	fn size(&self) -> usize {
		self.slice.len() - self.index
	}
}

impl<'a, K:Ord+'a, V:'a> Clone for SliceCursor<'a,K,V> {
	fn clone(&self) -> Self {
		SliceCursor::<'a,K,V> {
			index: self.index,
			slice: self.slice,
		}
	}
}

/// Reports the number of elements satisfing the predicate.
///
/// This methods *relies strongly* on the assumption that the predicate
/// stays false once it becomes false, a joint property of the predicate
/// and the slice. This allows `advance` to use exponential search to 
/// count the number of elements in time logarithmic in the result.
#[inline(never)]
pub fn advance<T, F: Fn(&T)->bool>(slice: &[T], function: F) -> usize {

	// start with no advance
	let mut index = 0;
	if index < slice.len() && function(&slice[index]) {

		// advance in exponentially growing steps.
		let mut step = 1;
		while index + step < slice.len() && function(&slice[index + step]) {
			index += step;
			step = step << 1;
		}

		// advance in exponentially shrinking steps.
		step = step >> 1;
		while step > 0 {
			if index + step < slice.len() && function(&slice[index + step]) {
				index += step;
			}
			step = step >> 1;
		}

		index += 1;
	}	

	index
}