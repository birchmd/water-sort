use {
    rand::Rng,
    std::{collections::HashMap, thread::JoinHandle},
    water_sort_core::{
        puzzle::{Vial, shuffle_state},
        solver,
    },
};

// Simulation config
const THREADS: usize = 20;
const BATCH_SIZE: usize = 10_000;

// Puzzle config
const N: usize = 5;
const C: usize = 4;
const K: usize = 2;
const T: usize = C + K;

fn main() {
    let mut hist: HashMap<usize, usize> = HashMap::new();
    let handles: Vec<JoinHandle<HashMap<usize, usize>>> =
        std::iter::repeat_with(|| std::thread::spawn(sim_batch))
            .take(THREADS)
            .collect();
    for handle in handles {
        match handle.join() {
            Ok(result) => {
                for (x, y) in result {
                    *hist.entry(x).or_default() += y;
                }
            }
            Err(e) => println!("Warn: join failed {e:?}"),
        }
    }
    let mut output: Vec<(usize, usize)> = hist.into_iter().collect();
    output.sort_unstable();
    for (x, y) in output {
        println!("{x} {y}");
    }
}

fn sim_batch() -> HashMap<usize, usize> {
    let mut rng = rand::rng();
    let mut hist: HashMap<usize, usize> = HashMap::new();
    for _ in 0..BATCH_SIZE {
        let n_moves = sim_single(&mut rng).unwrap_or_default();
        *hist.entry(n_moves).or_default() += 1;
    }
    hist
}

fn sim_single<R: Rng>(rng: &mut R) -> Option<usize> {
    let mut state = [Vial::new([0; N]); T];
    shuffle_state::<N, T, C, R>(&mut state, rng);
    solver::solve::<N, T, C>(state)
}
