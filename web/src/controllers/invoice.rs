use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use shipwright_db::{
    Entity,
    entities::invoices::Invoice,
    entities::invoices::InvoiceChangeset,
};
use shipwright_ui::view_engine::{View, ViewEngine};

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::invoices::InvoiceView,
};

use super::Controller;

pub struct InvoiceController;

#[async_trait]
impl Controller for InvoiceController {
    type Id = String;

    type View = InvoiceView;

    type EntityChangeset = InvoiceChangeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/invoices", get(Self::read_all).post(Self::create))
            .route("/invoices/batch", post(Self::create_batch))
            .route(
                "/invoices/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let invoices = Invoice::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), InvoiceView::Index(v, invoices, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let invoice = Invoice::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created new invoice")),
            Redirect::to(&format!("/invoices/{}", invoice.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Invoice::create_batch(records, &app_state.db_pool).await?;

        Ok((flash.success(&format!("✅ created invoices")), Redirect::to("/invoices")))
    }

    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let invoice = Invoice::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), InvoiceView::Show(v, invoice, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let invoice = Invoice::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ updated invoice")),
            Redirect::to(&format!("/invoices/{}", invoice.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _invoice = Invoice::delete(id, &app_state.db_pool).await?;

        Ok((flash.info(&format!("deleted invoice")), Redirect::to("/invoices")))
    }
}
