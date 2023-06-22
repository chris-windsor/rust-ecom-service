use entity::product;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::authorize_net::{Address, AuthorizeNetFee, CreditCard};

pub struct Discount {
    amount: usize,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Invoice {
    pub id: u32,
    pub subtotal: Decimal,
    pub shipping: Decimal,
    pub taxes: Decimal,
    pub total: Decimal,
}

#[derive(Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub first_name: Arc<str>,
    pub last_name: Arc<str>,
    pub email: Arc<str>,
    pub phone_number: Arc<str>,
    pub ip_address: Arc<str>,
    pub billing_address: Address,
    pub shipping_address: Address,
    pub credit_card: CreditCard,
}

#[derive(Clone, Deserialize, Serialize)]
pub struct Item {
    pub id: i32,
    pub qty: i32,
    pub price: f32,
}

#[derive(Clone, Serialize)]
pub struct Order {
    pub items: Vec<Item>,
}

pub struct OrderAdjustments {
    pub tax_rate: Decimal,
    pub shipping_fee: Decimal,
}

impl Invoice {
    pub fn create(
        order: &Order,
        order_products: Vec<product::Model>,
        adjustments: OrderAdjustments,
    ) -> Self {
        let subtotal = Self::calc_subtotal(&order, order_products);
        let taxes = Self::calc_taxes(&subtotal, &adjustments.tax_rate);

        Invoice {
            subtotal: subtotal.clone(),
            shipping: adjustments.shipping_fee.clone(),
            taxes: taxes.clone(),
            total: subtotal.clone() + adjustments.shipping_fee + taxes.clone(),
            id: rand::random(),
        }
    }

    fn calc_subtotal(order: &Order, order_products: Vec<product::Model>) -> Decimal {
        let mut subtotal = Decimal::from_f32(0.0).unwrap();

        for item in &order.items {
            subtotal = subtotal + Decimal::from_i32(item.qty).unwrap();
        }

        subtotal
    }

    fn calc_taxes(subtotal: &Decimal, tax_rate: &Decimal) -> Decimal {
        subtotal * tax_rate
    }

    pub fn get_shipping(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: "Shipping".into(),
            description: "Flat rate shipping fee".into(),
            amount: format!("{:.02}", self.shipping).into(),
        }
    }

    pub fn get_taxes(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: "Taxes".into(),
            description: "".into(),
            amount: format!("{:.02}", self.taxes).into(),
        }
    }

    pub fn get_duty(&self) -> AuthorizeNetFee {
        AuthorizeNetFee {
            name: "".into(),
            description: "".into(),
            amount: "0".into(),
        }
    }
}
