use std::{borrow::Cow, collections::HashMap, hash::Hash};

pub trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

#[derive(Debug)]
pub struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

#[derive(Debug)]
pub enum UserError {
    NotFound,
    AlreadyExists,
}

mod dynamic_dispatch {
    use crate::{Storage, User, UserError};

    pub struct UserRepository<'a>(&'a mut dyn Storage<u64, User>);

    impl<'a> UserRepository<'a> {
        pub fn new(storage: &'a mut dyn Storage<u64, User>) -> Self {
            Self(storage)
        }

        pub fn add(&mut self, user: User) -> Result<(), UserError> {
            if self.0.get(&user.id).is_some() {
                Err(UserError::AlreadyExists)
            } else {
                self.0.set(user.id, user);
                Ok(())
            }
        }

        pub fn get(&self, id: u64) -> Result<&User, UserError> {
            self.0.get(&id).ok_or(UserError::NotFound)
        }

        pub fn update(&mut self, user: User) -> Result<(), UserError> {
            if self.0.get(&user.id).is_none() {
                Err(UserError::NotFound)
            } else {
                self.0.set(user.id, user);
                Ok(())
            }
        }

        pub fn remove(&mut self, id: u64) -> Result<User, UserError> {
            self.0.remove(&id).ok_or(UserError::NotFound)
        }
    }
}

mod static_dispatch {
    use crate::{Storage, User, UserError};

    pub struct UserRepository<'a, T>(&'a mut T);

    impl<'a, T: Storage<u64, User>> UserRepository<'a, T> {
        pub fn new(storage: &'a mut T) -> Self {
            Self(storage)
        }

        pub fn add(&mut self, user: User) -> Result<(), UserError> {
            if self.0.get(&user.id).is_some() {
                Err(UserError::AlreadyExists)
            } else {
                self.0.set(user.id, user);
                Ok(())
            }
        }

        pub fn get(&self, id: u64) -> Result<&User, UserError> {
            self.0.get(&id).ok_or(UserError::NotFound)
        }

        pub fn update(&mut self, user: User) -> Result<(), UserError> {
            if self.0.get(&user.id).is_none() {
                Err(UserError::NotFound)
            } else {
                self.0.set(user.id, user);
                Ok(())
            }
        }

        pub fn remove(&mut self, id: u64) -> Result<User, UserError> {
            self.0.remove(&id).ok_or(UserError::NotFound)
        }
    }
}

struct HashStorage<K, V>(HashMap<K, V>);

impl<K: Eq + Hash, V> Storage<K, V> for HashStorage<K, V> {
    fn set(&mut self, key: K, val: V) {
        self.0.insert(key, val);
    }

    fn get(&self, key: &K) -> Option<&V> {
        self.0.get(key)
    }

    fn remove(&mut self, key: &K) -> Option<V> {
        self.0.remove(key)
    }
}

fn main() {
    let mut storage = HashStorage(HashMap::new());
    let mut dyn_repo = dynamic_dispatch::UserRepository::new(&mut storage);
    dyn_repo
        .add(User {
            id: 1,
            email: "email".into(),
            activated: true,
        })
        .unwrap();
    let stat_repo = static_dispatch::UserRepository::new(&mut storage);
    println!("{:?}", stat_repo.get(1));
}
