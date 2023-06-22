use axum::{
    extract::State,
    response::{sse::Event, IntoResponse, Sse},
    Extension, Json,
};
use entity::account;
use futures::Stream;
use http::StatusCode;
use lazy_static::lazy_static;
use lemon_tree_core::AppState;
use rust_decimal::{prelude::FromPrimitive, Decimal};
use serde_json::json;
use std::{
    collections::VecDeque,
    convert::Infallible,
    sync::{Arc, Mutex},
    time::Duration,
};

use crate::{
    authorize_net::{Address, ChargeCreditCardRequest, CreditCard},
    ecommerce::{Customer, Invoice, Order, OrderAdjustments},
    priveleges::check_admin,
    request::NewOrder,
};

lazy_static! {
    static ref UDPATE_QUEUE: Arc<Mutex<VecDeque<String>>> =
        Arc::new(Mutex::new(VecDeque::from([])));
}

pub async fn process_order(
    State(data): State<Arc<AppState>>,
    Json(req_order): Json<NewOrder>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let processing_msg = format!("Processing order");
    UDPATE_QUEUE
        .lock()
        .unwrap()
        .push_back(processing_msg.to_owned());

    let invoice_order = Order { items: vec![] };

    let invoice = Invoice::create(
        &invoice_order,
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
        email: req_order.customer_details.email_address,
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
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, (StatusCode, Json<serde_json::Value>)>
{
    if let Err(error) = check_admin(&user) {
        return Err(error);
    }

    let stream = async_stream::stream! {
        let mut interval = tokio::time::interval(Duration::from_millis(100));
        loop {
            interval.tick().await;
            let latest_update = UDPATE_QUEUE.lock().unwrap().pop_front();
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
