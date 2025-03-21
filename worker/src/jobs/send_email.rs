use apalis::prelude::Data;
use shipwright_mailer::{EmailClient, EmailPayload};

pub async fn job(
    job: EmailPayload,
    email_client: Data<EmailClient>,
) -> Result<(), shipwright_mailer::Error> {
    email_client.send_email(job).await?;

    Ok(())
}
