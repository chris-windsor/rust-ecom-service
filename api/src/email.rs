use aws_sdk_sesv2::{
    types::{Body, Content, Destination, EmailContent, Message},
    Client,
};

const SENDER_EMAIL_ADDRESS: &str = "awstest@chriswindsor.dev";

pub async fn send_password_reset_email(to: &String, content: &String) {
    let config = aws_config::load_from_env().await;
    let client = Client::new(&config);
    let dest = Destination::builder().to_addresses(to).build();
    let subject_content = Content::builder()
        .data("Password Reset Inquiry")
        .charset("UTF-8")
        .build();
    let body_content = Content::builder().data(content).charset("UTF-8").build();
    let body = Body::builder().html(body_content).build();

    let msg = Message::builder()
        .subject(subject_content)
        .body(body)
        .build();

    let email_content = EmailContent::builder().simple(msg).build();

    let _ = client
        .send_email()
        .from_email_address(SENDER_EMAIL_ADDRESS)
        .destination(dest)
        .content(email_content)
        .send()
        .await;
}
