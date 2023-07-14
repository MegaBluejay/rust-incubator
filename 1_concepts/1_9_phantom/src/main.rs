use std::{marker::PhantomData, sync::atomic::AtomicPtr};

use rand::Rng;

struct Fact<T: ?Sized>(PhantomData<AtomicPtr<*const T>>);

trait KnownFacts {
    const FACTS: &'static [&'static str];
}

impl<T: KnownFacts + ?Sized> Fact<T> {
    fn new() -> Self {
        Self(PhantomData)
    }

    fn fact(&self) -> &str {
        T::FACTS[rand::thread_rng().gen_range(0..T::FACTS.len())]
    }
}

impl<T> KnownFacts for Vec<T> {
    const FACTS: &'static [&'static str] =
        &["Vec is heap-allocated", "Vec may re-allocate on growing"];
}

fn main() {
    let fvec: Fact<Vec<i32>> = Fact::new();
    println!("{}", fvec.fact());
}
