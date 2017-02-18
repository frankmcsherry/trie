use trie::Cursor;

/// A cursor-like merge of several cursors.
pub struct CursorMerger<'a, C: Cursor<'a>> {
	// pairs of data and cursors, ordered by `C::Key`.
	pub cursors: Vec<((&'a C::Key, C::Val), C)>,
}

/// A view of merged results, reflecting a key and items with this key.
///
/// The view holds a mut reference for the `CursorMerger`, and must be dropped
/// before the merger may be advanced again. This is because the view uses the
/// storage of the merger to avoid allocating and copying its iterated values.
pub struct CursorView<'a, 'b, C> where 'a: 'b, C: Cursor<'a>+'b {
	pos: usize,
	len: usize,
	merger: &'b mut CursorMerger<'a, C>,
}

impl<'a, 'b, C> CursorView<'a, 'b, C> where 'a: 'b, C: Cursor<'a>+'b {
	/// Returns the key being merged, unless all elements have been consumed.
	pub fn key(&self) -> Option<&'a C::Key> {
		if self.pos < self.len {
			Some(&(self.merger.cursors[self.pos].0).0)
		}
		else {
			None
		}
	}
	/// Returns the number of remaining elements in the merge.
	pub fn len(&self) -> usize {
		self.len - self.pos
	}
}

impl<'a, 'b, C> Iterator for CursorView<'a, 'b, C> where 'a: 'b, C: Cursor<'a>+'b {
	type Item = C::Val;
	fn next(&mut self) -> Option<Self::Item> {
		if self.pos < self.len {
			if let Some(next) = self.merger.cursors[self.pos].1.next() {
				self.pos += 1;
				Some(::std::mem::replace(&mut self.merger.cursors[self.pos-1].0, next).1)
			}
			else {
				self.len -= 1;
				Some((self.merger.cursors.remove(self.pos).0).1)
			}
		}
		else {
			None
		}
	}
}

impl<'a, 'b, C> Drop for CursorView<'a, 'b, C> where 'a: 'b, C: Cursor<'a>+'b {
	fn drop(&mut self) {
		while self.pos < self.len {
			if let Some(next) = self.merger.cursors[self.pos].1.next() {
				self.pos += 1;
				self.merger.cursors[self.pos-1].0 = next;
			}
			else {
				self.len -= 1;
				self.merger.cursors.remove(self.pos);
			}
		}

		self.merger.re_sort(self.len);
	}
}


impl<'a, C: Cursor<'a>> CursorMerger<'a, C> {
	/// Creates a new, empty CursorMerger.
	pub fn new() -> Self { CursorMerger::<'a, C> { cursors: vec![] } }

	/// Returns a view over the data of the next key, if any, and advances the cursor.
	pub fn next<'b>(&'b mut self) -> Option<CursorView<'a, 'b, C>> {
		if self.cursors.len() > 0 {
			let mut count = 1;
			while count < self.cursors.len() && &(self.cursors[0].0).0 == &(self.cursors[count].0).0 {
				count += 1;
			}
			Some(CursorView { pos : 0, len: count, merger: self })
		}
		else {
			None
		}
	}

	/// Advances the CursorMerger to the first key at least as large as `key`.
	#[inline(never)]
	pub fn seek(&mut self, key: &C::Key) {
		// call seek until we don't have to
		let mut cursor = 0;
		while cursor < self.cursors.len() && (self.cursors[cursor].0).0 < key {

			// must advance cursor, pop next.
			self.cursors[cursor].1.seek(key);
			if let Some(next) = self.cursors[cursor].1.next() {
				self.cursors[cursor].0 = next;
				cursor += 1;
			}
			else {
				self.cursors.remove(cursor);
			}
		}

		self.re_sort(cursor);

		debug_assert!(self.cursors.len() == 0 || (self.cursors[0].0).0 >= key);
	}

	/// Reveals the next key, if one exists.
	pub fn peek(&mut self) -> Option<&'a C::Key> {
		if self.cursors.len() > 0 {
			Some((&self.cursors[0].0).0)
		}
		else {
			None
		}
	}

	/// Clears the CursorMerger.
	pub fn clear(&mut self) {
		self.cursors.clear();
	}

	/// Refills a CursorMerger from an iterator of Cursors, re-using allocated memory.
	pub fn refill_from<I: Iterator<Item=C>>(&mut self, iterator: I) {
		self.cursors.clear();
		for mut item in iterator {
			if let Some(next) = item.next() {
				self.cursors.push((next, item));
			}
		}
		self.cursors.sort_by(|x,y| (x.0).0.cmp(&(y.0).0));		
	}

	/// Constructs a new CursorMerger from a iterator of Cursors.
	pub fn from<I: Iterator<Item=C>>(iterator: I) -> Self {
		let mut result = Self::new();
		result.refill_from(iterator);
		result
	}

	pub fn push(&mut self, mut cursor: C) {
		if let Some(next) = cursor.next() {
			self.cursors.push((next, cursor));
		}
	}

	// internal method for ensuring order invariant.
	fn re_sort(&mut self, until: usize) {
		if until > 0 {
			// determine how large a prefix to re-sort
			let count = {
				let mut max_key = &(self.cursors[0].0).0; 
				for index in 1..until {
					if (self.cursors[index].0).0.gt(max_key) {
						max_key = &(self.cursors[index].0).0;
					}
				}
				// scan prefix of old keys until we pass it ...
				let mut count = until;
				while count < self.cursors.len() && (self.cursors[count].0).0.le(max_key) {
					count += 1;
				}

				count
			};

			// sort that prefix
			self.cursors[..count].sort_by(|x,y| (x.0).0.cmp(&(y.0).0));

			for index in 1 .. self.cursors.len() {
				debug_assert!((self.cursors[index-1].0).0 <= (self.cursors[index].0).0);
			}
		}
	}
}

