#![allow(dead_code)]

mod list {
    use std::{
        iter::FusedIterator,
        sync::{Arc, Mutex},
    };

    type Neigh<T> = Option<Arc<Mutex<Node<T>>>>;

    struct Node<T> {
        item: T,
        prev: Neigh<T>,
        next: Neigh<T>,
    }

    struct Forward;
    struct Back;

    trait GetNeighs<Dir> {
        type Item;

        fn next(&mut self) -> &mut Neigh<Self::Item>;
        fn prev(&mut self) -> &mut Neigh<Self::Item>;
    }

    impl<T> Node<T> {
        fn new(item: T) -> Arc<Mutex<Node<T>>> {
            Arc::new(Mutex::new(Node {
                item,
                prev: None,
                next: None,
            }))
        }
    }

    impl<T> GetNeighs<Forward> for Node<T> {
        type Item = T;

        fn next(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.next
        }

        fn prev(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.prev
        }
    }

    impl<T> GetNeighs<Back> for Node<T> {
        type Item = T;

        fn next(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.prev
        }

        fn prev(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.next
        }
    }

    trait GetEnds<Dir> {
        type Item;

        fn head(&mut self) -> &mut Neigh<Self::Item>;
        fn last(&mut self) -> &mut Neigh<Self::Item>;
    }

    struct Ends<T> {
        head: Neigh<T>,
        last: Neigh<T>,
    }

    impl<T> GetEnds<Forward> for Ends<T> {
        type Item = T;

        fn head(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.head
        }

        fn last(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.last
        }
    }

    impl<T> GetEnds<Back> for Ends<T> {
        type Item = T;

        fn head(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.last
        }

        fn last(&mut self) -> &mut Neigh<Self::Item> {
            &mut self.head
        }
    }

    pub struct List<T>(Mutex<Ends<T>>);

    impl<T> Default for Ends<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> Ends<T> {
        fn new() -> Self {
            Self {
                head: None,
                last: None,
            }
        }

        fn insert_single(&mut self, new: Neigh<T>) {
            self.head = new.clone();
            self.last = new;
        }

        fn push<Dir>(&mut self, item: T)
        where
            Self: GetEnds<Dir, Item = T>,
            Node<T>: GetNeighs<Dir, Item = T>,
        {
            let new_last = Node::new(item);
            if let Some(old_last) = self.last().take() {
                *old_last.lock().unwrap().next() = Some(new_last.clone());
                *new_last.lock().unwrap().prev() = Some(old_last);
            } else {
                *self.head() = Some(new_last.clone());
            }
            *self.last() = Some(new_last);
        }

        fn push_back(&mut self, item: T) {
            self.push::<Forward>(item)
        }

        fn push_front(&mut self, item: T) {
            self.push::<Back>(item)
        }

        fn pop<Dir>(&mut self) -> Option<T>
        where
            Self: GetEnds<Dir, Item = T>,
            Node<T>: GetNeighs<Dir, Item = T>,
        {
            self.last().take().map(|last| {
                *self.last() = if let Some(prev) = last.lock().unwrap().prev().take() {
                    *prev.lock().unwrap().next() = None;
                    Some(prev)
                } else {
                    *self.head() = None;
                    None
                };
                Arc::into_inner(last).unwrap().into_inner().unwrap().item
            })
        }

        fn pop_back(&mut self) -> Option<T> {
            self.pop::<Forward>()
        }

        fn pop_front(&mut self) -> Option<T> {
            self.pop::<Back>()
        }

        fn is_empty(&self) -> bool {
            self.head.is_none()
        }

        fn clear(&mut self) {
            while self.pop_back().is_some() {}
        }
    }

    impl<T> Drop for Ends<T> {
        fn drop(&mut self) {
            self.clear();
        }
    }

    macro_rules! impl_inner {
        ($vis:vis fn $name:ident(&self $(, $i:ident : $t:ty)*) $(-> $res:ty)?) => {
            $vis fn $name(&self $(, $i : $t)*) $(-> $res)? {
                self.0.lock().unwrap().$name($($i ,)*)
            }
        };
    }

    impl<T> Default for List<T> {
        fn default() -> Self {
            Self::new()
        }
    }

    impl<T> List<T> {
        pub fn new() -> Self {
            Self(Mutex::new(Ends::new()))
        }

        impl_inner! { pub fn push_back(&self, item: T) }

        impl_inner! { pub fn push_front(&self, item: T) }

        impl_inner! { pub fn pop_back(&self) -> Option<T> }

        impl_inner! { pub fn pop_front(&self) -> Option<T> }

        impl_inner! { pub fn is_empty(&self) -> bool }

        impl_inner! { pub fn clear(&self) }
    }

    pub struct IntoIter<T>(Ends<T>);

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.pop_front()
        }
    }

    impl<T> DoubleEndedIterator for IntoIter<T> {
        fn next_back(&mut self) -> Option<Self::Item> {
            self.0.pop_back()
        }
    }

    impl<T> FusedIterator for IntoIter<T> {}

    impl<T> IntoIterator for List<T> {
        type Item = T;

        type IntoIter = IntoIter<T>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter(self.0.into_inner().unwrap())
        }
    }

    impl<T> FromIterator<T> for List<T> {
        fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
            let mut ends = Ends::new();
            for item in iter {
                ends.push_back(item);
            }
            Self(Mutex::new(ends))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{collections::HashSet, thread};

    use super::list::*;

    #[test]
    fn back() {
        let list: List<i32> = List::new();
        assert_eq!(list.pop_back(), None);

        list.push_back(1);
        list.push_back(2);

        assert_eq!(list.pop_back(), Some(2));
        assert_eq!(list.pop_back(), Some(1));
        assert_eq!(list.pop_back(), None);
    }

    #[test]
    fn front() {
        let list: List<i32> = List::new();
        assert_eq!(list.pop_front(), None);

        list.push_front(1);
        list.push_front(2);

        assert_eq!(list.pop_front(), Some(2));
        assert_eq!(list.pop_front(), Some(1));
        assert_eq!(list.pop_front(), None);
    }

    #[test]
    fn into_iter() {
        let list: List<i32> = List::new();

        list.push_back(1);
        list.push_back(2);

        assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![1, 2]);
    }

    #[test]
    fn mixed() {
        let list: List<i32> = List::new();

        list.push_back(1);
        list.push_front(2);
        list.push_back(3);
        list.push_front(4);

        assert_eq!(list.into_iter().collect::<Vec<_>>(), vec![4, 2, 1, 3]);
    }

    #[test]
    fn threaded() {
        let list: List<i32> = List::new();

        thread::scope(|s| {
            s.spawn(|| {
                list.push_back(1);
            });

            s.spawn(|| {
                list.push_front(2);
            });

            s.spawn(|| {
                list.push_back(3);
            });

            s.spawn(|| {
                list.push_front(4);
            });
        });

        assert_eq!(
            list.into_iter().collect::<HashSet<_>>(),
            [1, 2, 3, 4].into()
        )
    }
}

fn main() {
    println!("Implement me!");
}
