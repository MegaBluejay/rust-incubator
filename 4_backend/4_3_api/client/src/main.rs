use std::num::NonZeroU16;

use anyhow::Result;
use clap::{Parser, Subcommand, ValueEnum};
use email_address::EmailAddress;
use openapi::{
    apis::{
        configuration::Configuration,
        role_api::{create_role, delete_role, get_role, list_roles, update_role},
        user_api::{create_user, delete_user, get_user, list_users, update_user},
    },
    models::{CreateRole, RolesPeriodModel, UserWithRoles},
};

#[derive(Debug, ValueEnum, Clone)]
#[value(rename_all = "snake_case")]
enum Permissions {
    Reader,
    Editor,
    Admin,
}

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

    let conf = Configuration {
        base_path: "http://localhost:3000".to_owned(),
        ..Default::default()
    };

    match command {
        Command::Create(command) => match command {
            Create::User { name, role, email } => {
                let user = create_user(
                    &conf,
                    openapi::models::CreateUser {
                        email: email.map(|email| Some(email.to_string())),
                        name,
                        role_slug: role,
                    },
                )
                .await?;
                print_user(&user);
            }
            Create::Role {
                slug,
                name,
                permissions,
            } => {
                let role = create_role(
                    &conf,
                    CreateRole {
                        name,
                        permissions: permissions.into(),
                        slug,
                    },
                )
                .await?;
                print_role(&role);
            }
        },
        Command::Update(command) => match command {
            Update::User {
                id,
                name,
                email,
                add_roles,
                remove_roles,
            } => {
                let user = update_user(
                    &conf,
                    id.get() as i32,
                    openapi::models::UpdateUser {
                        add_roles: add_roles.map(Some),
                        email: email.map(|email| Some(email.to_string())),
                        name: name.map(Some),
                        remove_roles: remove_roles.map(Some),
                    },
                )
                .await?;
                print_user(&user);
            }
            Update::Role {
                slug,
                name,
                permissions,
            } => {
                let role = update_role(
                    &conf,
                    &slug,
                    openapi::models::UpdateRole {
                        name: name.map(Some),
                        permissions: permissions.map(|permissions| Some(permissions.into())),
                    },
                )
                .await?;
                print_role(&role);
            }
        },
        Command::Get(command) => match command {
            Get::User { id } => {
                let user = get_user(&conf, id.get() as i32).await?;
                print_user(&user);
            }
            Get::Role { slug } => {
                let role = get_role(&conf, &slug).await?;
                print_role(&role);
            }
        },
        Command::List(command) => match command {
            List::Users => {
                let users = list_users(&conf).await?;
                for user in users {
                    print_user(&user);
                }
            }
            List::Roles => {
                let roles = list_roles(&conf).await?;
                for role in roles {
                    print_role(&role);
                }
            }
        },
        Command::Delete(command) => match command {
            Delete::User { id } => {
                delete_user(&conf, id.get() as i32).await?;
            }
            Delete::Role { slug } => {
                delete_role(&conf, &slug).await?;
            }
        },
    }

    Ok(())
}

fn print_user(user: &UserWithRoles) {
    println!("ID: {}", user.id);
    println!("\tName: {}", user.name);
    if let Some(Some(email)) = user.email.as_ref() {
        println!("\tEmail: {}", email);
    }
    println!(
        "\tRoles: {}",
        user.roles
            .iter()
            .map(|role| role.slug.to_owned())
            .collect::<Vec<_>>()
            .join(",")
    );
}

fn print_role(role: &RolesPeriodModel) {
    println!("Slug: {}", role.slug);
    println!("\tName: {}", role.name);
    println!("\tPermissions: {:?}", role.permissions);
}

impl From<Permissions> for openapi::models::Permissions {
    fn from(value: Permissions) -> Self {
        match value {
            Permissions::Admin => openapi::models::Permissions::Admin,
            Permissions::Editor => openapi::models::Permissions::Editor,
            Permissions::Reader => openapi::models::Permissions::Reader,
        }
    }
}
