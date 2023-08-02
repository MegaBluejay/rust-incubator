use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_table(
                Table::create()
                    .table(Users::Table)
                    .col(
                        ColumnDef::new(Users::Id)
                            .integer()
                            .not_null()
                            .auto_increment()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Users::Name).string().not_null())
                    .col(ColumnDef::new(Users::Email).string())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Roles::Table)
                    .col(
                        ColumnDef::new(Roles::Slug)
                            .string_len(256)
                            .not_null()
                            .primary_key(),
                    )
                    .col(ColumnDef::new(Roles::Name).string().not_null())
                    .col(
                        ColumnDef::new(Roles::Permissions)
                            .string_len(256)
                            .not_null(),
                    )
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(UsersRoles::Table)
                    .col(ColumnDef::new(UsersRoles::UserId).integer().not_null())
                    .col(
                        ColumnDef::new(UsersRoles::RoleSlug)
                            .string_len(256)
                            .not_null(),
                    )
                    .primary_key(
                        Index::create()
                            .col(UsersRoles::UserId)
                            .col(UsersRoles::RoleSlug),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersRoles::Table, UsersRoles::UserId)
                            .to(Users::Table, Users::Id)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .from(UsersRoles::Table, UsersRoles::RoleSlug)
                            .to(Roles::Table, Roles::Slug)
                            .on_delete(ForeignKeyAction::Cascade),
                    )
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(UsersRoles::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Users::Table).to_owned())
            .await?;

        manager
            .drop_table(Table::drop().table(Roles::Table).to_owned())
            .await
    }
}

#[derive(DeriveIden)]
pub enum Users {
    Table,
    Id,
    Name,
    Email,
}

#[derive(DeriveIden)]
pub enum Roles {
    Table,
    Slug,
    Name,
    Permissions,
}

#[derive(DeriveIden)]
pub enum UsersRoles {
    Table,
    UserId,
    RoleSlug,
}
