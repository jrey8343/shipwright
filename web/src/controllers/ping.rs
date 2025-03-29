use axum::{Router, extract::State, http::StatusCode, routing::get};
use shipwright_db::{
    Entity,
    entities::invoices::{Invoice, InvoiceChangeset},
};
use uuid::Uuid;

use crate::{error::Error, state::AppState};

pub struct PingController;

impl PingController {
    pub fn router() -> Router<AppState> {
        Router::new()
            .route("/ping", get(PingController::ping))
            .route("/pong", get(PingController::pong))
    }

    pub async fn ping(State(state): State<AppState>) -> Result<StatusCode, Error> {
        let invoice = InvoiceChangeset {
            amount: Some(100.0),
        };
        let res = Invoice::create(invoice, &state.db_pool).await?;

        tracing::info!("Invoice created: {:?}", res);
        Ok(StatusCode::OK)
    }
    pub async fn pong(State(state): State<AppState>) -> Result<StatusCode, Error> {
        let id = Uuid::parse_str("d68f6ed5-43f4-492f-a272-36379bfb4930").unwrap();
        let invoice = InvoiceChangeset {
            amount: Some(300.0),
        };
        let res = Invoice::update(id, invoice, &state.db_pool).await?;

        tracing::info!("Invoice updated: {:?}", res);
        Ok(StatusCode::OK)
    }
    // pub async fn ping() -> StatusCode {
    //     StatusCode::OK
    // }
}
