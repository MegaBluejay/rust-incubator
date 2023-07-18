pub mod my_error;
pub mod my_iterator_ext;

use std::fmt::Display;

pub use self::{my_error::MyError, my_iterator_ext::MyIteratorExt};

#[derive(Debug)]
struct Test;

impl Display for Test {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        unimplemented!()
    }
}

impl MyError for Test {
    fn source(&self) -> Option<&(dyn MyError + 'static)> {
        None
    }

    // fn type_id(&self, _: my_error::private::Token) -> std::any::TypeId
    // where
    //     Self: 'static,
    // {
    //     std::any::TypeId::of::<Self>()
    // }
}

impl Iterator for Test {
    type Item = ();

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

// impl MyIteratorExt for Test {}
