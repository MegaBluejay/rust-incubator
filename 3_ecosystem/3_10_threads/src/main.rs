use std::thread;

use crossbeam::channel::{unbounded, Receiver, Sender};
use ndarray::Array2;
use ndarray_rand::{rand_distr::Standard, RandomExt};
use rand::thread_rng;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};

fn producer(sender: &Sender<Array2<u8>>) {
    let mut rng = thread_rng();
    while sender
        .send(Array2::random_using((64, 64), Standard, &mut rng))
        .is_ok()
    {}
}

fn consumer(receiver: &Receiver<Array2<u8>>) {
    for matrix in receiver {
        let sum: u32 = matrix.into_par_iter().map(|elem| u32::from(*elem)).sum();
        println!("{sum}");
    }
}

fn main() {
    let (sender, receiver) = unbounded();
    thread::scope(|s| {
        s.spawn(|| producer(&sender));

        for _ in 0..2 {
            s.spawn(|| consumer(&receiver));
        }
    });
}
