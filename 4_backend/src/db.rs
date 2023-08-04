use argon2::{
    password_hash::{rand_core::OsRng, SaltString},
    Argon2, PasswordHash, PasswordHasher, PasswordVerifier,
};
use async_trait::async_trait;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, DatabaseConnection, DbErr, EntityTrait, ModelTrait,
    QueryFilter, TransactionError, TransactionTrait,
};
use thiserror::Error;

use crate::{
    api::{Database, EditUser, InUser, User},
    entities::{friends, prelude::*, users},
};

#[derive(Debug, Error)]
pub enum Error {
    #[error(transparent)]
    Db(#[from] DbErr),
    #[error("password hashing error: {0}")]
    PasswordHash(argon2::password_hash::Error),
    #[error("not authorized")]
    NotAuthorized,
    #[error("not found")]
    NotFound,
}

impl From<argon2::password_hash::Error> for Error {
    fn from(value: argon2::password_hash::Error) -> Self {
        Self::PasswordHash(value)
    }
}

impl<T: Into<Error> + std::error::Error> From<TransactionError<T>> for Error {
    fn from(value: TransactionError<T>) -> Self {
        match value {
            TransactionError::Connection(db) => Self::Db(db),
            TransactionError::Transaction(other) => other.into(),
        }
    }
}

#[async_trait]
impl Database for DatabaseConnection {
    type Error = Error;

    async fn get_users(
        &self,
        current_user: Option<&User>,
        user_ids: &[i32],
    ) -> Result<Vec<User>, Self::Error> {
        current_user.ok_or(Error::NotAuthorized)?;

        let db_users = Users::find()
            .filter(users::Column::Id.is_in(user_ids.iter().map(Clone::clone)))
            .find_with_linked(users::FriendsLink)
            .all(self)
            .await?;

        Ok(db_users.into_iter().map(Into::into).collect())
    }

    async fn find_user(
        &self,
        current_user: Option<&User>,
        name: Option<&str>,
    ) -> Result<User, Self::Error> {
        let current_user = current_user.ok_or(Error::NotAuthorized)?;

        if let Some(name) = name {
            let db_user = Users::find()
                .filter(users::Column::Name.eq(name))
                .find_with_linked(users::FriendsLink)
                .all(self)
                .await?
                .into_iter()
                .next()
                .ok_or(Error::NotFound)?;

            Ok(db_user.into())
        } else {
            Ok(current_user.clone())
        }
    }

    async fn register(&self, user: InUser) -> Result<User, Self::Error> {
        let InUser { name, password } = user;

        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)?
            .to_string();

        let user = users::ActiveModel {
            name: ActiveValue::Set(name),
            password: ActiveValue::Set(password_hash),
            ..Default::default()
        }
        .insert(self)
        .await?;

        Ok((user, vec![]).into())
    }

    async fn login(&self, user: InUser) -> Result<String, Self::Error> {
        // TODO actual tokens
        let InUser { name, password } = user;

        let user = Users::find()
            .filter(users::Column::Name.eq(name))
            .one(self)
            .await?
            .ok_or(Error::NotFound)?;

        let parsed = PasswordHash::new(&user.password)?;
        Argon2::default().verify_password(password.as_bytes(), &parsed)?;

        Ok(user.id.to_string())
    }

    async fn edit(&self, current_user: Option<&User>, edit: EditUser) -> Result<User, Self::Error> {
        let current_user = current_user.ok_or(Error::NotAuthorized)?;

        let id = current_user.id;

        self.transaction::<_, User, Error>(|txn| {
            Box::pin(async move {
                let user = Users::find_by_id(id)
                    .one(txn)
                    .await?
                    .ok_or(Error::NotFound)?;

                if let Some(add_friends) = edit.add_friends {
                    let new_friends = Users::find()
                        .filter(users::Column::Name.is_in(add_friends))
                        .all(txn)
                        .await?;

                    let adds = new_friends.into_iter().map(|friend| friends::ActiveModel {
                        user_id: ActiveValue::Set(id),
                        friend_id: ActiveValue::Set(friend.id),
                    });

                    Friends::insert_many(adds).exec(txn).await?;
                }

                if let Some(remove_friends) = edit.remove_friends {
                    let removes = Users::find()
                        .filter(users::Column::Name.is_in(remove_friends))
                        .all(txn)
                        .await?
                        .into_iter()
                        .map(|user| user.id);

                    Friends::delete_many().filter(
                        friends::Column::UserId
                            .eq(id)
                            .and(friends::Column::FriendId.is_in(removes)),
                    );
                }

                let friends = user.find_linked(users::FriendsLink).all(txn).await?;
                Ok((user, friends).into())
            })
        })
        .await
        .map_err(Into::into)
    }
}

impl From<(users::Model, Vec<friends::Model>)> for User {
    fn from(value: (users::Model, Vec<friends::Model>)) -> Self {
        let (user, friends) = value;
        Self {
            id: user.id,
            name: user.name,
            friend_ids: friends.into_iter().map(|friend| friend.friend_id).collect(),
        }
    }
}
