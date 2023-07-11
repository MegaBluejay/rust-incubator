use gstack::GlobalStack;

mod gstack {
    use std::{cell::RefCell, rc::Rc};

    pub struct GlobalStack<T>(Rc<RefCell<Vec<T>>>);

    impl<T> GlobalStack<T> {
        pub fn new() -> Self {
            Self(Rc::new(RefCell::new(vec![])))
        }

        pub fn push(&self, item: T) {
            self.0.borrow_mut().push(item)
        }

        pub fn pop(&self) -> Option<T> {
            self.0.borrow_mut().pop()
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
    let (stack1, stack2) = (&stack, &stack);
    stack1.push(1);
    stack2.push(2);
    println!("{:?}", stack1.pop());
}
