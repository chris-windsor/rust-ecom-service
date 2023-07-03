use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use chrono::Utc;
use entity::{prelude::*, *};
use futures::Stream;
use http::StatusCode;
use lemon_tree_core::{
    sea_orm::{ActiveValue, EntityTrait},
    AppState,
};
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};

use crate::{
    authorize_net::{Address, ChargeCreditCardRequest, CreditCard},
    ecommerce::{Customer, Invoice, OrderAdjustments},
    priveleges::check_admin,
    request::NewOrder,
};

pub async fn process_order(
    State(data): State<Arc<AppState>>,
    Json(req_order): Json<NewOrder>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let processing_msg = format!("Processing order");
    data.message_channel
        .lock()
        .unwrap()
        .push_back(processing_msg.into());

    let invoice = Invoice::create(
        vec![],
        OrderAdjustments {
            shipping_fee: Decimal::from_f32(5.0).unwrap(),
            tax_rate: Decimal::from_f32(0.0715).unwrap(),
        },
    );

    let customer_address = Address {
        first_name: req_order.customer_details.first_name.clone(),
        last_name: req_order.customer_details.last_name.clone(),
        company: "".into(),
        address: req_order.customer_details.street_address_1,
        city: req_order.customer_details.city,
        state: req_order.customer_details.state,
        zip: req_order.customer_details.zip_code,
        country: "US".into(),
    };

    let customer = Customer {
        first_name: req_order.customer_details.first_name,
        last_name: req_order.customer_details.last_name,
        email: req_order.customer_details.email_address.clone(),
        phone_number: req_order.customer_details.phone_number,
        ip_address: "".into(),
        billing_address: customer_address.clone(),
        shipping_address: customer_address.clone(),
        credit_card: CreditCard {
            card_number: req_order.payment_details.credit_card_number,
            expiration_date: req_order.payment_details.credit_card_expiry,
            card_code: req_order.payment_details.credit_card_cvv,
        },
    };

    let order_billing_address = order_address::ActiveModel {
        first_name: ActiveValue::Set(customer.first_name.to_string()),
        last_name: ActiveValue::Set(customer.last_name.to_string()),
        street: ActiveValue::Set(customer_address.address.to_string()),
        city: ActiveValue::Set(customer_address.city.to_string()),
        state: ActiveValue::Set(customer_address.state.to_string()),
        postal_code: ActiveValue::Set(customer_address.zip.to_string()),
        phone_number: ActiveValue::Set(customer.phone_number.to_string()),
        ..Default::default()
    };

    let order_address = OrderAddress::insert(order_billing_address)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let new_order = order::ActiveModel {
        email: ActiveValue::Set(Some(req_order.customer_details.email_address.to_string())),
        billing_address_id: ActiveValue::Set(order_address.id),
        shipping_address_id: ActiveValue::Set(order_address.id),
        creation_date: ActiveValue::Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    Order::insert(new_order)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    let auth_net_req = ChargeCreditCardRequest::create(&invoice, &customer)
        .await
        .unwrap();

    Ok(Json(
        json!({ "invoice": json!(invoice), "transaction": json!(auth_net_req) }),
    ))
}

pub async fn list_orders(
    Extension(user): Extension<account::Model>,
    State(data): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    Ok(())
}

pub async fn live_order_events(
    Extension(user): Extension<account::Model>,
    State(data): State<Arc<AppState>>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<serde_json::Value>)>
{
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let latest_update = data.message_channel.lock().unwrap().pop_front();
            match latest_update {
                Some(data) => {
                    yield Ok(Event::default().data(data));
                },
                None => {}
            }
        }
    };

    Ok(Sse::new(stream))
}
