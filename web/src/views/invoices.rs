use axum::response::{IntoResponse, Response};
use shipwright_db::entities::invoices::Invoice;
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{format, middlewares::flash::IncomingFlashes};

pub enum InvoiceView {
    Index(ViewEngine<View>, Vec<Invoice>, IncomingFlashes),
    Show(ViewEngine<View>, Invoice, IncomingFlashes),
}

impl IntoResponse for InvoiceView {
    fn into_response(self) -> Response {
        match self {
            InvoiceView::Index(ViewEngine(v), invoices, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "invoices/index.html",
                        json!({ "invoices": invoices, "flashes": flashes }),
                    )
                    .into_response()
            }
            InvoiceView::Show(ViewEngine(v), invoice, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "invoices/show.html",
                        json!({ "invoice": invoice, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
