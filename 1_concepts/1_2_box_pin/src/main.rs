use std::{
    future::Future,
    pin::Pin,
    rc::Rc,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use futures::executor::block_on;
use futures_timer::Delay;
use pin_project::pin_project;

mod special {
    use std::{fmt::Debug, pin::Pin, rc::Rc};

    pub trait SayHi: Debug {
        fn say_hi(self: Pin<&Self>) {
            println!("Hi from {:?}", self);
        }
    }

    impl<T: Debug> SayHi for Box<T> {}
    impl<T: Debug> SayHi for Rc<T> {}
    impl<T: Debug> SayHi for Vec<T> {}
    impl SayHi for String {}
    impl SayHi for &[u8] {}

    pub trait MutMeSomehow {
        fn mut_me_somehow(self: Pin<&mut Self>);
    }

    impl<T: ?Sized + Default> MutMeSomehow for Box<T> {
        fn mut_me_somehow(mut self: Pin<&mut Self>) {
            *self = Default::default();
        }
    }

    impl<T> MutMeSomehow for Rc<T> {
        fn mut_me_somehow(mut self: Pin<&mut Self>) {
            let _ = Rc::get_mut(&mut *self).expect("other rcs exist");
        }
    }

    impl<T> MutMeSomehow for Vec<T> {
        fn mut_me_somehow(self: Pin<&mut Self>) {
            let xs = unsafe { self.get_unchecked_mut() };
            xs.swap(0, 1);
        }
    }

    impl MutMeSomehow for String {
        fn mut_me_somehow(mut self: Pin<&mut Self>) {
            self.make_ascii_uppercase();
        }
    }

    impl MutMeSomehow for &[u8] {
        fn mut_me_somehow(mut self: Pin<&mut Self>) {
            *self = &[1, 2, 3];
        }
    }
}

mod any {
    use std::{fmt::Debug, pin::Pin};

    pub trait SayHi: Debug {
        fn say_hi(self: Pin<&Self>) {
            println!("Hi from {:?}", self);
        }
    }

    pub trait MutMeSomehow {
        fn mut_me_somehow(self: Pin<&mut Self>);
    }

    impl<T: Debug> SayHi for T {}

    impl<T: Default> MutMeSomehow for T {
        fn mut_me_somehow(mut self: Pin<&mut Self>) {
            self.set(Default::default());
        }
    }
}

#[pin_project]
struct MeasurableFuture<Fut> {
    #[pin]
    inner_future: Fut,
    started_at: Option<Instant>,
}

impl<Fut: Future> MeasurableFuture<Fut> {
    pub fn new(inner_future: Fut) -> Self {
        Self {
            inner_future,
            started_at: None,
        }
    }
}

impl<Fut: Future> Future for MeasurableFuture<Fut> {
    type Output = Fut::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let start = this.started_at.get_or_insert_with(Instant::now);
        this.inner_future.poll(cx).map(|res| {
            eprintln!("{}", start.elapsed().as_nanos());
            res
        })
    }
}

fn main() {
    let mut xs = vec![1, 2];
    special::MutMeSomehow::mut_me_somehow(Pin::new(&mut xs));
    special::SayHi::say_hi(Pin::new(&xs));

    let mut s = "abc".to_string();
    special::MutMeSomehow::mut_me_somehow(Pin::new(&mut s));
    special::SayHi::say_hi(Pin::new(&s));

    let mut bs = &[][..];
    special::MutMeSomehow::mut_me_somehow(Pin::new(&mut bs));
    special::SayHi::say_hi(Pin::new(&bs));

    let mut rc = Rc::new(1);
    special::MutMeSomehow::mut_me_somehow(Pin::new(&mut rc));
    special::SayHi::say_hi(Pin::new(&rc));

    let mut bx = Box::new(2);
    special::MutMeSomehow::mut_me_somehow(Pin::new(&mut bx));
    special::SayHi::say_hi(Pin::new(&bx));

    let mut x = 3;
    any::MutMeSomehow::mut_me_somehow(Pin::new(&mut x));
    any::SayHi::say_hi(Pin::new(&x));

    block_on(MeasurableFuture::new(Delay::new(Duration::from_secs(1))));
}
