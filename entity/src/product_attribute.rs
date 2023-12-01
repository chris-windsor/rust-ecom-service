//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.4

use sea_orm::entity::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq, Serialize, Deserialize)]
#[sea_orm(table_name = "product_attribute")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub product_id: i32,
    pub attribute_id: i32,
    pub content: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::attribute::Entity",
        from = "Column::AttributeId",
        to = "super::attribute::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Attribute,
    #[sea_orm(
        belongs_to = "super::product_detail::Entity",
        from = "Column::ProductId",
        to = "super::product_detail::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    ProductDetail,
}

impl Related<super::attribute::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Attribute.def()
    }
}

impl Related<super::product_detail::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::ProductDetail.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
