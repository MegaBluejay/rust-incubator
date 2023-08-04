//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "users")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub password: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

pub struct FriendsLink;

impl Linked for FriendsLink {
    type FromEntity = Entity;

    type ToEntity = super::friends::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![super::friends::Relation::Users1.def()]
    }
}

pub struct FriendOfLink;

impl Linked for FriendOfLink {
    type FromEntity = Entity;

    type ToEntity = super::friends::Entity;

    fn link(&self) -> Vec<sea_orm::LinkDef> {
        vec![super::friends::Relation::Users2.def()]
    }
}

impl ActiveModelBehavior for ActiveModel {}
