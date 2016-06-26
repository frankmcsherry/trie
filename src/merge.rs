use trie::Cursor;

pub struct CursorMerger<'a, C: Cursor<'a>> {
	started: Option<usize>,
	cursors: Vec<((&'a C::Key, C::Val), C)>,
}

impl<'a, C: Cursor<'a>> CursorMerger<'a, C> {
	pub fn new() -> Self { CursorMerger::<'a, C> { started: None, cursors: vec![] } }

	pub fn next(&mut self) -> Option<&[((&'a C::Key, C::Val), C)]> {

		// if started; advance cursor things
		if let Some(previous) = self.started {

			// advance each of self.cursors[0 .. previous]. discard empties.
			let mut finger = 0;
			let mut cursor = previous;
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

				// determine how large a prefix to sort
				let count = {
					let mut max_key = &(self.cursors[0].0).0; 
					for index in 1..cursor {
						if (self.cursors[index].0).0.gt(max_key) {
							max_key = &(self.cursors[index].0).0;
						}
					}
					// scan prefix of old keys until we pass it ...
					let mut count = cursor;
					while count < self.cursors.len() && (self.cursors[count].0).0.le(max_key) {
						count += 1;
					}

					count
				};

				// sort that prefix
				self.cursors[..count].sort_by(|x,y| (x.0).0.cmp(&(y.0).0));
			}
		}

		if self.cursors.len() > 0 {

			let mut count = 1;
			while count < self.cursors.len() && (self.cursors[0].0).0 == (self.cursors[count].0).0 {
				count += 1;
			}
			self.started = Some(count);
			Some(&self.cursors[..count])
		}
		else {
			None
		}
	}

	/// Constructs a 
	pub fn refill_from<I: Iterator<Item=C>>(&mut self, iterator: I) {
		self.started = None;
		self.cursors.clear();
		for mut item in iterator {
			if let Some(next) = item.next() {
				self.cursors.push((next, item));
			}
		}
		self.cursors.sort_by(|x,y| (x.0).0.cmp(&(y.0).0));		
	}

	pub fn from<I: Iterator<Item=C>>(iterator: I) -> Self {
		let mut result = Self::new();
		result.refill_from(iterator);
		result
	}

}

