
pub trait TrieLayer {
	type Iterator: Iterator;
	fn iterate_range(&self, lower: usize, upper: usize) -> Self::Iterator;
}

pub struct TrieStruct<K: Ord, L> where for<'a> &'a L: TrieLayer {
	keys: Vec<(K, usize)>,
	vals: L,
}

impl<'a, K: Ord+'a, L: TrieLayer+'a> TrieLayer for TrieStruct<K,L> where for<'a> &'a L: TrieLayer {
	type Iterator = TrieIter<'a, K, L>;
	fn iterate_range(&self, lower: usize, upper: usize) -> Self::Iterator {
		TrieIter {
			lower: lower, 
			upper: upper,
			trie: &*self,
		}
	}
}

/// An iterator over pairs `(&K, TIter<'a,K,T,V>)`.
pub struct TrieIter<'a, K:Ord+'a, L:'a> where for<'a> &'a L: TrieLayer {
	pub lower: usize,
	pub upper: usize,
	pub trie: &'a Trie<K,L>,
}

impl<'a, K:Ord+'a, L> Iterator for TrieIter<'a,K,L> where for<'a> &'a L: TrieLayer {
	type Item = (&'a K, <&'a L as TrieLayer>::Iterator);
	fn next(&mut self) -> Option<Self::Item> {
		if self.lower < self.upper {
			let current = self.lower;
			self.lower += 1;

			let t_lower = if current == 0 { 0 } else { self.trie.keys[current-1].1 };
			let t_upper = self.trie.keys[current].1;

			Some((
				&self.trie.keys[current].0,
				self.trie.vals.iterate_range(t_lower, t_upper),
			))
		}
		else {
			None
		}
	}
}

impl<'a, K: Ord+'a> TrieLayer for &'a Vec<(K, isize)> {
	type Iterator = VecIter<'a, K>;
	fn iterate_range(&self, lower: usize, upper: usize) -> Self::Iterator {
		VecIter {
			lower: lower,
			upper: upper,
			vec: &*self,
		}
	}
}

/// An iterator over pairs `(&K, TIter<'a,K,T,V>)`.
pub struct VecIter<'a, K:Ord+'a> {
	pub lower: usize,
	pub upper: usize,
	pub vec: &'a Vec<(K,isize)>,
}

impl<'a, K:Ord+'a> Iterator for VecIter<'a,K> {
	type Item = (&'a K, isize);
	fn next(&mut self) -> Option<Self::Item> {
		if self.lower < self.upper {
			let current = self.lower;
			self.lower += 1;

			Some((
				&self.vec[current].0,
				self.vec[current].1,
			))
		}
		else {
			None
		}
	}
}
