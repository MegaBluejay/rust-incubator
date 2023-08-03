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
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;

mod entities;

use entities::roles::Permissions;
use entities::{prelude::*, roles, users, users_roles};

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(Database::connect(env::var("DATABASE_URL")?).await?);

    let app = Router::new()
        .merge(SwaggerUi::new("/swagger").url("/openapi.json", ApiDoc::openapi()))
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

#[utoipa::path(
    post,
    path = "/users",
    request_body = CreateUser,
    responses(
        (status = 201, description = "User Created", body = UserWithRoles),
    ),
    tag = "user",
)]
async fn create_user(
    State(db): State<Arc<DatabaseConnection>>,
    Json(user): Json<CreateUser>,
) -> Result<(StatusCode, Json<UserWithRoles>), Error> {
    let CreateUser {
        name,
        role_slug,
        email,
    } = user;
    let (user, role) = db
        .transaction::<_, (users::Model, roles::Model), DbErr>(|txn| {
            Box::pin(async move {
                let user = users::ActiveModel {
                    id: ActiveValue::NotSet,
                    name: ActiveValue::Set(name),
                    email: ActiveValue::Set(email.map(|email| email.to_string())),
                }
                .insert(txn)
                .await?;

                let role = Roles::find_by_id(&role_slug).one(txn).await?;

                users_roles::ActiveModel {
                    user_id: ActiveValue::Set(user.id),
                    role_slug: ActiveValue::Set(role_slug),
                }
                .insert(txn)
                .await?;

                Ok((user, role.unwrap()))
            })
        })
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(UserWithRoles {
            user,
            roles: vec![role],
        }),
    ))
}

#[utoipa::path(
    post,
    path = "/roles",
    request_body = CreateRole,
    responses(
        (status = 201, description = "Role Created", body = roles::Model),
    ),
    tag = "role",
)]
async fn create_role(
    State(db): State<Arc<DatabaseConnection>>,
    Json(role): Json<CreateRole>,
) -> Result<(StatusCode, Json<roles::Model>), Error> {
    let CreateRole {
        slug,
        name,
        permissions,
    } = role;

    let role = roles::ActiveModel {
        slug: ActiveValue::Set(slug),
        name: ActiveValue::Set(name),
        permissions: ActiveValue::Set(permissions),
    }
    .insert(db.as_ref())
    .await?;

    Ok((StatusCode::CREATED, Json(role)))
}

#[utoipa::path(
    patch,
    path = "/users/{id}",
    request_body = UpdateUser,
    params(
        ("id", description = "User id"),
    ),
    responses(
        (status = 200, description = "User Updated", body = UserWithRoles),
    ),
    tag = "user",
)]
async fn update_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<NonZeroU16>,
    Json(user): Json<UpdateUser>,
) -> Result<Json<UserWithRoles>, Error> {
    let UpdateUser {
        name,
        email,
        add_roles,
        remove_roles,
    } = user;

    let (user, roles) = db
        .transaction::<_, (users::Model, Vec<roles::Model>), Error>(|txn| {
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
                    }

                    let roles = user.find_related(Roles).all(txn).await?;

                    if roles.is_empty() {
                        Err(Error::NoRole)
                    } else {
                        Ok((user, roles))
                    }
                } else {
                    Err(Error::NotFound(Entity::User))
                }
            })
        })
        .await?;

    Ok(Json(UserWithRoles { user, roles }))
}

#[utoipa::path(
    patch,
    path = "/roles/{slug}",
    request_body = UpdateRole,
    params(
        ("slug", description = "Role slug"),
    ),
    responses(
        (status = 200, description = "Role Updated", body = roles::Model),
    ),
    tag = "role",
)]
async fn update_role(
    State(db): State<Arc<DatabaseConnection>>,
    Path(slug): Path<String>,
    Json(role): Json<UpdateRole>,
) -> Result<Json<roles::Model>, Error> {
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

        let role = role.update(db).await?;
        Ok(Json(role))
    } else {
        Err(Error::NotFound(Entity::Role))
    }
}

#[utoipa::path(
    get,
    path = "/users",
    responses(
        (status = 200, description = "Users", body = Vec<UserWithRoles>),
    ),
    tag = "user",
)]
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

#[utoipa::path(
    get,
    path = "/roles",
    responses(
        (status = 200, description = "Roles", body = Vec<roles::Model>),
    ),
    tag = "role",
)]
async fn list_roles(
    State(db): State<Arc<DatabaseConnection>>,
) -> Result<Json<Vec<roles::Model>>, Error> {
    Ok(Json(Roles::find().all(db.as_ref()).await?))
}

#[utoipa::path(
    get,
    path = "/users/{id}",
    params(
        ("id", description = "User id"),
    ),
    responses(
        (status = 200, description = "User Updated", body = Vec<UserWithRoles>),
    ),
    tag = "user",
)]
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

#[utoipa::path(
    get,
    path = "/roles/{slug}",
    params(
        ("slug", description = "Role slug"),
    ),
    responses(
        (status = 200, description = "Role", body = roles::Model),
    ),
    tag = "role",
)]
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

#[utoipa::path(
    delete,
    path = "/users/{id}",
    params(
        ("id", description = "User id"),
    ),
    responses(
        (status = 200, description = "User deleted"),
    ),
    tag = "user",
)]
async fn delete_user(
    State(db): State<Arc<DatabaseConnection>>,
    Path(id): Path<NonZeroU16>,
) -> Result<StatusCode, Error> {
    Users::delete_by_id(id.get() as i32)
        .exec(db.as_ref())
        .await?;

    Ok(StatusCode::OK)
}

#[utoipa::path(
    delete,
    path = "/roles/{slug}",
    params(
        ("slug", description = "Role slug"),
    ),
    responses(
        (status = 200, description = "Role deleted"),
    ),
    tag = "role",
)]
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

#[derive(Debug, Deserialize, ToSchema)]
struct CreateUser {
    name: String,
    role_slug: String,
    #[schema(format = "email")]
    email: Option<EmailAddress>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct CreateRole {
    slug: String,
    name: String,
    permissions: Permissions,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateUser {
    name: Option<String>,
    #[schema(format = "email")]
    email: Option<EmailAddress>,
    add_roles: Option<Vec<String>>,
    remove_roles: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, ToSchema)]
struct UpdateRole {
    name: Option<String>,
    permissions: Option<Permissions>,
}

#[derive(Debug, Serialize, ToSchema)]
struct UserWithRoles {
    #[serde(flatten)]
    user: users::Model,
    roles: Vec<roles::Model>,
}

#[derive(OpenApi)]
#[openapi(
    paths(
        create_user,
        create_role,
        update_user,
        update_role,
        list_users,
        list_roles,
        get_user,
        get_role,
        delete_user,
        delete_role,
    ),
    components(schemas(
        UserWithRoles,
        CreateUser,
        CreateRole,
        UpdateUser,
        UpdateRole,
        roles::Model,
        Permissions,
        users::Model,
    )),
    tags(
        (name = "user", description = "User"),
        (name = "role", description = "Role"),
    ),
)]
struct ApiDoc;
