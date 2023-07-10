use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::ecommerce::{Customer, Invoice};

#[derive(Debug, Deserialize, Serialize)]
pub struct ChargeCreditCardRequest {
    pub transaction_amount: u32,
    pub order_number: Arc<str>,
    pub customer: Customer,
    pub invoice: Invoice,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ChargeCreditCardResponse {
    pub transaction_id: Arc<str>,
}

#[async_trait]
pub trait PaymentProcessor: Send + Sync {
    async fn charge_card(
        &self,
        request: ChargeCreditCardRequest,
    ) -> Result<ChargeCreditCardResponse, Box<dyn std::error::Error>>;
}

#[async_trait]
pub trait PaymentProcessorDyn: PaymentProcessor + Clone {}

impl<T> PaymentProcessorDyn for T where T: PaymentProcessor + Clone {}

pub fn get_payment_processor() -> Arc<dyn PaymentProcessor> {
    #[cfg(feature = "authorize_net")]
    {
        use super::authorize_net::AuthorizeNetProcessor;
        Arc::new(AuthorizeNetProcessor)
    }

    #[cfg(feature = "fortis_pay")]
    {
        use super::fortis_pay::FortisPayProcessor;
        Arc::new(FortisPayProcessor)
    }

    #[cfg(feautre = "stripe")]
    {
        use super::stripe::StripeProcessor;
        Arc::new(StripeProcessor)
    }

    #[cfg(not(any(feature = "authorize_net", feature = "fortis_pay", feature = "stripe")))]
    {
        use super::sandbox::SandboxProcessor;
        Arc::new(SandboxProcessor)
    }
}
