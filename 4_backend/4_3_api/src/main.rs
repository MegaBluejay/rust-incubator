use std::sync::Arc;
use std::{env, num::NonZeroU16};

use anyhow::Result;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use email_address::EmailAddress;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, Database, DatabaseConnection, DbErr, EntityTrait,
    ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, TransactionError, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

mod entities;

use entities::roles::Permissions;
use entities::{prelude::*, roles, users, users_roles};

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(Database::connect(env::var("DATABASE_URL")?).await?);

    let app = Router::new()
        .route("/users", get(list_users).post(create_user))
        .route(
            "/users/:id",
            get(get_user).patch(update_user).delete(delete_user),
        )
        .route("/roles", get(list_roles).post(create_role))
        .route(
            "/roles/:slug",
            get(get_role).patch(update_role).delete(delete_role),
        )
        .with_state(db);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

#[derive(Debug, Deserialize)]
struct CreateUser {
    name: String,
    role: String,
    email: Option<EmailAddress>,
}

#[derive(Debug, Deserialize)]
struct CreateRole {
    slug: String,
    name: String,
    permissions: Permissions,
}

#[derive(Debug, Deserialize)]
struct UpdateUser {
    name: Option<String>,
    email: Option<EmailAddress>,
    add_roles: Option<Vec<String>>,
    remove_roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct UpdateRole {
    name: Option<String>,
    permissions: Option<Permissions>,
}

async fn create_user(
    State(db): State<Arc<DatabaseConnection>>,
    Json(user): Json<CreateUser>,
) -> Result<StatusCode, Error> {
    let CreateUser { name, role, email } = user;
    db.transaction::<_, (), DbErr>(|txn| {
        Box::pin(async move {
            let user = users::ActiveModel {
                id: ActiveValue::NotSet,
                name: ActiveValue::Set(name),
                email: ActiveValue::Set(email.map(|email| email.to_string())),
            }
            .insert(txn)
            .await?;

            users_roles::ActiveModel {
                user_id: ActiveValue::Set(user.id),
                role_slug: ActiveValue::Set(role),
            }
            .insert(txn)
            .await?;

            Ok(())
        })
    })
    .await?;
    Ok(StatusCode::CREATED)
}

async fn create_role(
    State(db): State<Arc<DatabaseConnection>>,
    Json(role): Json<CreateRole>,
) -> Result<StatusCode, Error> {
    let CreateRole {
        slug,
        name,
        permissions,
    } = role;
    roles::ActiveModel {
        slug: ActiveValue::Set(slug),
        name: ActiveValue::Set(name),
        permissions: ActiveValue::Set(permissions),
    }
    .insert(db.as_ref())
    .await?;
    Ok(StatusCode::CREATED)
}

async fn update_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<NonZeroU16>,
    Json(user): Json<UpdateUser>,
) -> Result<StatusCode, Error> {
    let UpdateUser {
        name,
        email,
        add_roles,
        remove_roles,
    } = user;

    db.transaction::<_, (), Error>(|txn| {
        Box::pin(async move {
            let user = Users::find_by_id(id.get() as i32).one(txn).await?;
            if let Some(user) = user {
                let mut user: users::ActiveModel = user.into();
                if let Some(email) = email {
                    user.email = ActiveValue::Set(Some(email.to_string()));
                }
                if let Some(name) = name {
                    user.name = ActiveValue::Set(name);
                }
                let user = user.update(txn).await?;

                if let Some(add_roles) = add_roles {
                    UsersRoles::insert_many(add_roles.into_iter().map(|role| {
                        users_roles::ActiveModel {
                            user_id: ActiveValue::Set(user.id),
                            role_slug: ActiveValue::Set(role),
                        }
                    }))
                    .on_conflict(
                        OnConflict::columns([
                            users_roles::Column::UserId,
                            users_roles::Column::RoleSlug,
                        ])
                        .do_nothing()
                        .to_owned(),
                    )
                    .exec(txn)
                    .await?;
                }

                if let Some(remove_roles) = remove_roles {
                    UsersRoles::delete_many()
                        .filter(users_roles::Column::UserId.eq(user.id))
                        .filter(users_roles::Column::RoleSlug.is_in(remove_roles))
                        .exec(txn)
                        .await?;

                    let count = user.find_related(Roles).count(txn).await?;
                    if count == 0 {
                        return Err(Error::NoRole);
                    }
                }
            } else {
                return Err(Error::NotFound(Entity::User));
            }

            Ok(())
        })
    })
    .await?;

    Ok(StatusCode::OK)
}

async fn update_role(
    State(db): State<Arc<DatabaseConnection>>,
    Path(slug): Path<String>,
    Json(role): Json<UpdateRole>,
) -> Result<StatusCode, Error> {
    let UpdateRole { name, permissions } = role;
    let db = db.as_ref();

    let role = Roles::find_by_id(slug).one(db).await?;
    if let Some(role) = role {
        let mut role: roles::ActiveModel = role.into();
        if let Some(name) = name {
            role.name = ActiveValue::Set(name);
        }
        if let Some(permissions) = permissions {
            role.permissions = ActiveValue::Set(permissions);
        }
        role.update(db).await?;
    } else {
        return Err(Error::NotFound(Entity::Role));
    }

    Ok(StatusCode::OK)
}

#[derive(Debug, Serialize)]
struct UserWithRoles {
    #[serde(flatten)]
    user: users::Model,
    roles: Vec<roles::Model>,
}

async fn list_users(
    State(db): State<Arc<DatabaseConnection>>,
) -> Result<Json<Vec<UserWithRoles>>, Error> {
    Ok(Json(
        Users::find()
            .find_with_related(Roles)
            .all(db.as_ref())
            .await?
            .into_iter()
            .map(|(user, roles)| UserWithRoles { user, roles })
            .collect(),
    ))
}

async fn list_roles(
    State(db): State<Arc<DatabaseConnection>>,
) -> Result<Json<Vec<roles::Model>>, Error> {
    Ok(Json(Roles::find().all(db.as_ref()).await?))
}

async fn get_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<NonZeroU16>,
) -> Result<Json<UserWithRoles>, Error> {
    let users = Users::find_by_id(id.get() as i32)
        .find_with_related(Roles)
        .all(db.as_ref())
        .await?;

    if let Some((user, roles)) = users.into_iter().next() {
        Ok(Json(UserWithRoles { user, roles }))
    } else {
        Err(Error::NotFound(Entity::User))
    }
}

async fn get_role(
    State(db): State<Arc<DatabaseConnection>>,
    Path(slug): Path<String>,
) -> Result<Json<roles::Model>, Error> {
    let role = Roles::find_by_id(slug).one(db.as_ref()).await?;

    if let Some(role) = role {
        Ok(Json(role))
    } else {
        Err(Error::NotFound(Entity::Role))
    }
}

async fn delete_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<NonZeroU16>,
) -> Result<StatusCode, Error> {
    Users::delete_by_id(id.get() as i32)
        .exec(db.as_ref())
        .await?;

    Ok(StatusCode::OK)
}

async fn delete_role(
    State(db): State<Arc<DatabaseConnection>>,
    Path(slug): Path<String>,
) -> Result<StatusCode, Error> {
    db.transaction::<_, (), Error>(|txn| {
        Box::pin(async move {
            let single = UsersRoles::find()
                .group_by(users_roles::Column::UserId)
                .having(
                    users_roles::Column::RoleSlug
                        .count()
                        .eq(1)
                        .and(users_roles::Column::RoleSlug.eq(&slug)),
                )
                .count(txn)
                .await?;
            if single != 0 {
                return Err(Error::NoRole);
            }

            Roles::delete_by_id(slug).exec(txn).await?;

            Ok(())
        })
    })
    .await?;

    Ok(StatusCode::OK)
}

#[derive(Debug)]
enum Entity {
    User,
    Role,
}

#[derive(Debug, Error)]
enum Error {
    #[error(transparent)]
    Db(#[from] DbErr),
    #[error("{0:?} not found")]
    NotFound(Entity),
    #[error("user can't have no roles")]
    NoRole,
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = match &self {
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::BAD_REQUEST,
        };
        (status, self.to_string()).into_response()
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
