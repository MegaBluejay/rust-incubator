//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(
    Debug, Clone, PartialEq, Eq, EnumIter, DeriveActiveEnum, Deserialize, Serialize, ToSchema,
)]
#[serde(rename_all = "snake_case")]
#[sea_orm(rs_type = "String", db_type = "String(Some(256))")]
pub enum Permissions {
    #[sea_orm(string_value = "reader")]
    Reader,
    #[sea_orm(string_value = "editor")]
    Editor,
    #[sea_orm(string_value = "admin")]
    Admin,
}

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, ToSchema)]
#[schema(as = roles::Model, title = "Role")]
#[serde(rename_all = "camelCase")]
#[sea_orm(table_name = "roles")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub slug: String,
    pub name: String,
    pub permissions: Permissions,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::users_roles::Entity")]
    UsersRoles,
}

impl Related<super::users_roles::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::UsersRoles.def()
    }
}

impl Related<super::users::Entity> for Entity {
    fn to() -> RelationDef {
        super::users_roles::Relation::Users.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::users_roles::Relation::Roles.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
