use std::{thread, time::Duration};

use gstack::GlobalStack;

mod gstack {
    use std::sync::{Arc, Mutex};

    pub struct GlobalStack<T>(Arc<Mutex<Vec<T>>>);

    impl<T> GlobalStack<T> {
        pub fn new() -> Self {
            Self(Arc::new(Mutex::new(vec![])))
        }
        pub fn push(&self, item: T) {
            self.0.lock().unwrap().push(item)
        }

        pub fn pop(&self) -> Option<T> {
            self.0.lock().unwrap().pop()
        }
    }

    impl<T> Clone for GlobalStack<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }
}

fn main() {
    let stack = GlobalStack::new();
    let thread_stack = stack.clone();
    thread::spawn(move || {
        thread_stack.push(2);
    });
    stack.push(1);
    thread::sleep(Duration::from_secs(1));
    println!("{:?}, {:?}", stack.pop(), stack.pop());
}
