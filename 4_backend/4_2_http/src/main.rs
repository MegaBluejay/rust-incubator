use std::io::Write;
use std::sync::Arc;
use std::{env, num::NonZeroU16};

use anyhow::Result;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{Json, Router};
use clap::{Parser, Subcommand};
use email_address::EmailAddress;
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DatabaseConnection,
    DbErr, EntityTrait, ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, TransactionError,
    TransactionTrait,
};
use thiserror::Error;

mod entities;

use entities::roles::Permissions;
use entities::{prelude::*, roles, users, users_roles};

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    #[command(subcommand)]
    Create(Create),
    #[command(subcommand)]
    Update(Update),
    #[command(subcommand)]
    List(List),
    #[command(subcommand)]
    Get(Get),
    #[command(subcommand)]
    Delete(Delete),
}

#[derive(Debug, Subcommand)]
enum Create {
    User {
        #[arg(long)]
        name: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        email: Option<EmailAddress>,
    },
    Role {
        #[arg(long)]
        slug: String,
        #[arg(long)]
        name: String,
        #[arg(long)]
        permissions: Permissions,
    },
}

#[derive(Debug, Subcommand)]
enum Update {
    User {
        id: NonZeroU16,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        email: Option<EmailAddress>,
        #[arg(long)]
        add_roles: Option<Vec<String>>,
        #[arg(long)]
        remove_roles: Option<Vec<String>>,
    },
    Role {
        slug: String,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        permissions: Option<Permissions>,
    },
}

#[derive(Debug, Subcommand)]
enum List {
    Users,
    Roles,
}

#[derive(Debug, Subcommand)]
enum Get {
    User { id: NonZeroU16 },
    Role { slug: String },
}

#[derive(Debug, Subcommand)]
enum Delete {
    User { id: NonZeroU16 },
    Role { slug: String },
}

#[tokio::main]
async fn main() -> Result<()> {
    let db = Arc::new(Database::connect(env::var("DATABASE_URL")?).await?);

    let app = Router::new().route("/", get(handler)).with_state(db);

    axum::Server::bind(&"0.0.0.0:3000".parse()?)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

async fn handler(
    State(db): State<Arc<DatabaseConnection>>,
    Json(args): Json<Vec<String>>,
) -> Result<Response, Error> {
    let Cli { command } = Cli::try_parse_from(args)?;

    let db = db.as_ref();

    Ok(match command {
        Command::Create(command) => do_create(command, db).await?,
        Command::Update(command) => do_update(command, db).await?,
        Command::Delete(command) => do_delete(command, db).await?,
        Command::Get(command) => do_get(command, db).await?,
        Command::List(command) => do_list(command, db).await?,
    })
}

enum Response {
    None,
    Output(Vec<u8>),
}

impl IntoResponse for Response {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::None => StatusCode::NO_CONTENT.into_response(),
            Self::Output(out) => out.into_response(),
        }
    }
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
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error("user can't have no roles")]
    NoRole,
    #[error(transparent)]
    Clap(#[from] clap::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::BAD_REQUEST, self.to_string()).into_response()
    }
}

impl From<TransactionError<Error>> for Error {
    fn from(value: TransactionError<Error>) -> Self {
        match value {
            TransactionError::Connection(db) => Self::Db(db),
            TransactionError::Transaction(other) => other,
        }
    }
}

impl From<TransactionError<DbErr>> for Error {
    fn from(value: TransactionError<DbErr>) -> Self {
        match value {
            TransactionError::Connection(db) => Self::Db(db),
            TransactionError::Transaction(db) => Self::Db(db),
        }
    }
}

async fn do_list(command: List, db: &impl ConnectionTrait) -> Result<Response, Error> {
    let mut out = vec![];
    match command {
        List::Users => {
            let users = Users::find().find_with_related(Roles).all(db).await?;
            for (user, roles) in users.iter() {
                print_user(user, roles, &mut out)?;
            }
        }
        List::Roles => {
            let roles = Roles::find().all(db).await?;
            for role in roles.iter() {
                print_role(role, &mut out)?;
            }
        }
    }
    Ok(Response::Output(out))
}

fn print_user<'a>(
    user: &users::Model,
    roles: impl IntoIterator<Item = &'a roles::Model>,
    writer: &mut impl Write,
) -> Result<(), Error> {
    writeln!(writer, "ID: {}", user.id)?;
    writeln!(writer, "\tName: {}", user.name)?;
    if let Some(email) = user.email.as_ref() {
        writeln!(writer, "\tEmail: {}", email)?;
    }
    writeln!(
        writer,
        "\tRoles: {}",
        roles
            .into_iter()
            .map(|role| role.slug.to_owned())
            .collect::<Vec<_>>()
            .join(",")
    )?;
    Ok(())
}

fn print_role(role: &roles::Model, writer: &mut impl Write) -> Result<(), Error> {
    writeln!(writer, "Slug: {}", role.slug)?;
    writeln!(writer, "\tName: {}", role.name)?;
    writeln!(writer, "\tPermissions: {:?}", role.permissions)?;
    Ok(())
}

async fn do_get(command: Get, db: &impl ConnectionTrait) -> Result<Response, Error> {
    let mut out = vec![];
    match command {
        Get::User { id } => {
            let user = Users::find_by_id(id.get() as i32)
                .find_with_related(Roles)
                .all(db)
                .await?;
            if let [(user, roles)] = &user[..] {
                print_user(user, roles, &mut out)?;
            } else {
                return Err(Error::NotFound(Entity::User));
            }
        }
        Get::Role { slug } => {
            let role = Roles::find_by_id(slug).one(db).await?;
            if let Some(role) = role {
                print_role(&role, &mut out)?;
            } else {
                return Err(Error::NotFound(Entity::User));
            }
        }
    }
    Ok(Response::Output(out))
}

async fn do_create(
    command: Create,
    db: &(impl ConnectionTrait + TransactionTrait),
) -> Result<Response, Error> {
    match command {
        Create::User { name, role, email } => {
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
        }
        Create::Role {
            slug,
            name,
            permissions,
        } => {
            roles::ActiveModel {
                slug: ActiveValue::Set(slug),
                name: ActiveValue::Set(name),
                permissions: ActiveValue::Set(permissions),
            }
            .insert(db)
            .await?;
        }
    }
    Ok(Response::None)
}

async fn do_update(
    command: Update,
    db: &(impl ConnectionTrait + TransactionTrait),
) -> Result<Response, Error> {
    match command {
        Update::User {
            id,
            name,
            email,
            add_roles,
            remove_roles,
        } => {
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
        }
        Update::Role {
            slug,
            name,
            permissions,
        } => {
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
        }
    }
    Ok(Response::None)
}

async fn do_delete(
    command: Delete,
    db: &(impl ConnectionTrait + TransactionTrait),
) -> Result<Response, Error> {
    match command {
        Delete::User { id } => {
            Users::delete_by_id(id.get() as i32).exec(db).await?;
        }
        Delete::Role { slug } => {
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
            .await?
        }
    }
    Ok(Response::None)
}
