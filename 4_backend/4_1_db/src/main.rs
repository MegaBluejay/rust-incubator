use std::num::NonZeroU16;

use clap::{Parser, Subcommand};

mod entities;

use entities::roles::Permissions;

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
}

#[derive(Debug, Subcommand)]
enum Create {
    User {
        #[arg(long)]
        name: String,
        #[arg(long)]
        role: String,
        #[arg(long)]
        email: Option<String>,
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
        role: Option<String>,
        #[arg(long)]
        email: Option<String>,
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
        permissions: Option<String>,
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

fn main() {
    let _ = Cli::parse();
}
