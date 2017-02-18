extern crate rand;
extern crate trie;

use rand::{Rng, SeedableRng, StdRng};

use trie::arbor::Arbor;
use trie::trie::TrieLayer;
// use trie::merge::CursorMerger;

fn main() {

    let keys: u64 = ::std::env::args().nth(1).unwrap().parse().unwrap();
    let vals: u64 = ::std::env::args().nth(2).unwrap().parse().unwrap();
    let batch: u64 = ::std::env::args().nth(3).unwrap().parse().unwrap();
    let waves: u64 = ::std::env::args().nth(4).unwrap().parse().unwrap();

    let mut trace = Arbor::<TrieLayer<u64, TrieLayer<u64, Vec<(u64, isize)>>>>::new();

    let mut counter = 0;
    let mut buffer = Vec::new();

    for round in 0u64 .. {

        for _wave in 0u64 .. waves {

            for x in counter .. counter + batch {
                buffer.push((x % keys, (x % vals, (x, 1))));
            }

            counter += batch;
            buffer.sort();
            trace.extend_ordered(buffer.drain(..));
        }

        let seed: &[_] = &[1, 2, 3, 4];
        let mut rng: StdRng = SeedableRng::from_seed(seed);    // rng for edge additions

        let mut queries = Vec::with_capacity(batch as usize);
        for _ in 0 .. batch {
            queries.push(rng.gen_range(0, keys));
        }

        let timer = ::std::time::Instant::now();

        queries.sort();

        let mut count = 0;

        let mut cursor = trace.cursor();
        for query in queries.drain(..) {
            cursor.seek(&query);
            if cursor.peek().map(|x| x == &query).unwrap_or(false) { 
                count += 1; 
            }
        }

        let elapsed = timer.elapsed();
        let rate = (batch as f64) / (elapsed.as_secs() as f64 + (elapsed.subsec_nanos() as f64 / 1000000000.0f64));
        println!("throughput: {:.*} lookups/sec @ {:?} elts ({})", 2, rate, (round + 1) * waves * batch, count);
    }
}