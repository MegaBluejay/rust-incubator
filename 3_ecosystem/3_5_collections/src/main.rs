use std::collections::{HashMap, HashSet};

use rpds::HashTrieMap;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct User {
    id: u64,
    nickname: String,
}

trait UserRepository {
    fn get_user(&self, id: u64) -> Option<User>;

    fn get_users(&self, ids: impl AsRef<[u64]>) -> HashMap<u64, User>;

    fn find_users(&self, nickname: impl AsRef<str>) -> HashSet<u64>;
}

pub struct PersistentRepo {
    users: HashTrieMap<u64, User>,
}

impl UserRepository for PersistentRepo {
    fn get_user(&self, id: u64) -> Option<User> {
        self.users.get(&id).cloned()
    }

    fn get_users(&self, ids: impl AsRef<[u64]>) -> HashMap<u64, User> {
        ids.as_ref()
            .iter()
            .filter_map(|&id| self.get_user(id).map(|user| (id, user)))
            .collect()
    }

    fn find_users(&self, nickname: impl AsRef<str>) -> HashSet<u64> {
        self.users
            .values()
            .filter(|user| user.nickname.contains(nickname.as_ref()))
            .map(|user| user.id)
            .collect()
    }
}

impl FromIterator<User> for PersistentRepo {
    fn from_iter<T: IntoIterator<Item = User>>(iter: T) -> Self {
        Self {
            users: iter.into_iter().map(|user| (user.id, user)).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_user() {
        let u1 = User {
            id: 1,
            nickname: "a".to_owned(),
        };
        let u2 = User {
            id: 2,
            nickname: "b".to_owned(),
        };
        let repo = PersistentRepo::from_iter([u1.clone(), u2]);
        assert_eq!(repo.get_user(1), Some(u1));
    }

    #[test]
    fn get_users() {
        let u1 = User {
            id: 1,
            nickname: "a".to_owned(),
        };
        let u2 = User {
            id: 2,
            nickname: "b".to_owned(),
        };
        let u3 = User {
            id: 3,
            nickname: "c".to_owned(),
        };
        let repo = PersistentRepo::from_iter([u1.clone(), u2.clone(), u3]);
        assert_eq!(
            repo.get_users([1, 2]),
            [u1, u2].into_iter().map(|user| (user.id, user)).collect(),
        );
    }

    #[test]
    fn find_users() {
        let u1 = User {
            id: 1,
            nickname: "a".to_owned(),
        };
        let u2 = User {
            id: 2,
            nickname: "bap".to_owned(),
        };
        let u3 = User {
            id: 3,
            nickname: "bop".to_owned(),
        };
        let repo = PersistentRepo::from_iter([u1, u2, u3]);
        assert_eq!(repo.find_users("a"), HashSet::from_iter([1, 2]));
    }
}

fn main() {}
