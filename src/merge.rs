use trie::Cursor;

pub struct CursorMerger<'a, C: Cursor<'a>> {
	// indicates a prefix length of self.cursors to consume.
	consume: Option<usize>,
	// pairs of data and cursors, ordered by `C::Key`.
	cursors: Vec<((&'a C::Key, C::Val), C)>,
}

impl<'a, C: Cursor<'a>> CursorMerger<'a, C> {
	/// Creates a new, empty CursorMerger.
	pub fn new() -> Self { CursorMerger::<'a, C> { consume: None, cursors: vec![] } }

	/// Advances the CursorMerger revealing values with like keys.
	pub fn next(&mut self) -> Option<&[((&'a C::Key, C::Val), C)]> {
		self.advance();
		if self.cursors.len() > 0 {
			let mut count = 1;
			while count < self.cursors.len() && (self.cursors[0].0).0 == (self.cursors[count].0).0 {
				count += 1;
			}
			self.consume = Some(count);
			Some(&self.cursors[..count])
		}
		else {
			None
		}
	}

	/// Advances the CursorMerger to the first key at least as large as `key`.
	pub fn seek(&mut self, key: &C::Key) {
		self.advance();

		// call seek until we don't have to
		let mut cursor = 0;
		while (self.cursors[cursor].0).0 <= key {
			self.cursors[cursor].1.seek(key);
			cursor += 1;
		}

		self.re_sort(cursor);
	}

	/// Reveals the next key, if one exists.
	pub fn peek(&mut self) -> Option<&'a C::Key> {
		self.advance();
		if self.cursors.len() > 0 {
			Some((&self.cursors[0].0).0)
		}
		else {
			None
		}
	}

	/// Clears the CursorMerger.
	pub fn clear(&mut self) {
		self.consume = None;
		self.cursors.clear();
	}

	/// Refills a CursorMerger from an iterator of Cursors, re-using allocated memory.
	pub fn refill_from<I: Iterator<Item=C>>(&mut self, iterator: I) {
		self.consume = None;
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

	// internal method for ensuring order invariant.
	fn re_sort(&mut self, until: usize) {
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
	}

	// internal method for advancing cursors.
	fn advance(&mut self) {
		if let Some(steps) = self.consume {

			// advance each of self.cursors[0 .. steps]. discard empties.
			let mut finger = 0;
			let mut cursor = steps;
			while finger < cursor {
				if let Some(next) = self.cursors[finger].1.next() {
					self.cursors[finger].0 = next;
					finger += 1;
				}
				else {
					self.cursors.remove(finger);
					cursor -= 1;
				}
			}

			// sort the disordered elements
			if cursor > 0 {
				self.re_sort(cursor);
			}

			self.consume = None;
		}
	}
}

