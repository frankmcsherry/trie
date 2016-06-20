// extern crate rand;
extern crate trie;
// extern crate graph_map;

// use trie::merge::Merger;
use trie::arbor::Arbor;
use trie::trie::TrieLayer;
use trie::merge::CursorMerger;

// use graph_map::GraphMMap;

fn main() {
    test_arbor(1_000_000, 10, 100);
}

fn test_arbor(nodes: usize, degree: usize, batch: usize) {

    let timer = ::std::time::Instant::now();

    // let filename = std::env::args().nth(1).unwrap();
    // let graph = GraphMMap::new(&filename);

    let mut arbor_forward = Arbor::<TrieLayer<usize, Vec<(usize, i32)>>>::new();
    let mut arbor_reverse = Arbor::<TrieLayer<usize, Vec<(usize, i32)>>>::new();

    let mut forward = Vec::new();
    let mut reverse = Vec::new();

    for node in 0 .. nodes {
        for edge in 0 .. degree {
            forward.push((node, ((node + edge) % nodes, 1)));
            reverse.push(((node + edge) % nodes, (node, 1)));
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

    let mut count = 0;
    let mut cursor = arbor_forward.cursor();
    while let Some(slice) = cursor.next() {
        let mut merger = CursorMerger::from(slice.iter().map(|&((_,ref x),_)| (*x).clone()));

        while let Some(thing) = merger.next() {
            count += thing.len();
        }
    }

    println!("edges: {}", count);
    println!("elapsed: {:?}", timer.elapsed());

    let mut count = 0;
    let mut cursor = arbor_reverse.cursor();
    while let Some(slice) = cursor.next() {
        let mut merger = CursorMerger::from(slice.iter().map(|&((_,ref x),_)| (*x).clone()));

        while let Some(thing) = merger.next() {
            count += thing.len();
        }
    }

    println!("edges: {}", count);
    println!("elapsed: {:?}", timer.elapsed());
}