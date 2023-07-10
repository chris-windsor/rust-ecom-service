use async_trait::async_trait;
use rand::Rng;
use reqwest::header::CONTENT_TYPE;
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::Arc;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChargeCreditCardRequest {
    create_transaction_request: CreateTransactionRequest,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct CreateTransactionRequest {
    merchant_authentication: MerchantAuthentication,
    ref_id: Arc<str>,
    transaction_request: TransactionRequest,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct MerchantAuthentication {
    name: Arc<str>,
    transaction_key: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionRequest {
    transaction_type: Arc<str>,
    amount: Arc<str>,
    payment: Payment,
    line_items: Vec<()>,
    tax: AuthorizeNetFee,
    duty: AuthorizeNetFee,
    shipping: AuthorizeNetFee,
    po_number: Arc<str>,
    customer: AuthorizeNetCustomer,
    bill_to: Address,
    ship_to: Address,
    #[serde(rename(serialize = "customerIP"))]
    customer_ip: Arc<str>,
    // transaction_settings: TransactionSettings,
    user_fields: UserFields,
    // processing_options: ProcessingOptions,
    // subsequent_auth_information: SubsequentAuthInformation,
    authorization_indicator_type: AuthorizationIndicatorType,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthorizationIndicatorType {
    authorization_indicator: Arc<str>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Address {
    pub first_name: Arc<str>,
    pub last_name: Arc<str>,
    pub company: Arc<str>,
    pub address: Arc<str>,
    pub city: Arc<str>,
    pub state: Arc<str>,
    pub zip: Arc<str>,
    pub country: Arc<str>,
}

#[derive(Deserialize, Serialize)]
struct AuthorizeNetCustomer {
    id: Arc<str>,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct AuthorizeNetFee {
    pub amount: Arc<str>,
    pub name: Arc<str>,
    pub description: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct Payment {
    credit_card: CreditCard,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct CreditCard {
    pub card_number: Arc<str>,
    pub expiration_date: Arc<str>,
    pub card_code: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ProcessingOptions {
    is_subsequent_auth: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SubsequentAuthInformation {
    original_network_trans_id: Arc<str>,
    original_auth_amount: Arc<str>,
    reason: Arc<str>,
}

#[derive(Deserialize, Serialize)]
struct TransactionSettings {
    setting: TransactionSetting,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionSetting {
    setting_name: Arc<str>,
    setting_value: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserFields {
    user_field: Vec<UserField>,
}

impl UserFields {
    fn get_default() -> Self {
        UserFields { user_field: vec![] }
    }
}

#[derive(Deserialize, Serialize)]
struct UserField {
    name: Arc<str>,
    value: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChargeCreditCardResponse {
    transaction_response: TransactionResponse,
    pub ref_id: Arc<str>,
    messages: TransactionResponseResultMessages,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponse {
    response_code: Arc<str>,
    auth_code: Arc<str>,
    avs_result_code: Arc<str>,
    cvv_result_code: Arc<str>,
    trans_id: Arc<str>,
    #[serde(rename(deserialize = "refTransID"))]
    ref_trans_id: Arc<str>,
    trans_hash: Arc<str>,
    test_request: Arc<str>,
    account_number: Arc<str>,
    account_type: Arc<str>,
    messages: Vec<TransactionResponseMessage>,
    #[serde(default = "UserFields::get_default")]
    user_fields: UserFields,
    trans_hash_sha2: Arc<str>,
    #[serde(rename(deserialize = "SupplementalDataQualificationIndicator"))]
    supplemental_data_qualification_indicator: usize,
    network_trans_id: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct TransactionResponseResultMessages {
    result_code: Arc<str>,
    message: Vec<TransactionResponseResultMessage>,
}

#[derive(Deserialize, Serialize)]
struct TransactionResponseResultMessage {
    code: Arc<str>,
    text: Arc<str>,
}

#[derive(Deserialize, Serialize)]
struct TransactionResponseMessage {
    code: Arc<str>,
    description: Arc<str>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    messages: ErrorResponseMessages,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseMessages {
    result_code: Arc<str>,
    message: Vec<ErrorResponseMessage>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ErrorResponseMessage {
    code: Arc<str>,
    text: Arc<str>,
}

#[derive(Clone)]
pub struct AuthorizeNetProcessor;

#[async_trait]
impl super::manager::PaymentProcessor for AuthorizeNetProcessor {
    async fn charge_card(
        &self,
        request: super::manager::ChargeCreditCardRequest,
    ) -> Result<super::manager::ChargeCreditCardResponse, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();

        let merchant_id =
            env::var("AUTH_NET_MERCHANT_ID").expect("Could not find AUTH_NET_MERCHANT_ID in .env");
        let transaction_key = env::var("AUTH_NET_TRANSACTION_KEY")
            .expect("Could not find AUTH_NET_TRANSACTION_KEY in .env");

        let ref_id = request.invoice.id.to_string();
        let transaction_type = "authCaptureTransaction";
        let transaction_total = request.invoice.total.to_string();

        let taxes = request.invoice.get_taxes();
        let duties = request.invoice.get_duty();
        let shipping_fees = request.invoice.get_shipping();

        let po_number = rand::thread_rng().gen_range(0..100000).to_string();
        let customer_id = "";

        let charge_request = ChargeCreditCardRequest {
            create_transaction_request: CreateTransactionRequest {
                merchant_authentication: MerchantAuthentication {
                    name: merchant_id.into(),
                    transaction_key: transaction_key.into(),
                },
                ref_id: ref_id.into(),
                transaction_request: TransactionRequest {
                    transaction_type: transaction_type.into(),
                    amount: transaction_total.into(),
                    payment: Payment {
                        credit_card: CreditCard {
                            card_code: request.customer.credit_card.card_code.clone(),
                            card_number: request.customer.credit_card.card_number.clone(),
                            expiration_date: request.customer.credit_card.expiration_date.clone(),
                        },
                    },
                    line_items: vec![],
                    tax: taxes,
                    duty: duties,
                    shipping: shipping_fees,
                    po_number: po_number.into(),
                    customer: AuthorizeNetCustomer {
                        id: customer_id.into(),
                    },
                    bill_to: request.customer.billing_address.clone(),
                    ship_to: request.customer.shipping_address.clone(),
                    customer_ip: request.customer.ip_address.clone(),
                    // transaction_settings: TransactionSettings {
                    //     setting: TransactionSetting {
                    //         setting_name: "".into(),
                    //         setting_value: "".into(),
                    //     },
                    // },
                    user_fields: UserFields { user_field: vec![] },
                    // processing_options: ProcessingOptions {
                    //     is_subsequent_auth: "true".into(),
                    // },
                    // subsequent_auth_information: SubsequentAuthInformation {
                    //     original_auth_amount: "".into(),
                    //     original_network_trans_id: "".into(),
                    //     reason: "resubmission".into(),
                    // },
                    authorization_indicator_type: AuthorizationIndicatorType {
                        authorization_indicator: "final".into(),
                    },
                },
            },
        };

        let response = client
            .post("https://apitest.authorize.net/xml/v1/request.api")
            .header(CONTENT_TYPE, "application/json")
            .json(&charge_request)
            .send()
            .await?
            .text()
            .await?;

        // Authorize.NET returns a ZWSP at the start of the JSON response
        let response = response.replace("\u{feff}", "");

        let response: ChargeCreditCardResponse = serde_json::from_str(&response)?;
        Ok(super::manager::ChargeCreditCardResponse {
            transaction_id: response.ref_id,
        })
    }
}
