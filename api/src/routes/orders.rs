use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use chrono::Utc;
use entity::{prelude::*, *};
use futures::Stream;
use http::StatusCode;
use rand::Rng;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use rust_ecom_service_core::{
    ecommerce::{Customer, Invoice, OrderAdjustments},
    payment_processing::{authorize_net, manager::ChargeCreditCardRequest},
    sea_orm::{ActiveValue, EntityTrait},
    AppState,
};
use serde_json::json;
use std::{convert::Infallible, sync::Arc, time::Duration};

use crate::{priveleges::check_admin, request::NewOrder};

pub async fn process_order(
    State(data): State<Arc<AppState>>,
    Json(req_order): Json<NewOrder>,
) -> Result<(StatusCode, Json<serde_json::Value>), (StatusCode, Json<serde_json::Value>)> {
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

    let customer_address = authorize_net::Address {
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
        credit_card: authorize_net::CreditCard {
            card_number: req_order.payment_details.credit_card_number,
            expiration_date: req_order.payment_details.credit_card_expiry,
            card_code: req_order.payment_details.credit_card_cvv,
        },
    };

    let order_billing_address = address::ActiveModel {
        first_name: ActiveValue::Set(customer.first_name.to_string()),
        last_name: ActiveValue::Set(customer.last_name.to_string()),
        street: ActiveValue::Set(customer_address.address.to_string()),
        city: ActiveValue::Set(customer_address.city.to_string()),
        state: ActiveValue::Set(customer_address.state.to_string()),
        postal_code: ActiveValue::Set(customer_address.zip.to_string()),
        phone_number: ActiveValue::Set(customer.phone_number.to_string()),
        ..Default::default()
    };

    let order_address = Address::insert(order_billing_address)
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
        tax_amount: ActiveValue::Set(Decimal::from_f32(0.0).unwrap()),
        shipping_amount: ActiveValue::Set(Decimal::from_f32(0.0).unwrap()),
        total_amount: ActiveValue::Set(Decimal::from_f32(0.0).unwrap()),
        email: ActiveValue::Set(req_order.customer_details.email_address.to_string()),
        billing_address_id: ActiveValue::Set(order_address.id),
        shipping_address_id: ActiveValue::Set(order_address.id),
        creation_date: ActiveValue::Set(Utc::now().naive_utc()),
        ..Default::default()
    };

    let new_order = Order::insert(new_order)
        .exec_with_returning(&data.db)
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format!("Database error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    for item in req_order.order_items {
        let item = order_item::ActiveModel {
            order_id: ActiveValue::Set(new_order.id),
            qty: ActiveValue::Set(item.qty),
            price: ActiveValue::Set(Decimal::from_f32(0.0).unwrap()),
            ..Default::default()
        };

        OrderItem::insert(item)
            .exec_with_returning(&data.db)
            .await
            .map_err(|e| {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": format!("Database error: {}", e),
                });
                (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
            })?;
    }

    let transaction_req = ChargeCreditCardRequest {
        transaction_amount: 1000,
        order_number: rand::thread_rng().gen_range(1000..10000).to_string().into(),
        customer: customer.clone(),
        invoice: invoice.clone(),
    };

    let transaction_req = data
        .payment_processor
        .charge_card(transaction_req)
        .await
        .map_err(|e| {
            let error_response = json!({
                "status": "fail",
                "message": format!("Transaction processing error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;

    Ok((
        StatusCode::OK,
        Json(json!({ "invoice": json!(invoice), "transaction": json!(transaction_req) })),
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
