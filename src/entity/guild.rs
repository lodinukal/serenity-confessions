//! `SeaORM` Entity. Generated by sea-orm-codegen 0.11.3

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "guild")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub id: u64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::channels::Entity")]
    Channels,
}

impl Related<super::channels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Channels.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
