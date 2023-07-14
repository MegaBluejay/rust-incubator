#![allow(dead_code)]

mod list {
    use std::sync::{Arc, Mutex};

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

    impl<T> List<T> {
        pub fn new() -> Self {
            List(Mutex::new(Ends {
                head: None,
                last: None,
            }))
        }

        fn insert_single(this: &mut Ends<T>, new: Neigh<T>) {
            this.head = new.clone();
            this.last = new;
        }

        fn push<Dir>(&self, item: T)
        where
            Ends<T>: GetEnds<Dir, Item = T>,
            Node<T>: GetNeighs<Dir, Item = T>,
        {
            let mut this = self.0.lock().unwrap();
            let new_last = Node::new(item);
            if let Some(old_last) = this.last().take() {
                *old_last.lock().unwrap().next() = Some(new_last.clone());
                *new_last.lock().unwrap().prev() = Some(old_last);
            } else {
                *this.head() = Some(new_last.clone());
            }
            *this.last() = Some(new_last);
        }

        pub fn push_back(&self, item: T) {
            self.push::<Forward>(item);
        }

        pub fn push_front(&self, item: T) {
            self.push::<Back>(item);
        }

        fn pop<Dir>(&self) -> Option<T>
        where
            Ends<T>: GetEnds<Dir, Item = T>,
            Node<T>: GetNeighs<Dir, Item = T>,
        {
            let mut this = self.0.lock().unwrap();
            this.last().take().map(|last| {
                *this.last() = if let Some(prev) = last.lock().unwrap().prev().take() {
                    *prev.lock().unwrap().next() = None;
                    Some(prev)
                } else {
                    *this.head() = None;
                    None
                };
                Arc::into_inner(last).unwrap().into_inner().unwrap().item
            })
        }

        pub fn pop_back(&self) -> Option<T> {
            self.pop::<Forward>()
        }

        pub fn pop_front(&self) -> Option<T> {
            self.pop::<Back>()
        }

        pub fn is_empty(&self) -> bool {
            self.0.lock().unwrap().head.is_some()
        }
    }

    pub struct IntoIter<T>(Neigh<T>);

    impl<T> IntoIterator for List<T> {
        type Item = T;

        type IntoIter = IntoIter<T>;

        fn into_iter(self) -> Self::IntoIter {
            IntoIter(self.0.into_inner().unwrap().head)
        }
    }

    impl<T> Iterator for IntoIter<T> {
        type Item = T;

        fn next(&mut self) -> Option<Self::Item> {
            self.0.take().map(|node| {
                self.0 = node.lock().unwrap().next.take().map(|next| {
                    next.lock().unwrap().prev = None;
                    next
                });
                Arc::into_inner(node).unwrap().into_inner().unwrap().item
            })
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
