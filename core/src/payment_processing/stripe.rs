use async_trait::async_trait;

#[derive(Clone)]
pub struct StripeProcessor;

#[async_trait]
impl super::manager::PaymentProcessor for StripeProcessor {
    async fn charge_card(
        &self,
        request: super::manager::ChargeCreditCardRequest,
    ) -> Result<super::manager::ChargeCreditCardResponse, Box<dyn std::error::Error>> {
        todo!()
    }
}
