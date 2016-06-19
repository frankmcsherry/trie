extern crate rand;
extern crate trie;
extern crate graph_map;

// use trie::merge::Merger;
use trie::arbor::Arbor;

use trie::trie::{TrieLayer, TrieRef, TrieStorage};
use trie::merge::CursorMerger;

use graph_map::GraphMMap;

fn main() {

    test_arbor(100);

}

fn test_arbor(batch: usize) {

    let timer = ::std::time::Instant::now();

    let filename = std::env::args().nth(1).unwrap();
    let graph = GraphMMap::new(&filename);

	  let mut arbor_forward = Arbor::<TrieLayer<u32, Vec<(u32, i32)>>>::new();
	  let mut arbor_reverse = Arbor::<TrieLayer<u32, Vec<(u32, i32)>>>::new();

    let mut forward = Vec::new();
    let mut reverse = Vec::new();

    for node in 0 .. graph.nodes() {
    		for &edge in graph.edges(node) {
            forward.push((node as u32, (edge, 1)));
            reverse.push((edge, (node as u32, 1)));
       	}

       	if node % batch == (batch - 1) {
       		forward.sort();
       		reverse.sort();
       		arbor_forward.extend_ordered(forward.drain(..));
       		arbor_reverse.extend_ordered(reverse.drain(..));
       	}
    }

  	forward.sort();
  	reverse.sort();
  	arbor_forward.extend_ordered(forward.drain(..));
  	arbor_reverse.extend_ordered(reverse.drain(..));
  	println!("elapsed: {:?}", timer.elapsed());

    // let mut cursor = arbor_forward.cursor();
    // while let Some(something) = cursor.next() {

    // }

    let mut x = Vec::<(u32,i32)>::new();
    let mut c = (&x).cursor(0,0);

    let mut y = TrieLayer::<u32, Vec<(u32,i32)>>::new();
    // let mut c = (&y).cursor(0,0);

    let arbor_test = Arbor::<Vec<(u32,i32)>>::new();

    let z = arbor_test.cursor();

    // let mut z = CursorMerger::new();
    // z.push((&y).cursor(0,0));
    // z.sort();

}