use trie::Cursor;

// /// A structure for merging Iterators over `(Key, Val)` pairs where `Key: Ord`.
// pub struct Merger<K:Ord, V, Z: Iterator<Item=(K,V)>> {
// 	// This structure ensures that there is always a valid Z::Item, 
// 	// which helps us avoid a great many is_some() checks when we
// 	// manipulate elements in the list. The main annoyance is that 
// 	// when an iterator expires, we have to immediately remove it.
// 	// 
// 	// The list is maintained in order of the key, so that we work
// 	// with a prefix of the list at all times. 
// 	pub iters: Vec<(Z::Item, Z)>,
// }

// /// A guard that appears as a &[(Z::Item, Z)] but whose Drop advances the iterator.
// pub struct MergerGuard<'a, K:Ord+'a, V:'a, Z: Iterator<Item=(K,V)>+'a> {
// 	count: usize,
// 	iters: &'a mut Vec<(Z::Item, Z)>,
// }

// impl<'a, K:Ord+'a, V:'a, Z: Iterator<Item=(K,V)>+'a> ::std::ops::Deref for MergerGuard<'a,K,V,Z> {
// 	type Target = [(Z::Item, Z)];
// 	fn deref(&self) -> &Self::Target {
// 		&self.iters[..self.count]
// 	}
// }

// impl<'a, K:Ord+'a, V:'a, Z: Iterator<Item=(K,V)>+'a> Drop for MergerGuard<'a,K,V,Z> {
// 	fn drop(&mut self) {
// 		if self.iters.len() == 1 {
// 			if let Some(next) = self.iters[0].1.next() {
// 				self.iters[0].0 = next;
// 			}
// 			else {
// 				self.iters.pop();
// 			}
// 		}
// 		else {

// 			let mut finger = 0;
// 			let mut cursor = self.count;
// 			while finger < cursor {
// 				if let Some(next) = self.iters[finger].1.next() {
// 					self.iters[finger].0 = next;
// 					finger += 1;
// 				}
// 				else {
// 					self.iters.remove(finger);
// 					cursor -= 1;
// 				}
// 			}

// 			// sort the disordered elements
// 			if cursor > 0 {

// 				// determine how large a prefix to sort
// 				let count = {
// 					let mut max_key = &(self.iters[0].0).0; 
// 					for index in 1..cursor {
// 						if (self.iters[index].0).0.gt(max_key) {
// 							max_key = &(self.iters[index].0).0;
// 						}
// 					}
// 					// scan prefix of old keys until we pass it ...
// 					let mut count = cursor;
// 					while count < self.iters.len() && (self.iters[count].0).0.le(max_key) {
// 						count += 1;
// 					}

// 					count
// 				};

// 				// sort that prefix
// 				self.iters[..count].sort_by(|x,y| (x.0).0.cmp(&(y.0).0));
// 			}
// 		}
// 	}
// }

// impl<K:Ord, V, Z: Iterator<Item=(K,V)>> Merger<K,V,Z> {
// 	/// Returns an `Option<&[(Z::Item, Z)]>` slice, or something like it.
// 	///
// 	/// This reveals the iterators as well as the items, which is too bad but
// 	/// how it needs to be for the moment. I suppose instead of pairs they could
// 	/// be something that implements Deref for just the key ...
// 	///
// 	/// Also, perhaps this should find some way to give up ownership of the items?
// 	/// It isn't needed for the use cases, but worth considering. Hard to do though.
// 	pub fn next<'a>(&'a mut self) -> Option<MergerGuard<'a,K,V,Z>> {
// 		if self.iters.len() == 1 {
// 			Some(MergerGuard { 
// 				count: 1,
// 				iters: &mut self.iters,
// 			})
// 		}
// 		else if self.iters.len() > 1 {
// 			let mut cursor = 1;
// 			while cursor < self.iters.len() && (self.iters[cursor].0).0 == (self.iters[0].0).0 {
// 				cursor += 1;
// 			}
// 			Some(MergerGuard {
// 				count: cursor,
// 				iters: &mut self.iters,
// 			})
// 		}
// 		else {
// 			None
// 		}
// 	}

// 	/// Adds an iterator to the list of items.
// 	///
// 	/// This should only be used as part of re-loading a Merger. It does not maintain 
// 	/// the invariant that the iterators are sorted. I suppose it could, because this 
// 	/// isn't meant to be the fast path, but ... Hrm.
// 	pub fn push(&mut self, mut iter: Z) {
// 		if let Some(next) = iter.next() {
// 			self.iters.push((next, iter));
// 		}
// 	}
// 	/// Sorts iterators by the key of the first item.
// 	///
// 	pub fn sort(&mut self) {
// 		self.iters.sort_by(|x,y| (x.0).0.cmp(&(y.0).0))
// 	}
// }


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

	pub fn from<I: Iterator<Item=C>>(iterator: I) -> Self {
		let mut result = CursorMerger::<'a, C>::new();
		for mut item in iterator {
			if let Some(next) = item.next() {
				result.cursors.push((next, item));
			}
		}
		result.cursors.sort_by(|x,y| (x.0).0.cmp(&(y.0).0));		
		result
	}

	// pub fn push(&'a mut self, mut cursor: C) {
	// 	if let Some(next) = cursor.next() {
	// 		self.cursors.push((next, cursor));
	// 	}
	// }
	// pub fn sort(&'a mut self) {
	// 	self.cursors.sort_by(|x,y| (x.0).0.cmp(&(y.0).0));
	// }
}

