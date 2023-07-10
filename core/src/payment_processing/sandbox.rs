use async_trait::async_trait;

#[derive(Clone)]
pub struct SandboxProcessor;

#[async_trait]
impl super::manager::PaymentProcessor for SandboxProcessor {
    async fn charge_card(
        &self,
        request: super::manager::ChargeCreditCardRequest,
    ) -> Result<super::manager::ChargeCreditCardResponse, Box<dyn std::error::Error>> {
        println!("Sandbox Payment Processor:\n\n{:#?}", request);

        Ok(super::manager::ChargeCreditCardResponse {
            transaction_id: "1234".into(),
        })
    }
}
