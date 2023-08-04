use std::fmt::Display;

use async_trait::async_trait;
use juniper::{graphql_object, EmptySubscription, FieldError, GraphQLInputObject, RootNode};

#[async_trait]
pub trait Database {
    type Error;

    async fn get_users(
        &self,
        current_user: Option<&User>,
        user_ids: &[i32],
    ) -> Result<Vec<User>, Self::Error>;

    async fn find_user(
        &self,
        current_user: Option<&User>,
        name: Option<&str>,
    ) -> Result<User, Self::Error>;

    async fn register(&self, user: InUser) -> Result<User, Self::Error>;

    async fn login(&self, user: InUser) -> Result<String, Self::Error>;

    async fn edit(&self, current_user: Option<&User>, edit: EditUser) -> Result<User, Self::Error>;
}

#[derive(Clone)]
pub struct WrappedDatabase<T> {
    inner: T,
}

impl<T> WrappedDatabase<T>
where
    T: Database + Sync + Send,
    <T as Database>::Error: Display,
{
    pub fn new(inner: T) -> Self {
        Self { inner }
    }
}

#[async_trait]
impl<T> Database for WrappedDatabase<T>
where
    T: Database + Sync + Send,
    <T as Database>::Error: Display,
{
    type Error = FieldError;

    async fn get_users(
        &self,
        current_user: Option<&User>,
        user_ids: &[i32],
    ) -> Result<Vec<User>, Self::Error> {
        self.inner
            .get_users(current_user, user_ids)
            .await
            .map_err(Into::into)
    }

    async fn find_user(
        &self,
        current_user: Option<&User>,
        name: Option<&str>,
    ) -> Result<User, Self::Error> {
        self.inner
            .find_user(current_user, name)
            .await
            .map_err(Into::into)
    }

    async fn register(&self, user: InUser) -> Result<User, Self::Error> {
        self.inner.register(user).await.map_err(Into::into)
    }

    async fn login(&self, user: InUser) -> Result<String, Self::Error> {
        self.inner.login(user).await.map_err(Into::into)
    }

    async fn edit(&self, current_user: Option<&User>, edit: EditUser) -> Result<User, Self::Error> {
        self.inner
            .edit(current_user, edit)
            .await
            .map_err(Into::into)
    }
}

pub struct Context {
    pub db: Box<dyn Database<Error = FieldError> + Send + Sync>,
    pub current_user: Option<User>,
}

impl juniper::Context for Context {}

#[derive(Clone)]
pub struct User {
    pub id: i32,
    pub name: String,
    pub friend_ids: Vec<i32>,
}

#[graphql_object(context = Context)]
impl User {
    fn id(&self) -> i32 {
        self.id
    }

    fn name(&self) -> &str {
        &self.name
    }

    async fn friends(&self, ctx: &Context) -> Result<Vec<User>, FieldError> {
        ctx.db
            .get_users(ctx.current_user.as_ref(), &self.friend_ids)
            .await
    }
}

#[derive(GraphQLInputObject)]
pub struct InUser {
    pub name: String,
    pub password: String,
}

#[derive(GraphQLInputObject)]
pub struct EditUser {
    pub add_friends: Option<Vec<String>>,
    pub remove_friends: Option<Vec<String>>,
}

pub struct Query;

#[graphql_object(context = Context)]
impl Query {
    async fn user(name: Option<String>, ctx: &Context) -> Result<User, FieldError> {
        ctx.db
            .find_user(ctx.current_user.as_ref(), name.as_deref())
            .await
    }
}

pub struct Mutation;

#[graphql_object(context = Context)]
impl Mutation {
    async fn register(user: InUser, ctx: &Context) -> Result<User, FieldError> {
        ctx.db.register(user).await
    }

    async fn login(user: InUser, ctx: &Context) -> Result<String, FieldError> {
        ctx.db.login(user).await
    }

    async fn edit(edit: EditUser, ctx: &Context) -> Result<User, FieldError> {
        ctx.db.edit(ctx.current_user.as_ref(), edit).await
    }
}

pub type Root = RootNode<'static, Query, Mutation, EmptySubscription<Context>>;
