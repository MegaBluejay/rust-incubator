use std::{env, num::NonZeroU16};

use anyhow::Result;
use clap::{Parser, Subcommand};
use email_address::EmailAddress;

mod entities;

use entities::roles::Permissions;
use entities::{prelude::*, roles, users, users_roles};
use sea_orm::sea_query::OnConflict;
use sea_orm::{
    ActiveModelTrait, ActiveValue, ColumnTrait, ConnectionTrait, Database, DbErr, EntityTrait,
    ModelTrait, PaginatorTrait, QueryFilter, QuerySelect, TransactionTrait,
};

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
    let Cli { command } = Cli::try_parse()?;

    let db = Database::connect(env::var("DATABASE_URL")?).await?;

    match command {
        Command::Create(create) => do_create(create, &db).await?,
        Command::Update(update) => do_update(update, &db).await?,
        Command::List(list) => do_list(list, &db).await?,
        Command::Get(get) => do_get(get, &db).await?,
        Command::Delete(delete) => do_delete(delete, &db).await?,
    };

    Ok(())
}

async fn do_list(command: List, db: &impl ConnectionTrait) -> Result<()> {
    match command {
        List::Users => {
            let users = Users::find().find_with_related(Roles).all(db).await?;
            for (user, roles) in users.iter() {
                print_user(user, roles);
            }
        }
        List::Roles => {
            let roles = Roles::find().all(db).await?;
            for role in roles.iter() {
                print_role(role);
            }
        }
    }
    Ok(())
}

fn print_user<'a>(user: &users::Model, roles: impl IntoIterator<Item = &'a roles::Model>) {
    println!("ID: {}", user.id);
    println!("\tName: {}", user.name);
    if let Some(email) = user.email.as_ref() {
        println!("\tEmail: {}", email);
    }
    println!(
        "\tRoles: {}",
        roles
            .into_iter()
            .map(|role| role.slug.to_owned())
            .collect::<Vec<_>>()
            .join(",")
    );
}

fn print_role(role: &roles::Model) {
    println!("Slug: {}", role.slug);
    println!("\tName: {}", role.name);
    println!("\tPermissions: {:?}", role.permissions);
}

async fn do_get(command: Get, db: &impl ConnectionTrait) -> Result<()> {
    match command {
        Get::User { id } => {
            let user = Users::find_by_id(id.get() as i32)
                .find_with_related(Roles)
                .all(db)
                .await?;
            if let [(user, roles)] = &user[..] {
                print_user(user, roles);
            } else {
                println!("User not found");
            }
        }
        Get::Role { slug } => {
            let role = Roles::find_by_id(slug).one(db).await?;
            if let Some(role) = role {
                print_role(&role);
            } else {
                println!("Role not found");
            }
        }
    }
    Ok(())
}

async fn do_create(command: Create, db: &(impl ConnectionTrait + TransactionTrait)) -> Result<()> {
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
    Ok(())
}

async fn do_update(command: Update, db: &(impl ConnectionTrait + TransactionTrait)) -> Result<()> {
    match command {
        Update::User {
            id,
            name,
            email,
            add_roles,
            remove_roles,
        } => {
            db.transaction::<_, (), DbErr>(|txn| {
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
                                Err(DbErr::Custom("user has 0 roles".to_owned()))?;
                            }
                        }
                    } else {
                        println!("User not found");
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
                println!("Role not found");
            }
        }
    }
    Ok(())
}

async fn do_delete(command: Delete, db: &(impl ConnectionTrait + TransactionTrait)) -> Result<()> {
    match command {
        Delete::User { id } => {
            Users::delete_by_id(id.get() as i32).exec(db).await?;
        }
        Delete::Role { slug } => {
            db.transaction::<_, (), DbErr>(|txn| {
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
                        Err(DbErr::Custom(
                            "role is the only one for some user".to_owned(),
                        ))?;
                    }

                    Roles::delete_by_id(slug).exec(txn).await?;

                    Ok(())
                })
            })
            .await?
        }
    }
    Ok(())
}
