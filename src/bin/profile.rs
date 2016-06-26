extern crate trie;

use trie::arbor::Arbor;
use trie::trie::TrieLayer;
use trie::merge::CursorMerger;

fn main() {

    if ::std::env::args().count() != 4 { 
        println!("tests trie building and scanning with a bogus graph structure");
        println!("usage: <nodes> <degree> <batch_size>"); 
    }
    else {
        let nodes: usize = ::std::env::args().nth(1).unwrap().parse().unwrap();
        let degree: usize = ::std::env::args().nth(2).unwrap().parse().unwrap();
        let batch: usize = ::std::env::args().nth(3).unwrap().parse().unwrap();

        println!("running with nodes: {}, degree: {}, batch: {}", nodes, degree, batch);

        test_arbor(nodes, degree, batch);
    }
}

fn test_arbor(nodes: usize, degree: usize, batch: usize) {

    let timer = ::std::time::Instant::now();

    let mut arbor_forward = Arbor::<TrieLayer<u32, Vec<(u32, i32)>>>::new();
    let mut arbor_reverse = Arbor::<TrieLayer<u32, Vec<(u32, i32)>>>::new();

    let mut forward = Vec::new();
    let mut reverse = Vec::new();

    for node in 0 .. nodes {
        for edge in 0 .. degree {
            forward.push((node as u32, (((node + edge) % nodes)  as u32, 1)));
            reverse.push((((node + edge) % nodes) as u32, (node as u32, 1)));
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
    println!("loading elapsed: {:?}", timer.elapsed());

    let mut count = 0;
    let mut merger = CursorMerger::new();
    let mut cursor = arbor_forward.cursor();
    while let Some(slice) = cursor.next() {
        merger.refill_from(slice.iter().map(|&((_,ref x),_)| (*x).clone()));
        while let Some(thing) = merger.next() {
            count += thing.len();
        }
    }

    assert_eq!(count, nodes * degree);
    println!("forward elapsed: {:?}", timer.elapsed());

    let mut count = 0;
    let mut merger = CursorMerger::new();
    let mut cursor = arbor_reverse.cursor();
    while let Some(slice) = cursor.next() {
        merger.refill_from(slice.iter().map(|&((_,ref x),_)| (*x).clone()));
        while let Some(thing) = merger.next() {
            count += thing.len();
        }
    }

    assert_eq!(count, nodes * degree);
    println!("reverse elapsed: {:?}", timer.elapsed());
}