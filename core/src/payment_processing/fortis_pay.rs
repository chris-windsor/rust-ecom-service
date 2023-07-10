use async_trait::async_trait;
use http::{header::CONTENT_TYPE, HeaderName};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

#[derive(Debug, Deserialize, Serialize)]
struct ChargeCreditCardRequest {
    transaction_amount: u32,
    order_number: Arc<str>,
    account_number: Arc<str>,
    exp_date: Arc<str>,
    cvv: Arc<str>,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChargeCreditCardResponse {
    data: ChargeCreditCardResponseData,
}

#[derive(Debug, Deserialize, Serialize)]
struct ChargeCreditCardResponseData {
    id: Arc<str>,
    last_four: Arc<str>,
    account_type: Arc<str>,
    transaction_batch_id: Arc<str>,
}

#[derive(Clone)]
pub struct FortisPayProcessor;

#[async_trait]
impl super::manager::PaymentProcessor for FortisPayProcessor {
    async fn charge_card(
        &self,
        request: super::manager::ChargeCreditCardRequest,
    ) -> Result<super::manager::ChargeCreditCardResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let user_id =
            env::var("FORTIS_PAY_USER_ID").expect("Could not find FORTIS_PAY_USER_ID in .env");
        let user_api_key = env::var("FORTIS_PAY_USER_API_KEY")
            .expect("Could not find FORTIS_PAY_USER_API_KEY in .env");
        let developer_id = env::var("FORTIS_PAY_DEVELOPER_ID")
            .expect("Could not find FORTIS_PAY_DEVELOPER_ID in .env");

        let actual_req = ChargeCreditCardRequest {
            transaction_amount: request.transaction_amount,
            order_number: request.order_number,
            account_number: request.customer.credit_card.card_number,
            exp_date: request.customer.credit_card.expiration_date,
            cvv: request.customer.credit_card.card_code,
        };

        let response = client
            .post("https://api.sandbox.fortis.tech/v1/transactions/cc/sale/keyed")
            .header(HeaderName::from_static("user-id"), user_id)
            .header(HeaderName::from_static("user-api-key"), user_api_key)
            .header(HeaderName::from_static("developer-id"), developer_id)
            .header(CONTENT_TYPE, "application/json")
            .json(&actual_req)
            .send()
            .await?
            .text()
            .await?;

        let response: ChargeCreditCardResponse = serde_json::from_str(&response)?;
        Ok(super::manager::ChargeCreditCardResponse {
            transaction_id: response.data.transaction_batch_id,
        })
    }
}
