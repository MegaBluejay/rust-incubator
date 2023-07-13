use std::borrow::Cow;

#[derive(Debug, Clone, PartialEq)]
pub struct User {
    id: u64,
    email: Cow<'static, str>,
    activated: bool,
}

#[derive(Debug)]
enum UserError {
    NotFound,
    AlreadyExists,
}

trait UserRepository {
    fn get(&self, id: u64) -> Result<User, UserError>;

    fn add(&self, user: User) -> Result<(), UserError>;

    fn update(&self, user: User) -> Result<(), UserError>;

    fn remove(&self, id: u64) -> Result<User, UserError>;
}

trait Storage<K, V> {
    fn set(&mut self, key: K, val: V);
    fn get(&self, key: &K) -> Option<&V>;
    fn remove(&mut self, key: &K) -> Option<V>;
}

mod dynamic_dispatch {
    use std::cell::RefCell;

    use crate::{Storage, User, UserError};

    pub struct UserRepository(RefCell<dyn Storage<u64, User>>);

    impl crate::UserRepository for UserRepository {
        fn get(&self, id: u64) -> Result<User, UserError> {
            self.0.borrow().get(&id).cloned().ok_or(UserError::NotFound)
        }

        fn add(&self, user: User) -> Result<(), UserError> {
            let mut st = self.0.borrow_mut();
            if st.get(&user.id).is_some() {
                Err(UserError::AlreadyExists)
            } else {
                st.set(user.id, user);
                Ok(())
            }
        }

        fn update(&self, user: User) -> Result<(), UserError> {
            let mut st = self.0.borrow_mut();
            if st.get(&user.id).is_none() {
                Err(UserError::NotFound)
            } else {
                st.set(user.id, user);
                Ok(())
            }
        }

        fn remove(&self, id: u64) -> Result<User, UserError> {
            self.0.borrow_mut().remove(&id).ok_or(UserError::NotFound)
        }
    }
}

mod static_dispatch {
    use std::cell::RefCell;

    use crate::{Storage, User, UserError};

    pub struct UserRepository<S>(RefCell<S>);

    impl<S: Storage<u64, User>> crate::UserRepository for UserRepository<S> {
        fn get(&self, id: u64) -> Result<User, UserError> {
            self.0.borrow().get(&id).cloned().ok_or(UserError::NotFound)
        }

        fn add(&self, user: User) -> Result<(), UserError> {
            let mut st = self.0.borrow_mut();
            if st.get(&user.id).is_some() {
                Err(UserError::AlreadyExists)
            } else {
                st.set(user.id, user);
                Ok(())
            }
        }

        fn update(&self, user: User) -> Result<(), UserError> {
            let mut st = self.0.borrow_mut();
            if st.get(&user.id).is_none() {
                Err(UserError::NotFound)
            } else {
                st.set(user.id, user);
                Ok(())
            }
        }

        fn remove(&self, id: u64) -> Result<User, UserError> {
            self.0.borrow_mut().remove(&id).ok_or(UserError::NotFound)
        }
    }
}

trait Command {}

struct CreateUser;

impl Command for CreateUser {}

trait CommandHandler<C: Command + ?Sized> {
    type Context: ?Sized;
    type Result;

    fn handle_command(&self, cmd: &C, ctx: &Self::Context) -> Self::Result;
}

impl CommandHandler<CreateUser> for User {
    type Context = dyn UserRepository;

    type Result = Result<(), UserError>;

    fn handle_command(&self, _cmd: &CreateUser, ctx: &Self::Context) -> Self::Result {
        ctx.add(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    struct VecAddedRepo(RefCell<Vec<User>>);

    impl UserRepository for VecAddedRepo {
        fn get(&self, _id: u64) -> Result<User, UserError> {
            unimplemented!()
        }

        fn add(&self, user: User) -> Result<(), UserError> {
            self.0.borrow_mut().push(user);
            Ok(())
        }

        fn update(&self, _user: User) -> Result<(), UserError> {
            unimplemented!()
        }

        fn remove(&self, _id: u64) -> Result<User, UserError> {
            unimplemented!()
        }
    }

    #[test]
    fn create() {
        let user = User {
            id: 1,
            email: "email@email.com".into(),
            activated: false,
        };
        let repo = VecAddedRepo(RefCell::new(vec![]));
        user.handle_command(&CreateUser, &repo).unwrap();
        assert_eq!(repo.0.into_inner(), vec![user]);
    }
}

fn main() {
    println!("Implement me!");
}
