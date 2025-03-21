use axum::response::{IntoResponse, Response};
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::format;
use crate::middlewares::flash::IncomingFlashes;

pub enum RegisterConfirmView {
    Index(ViewEngine<View>, IncomingFlashes),
}

impl IntoResponse for RegisterConfirmView {
    fn into_response(self) -> Response {
        match self {
            RegisterConfirmView::Index(ViewEngine(v), IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "auth/register_confirm/index.html",
                        json!({"flashes": flashes}),
                    )
                    .into_response()
            }
        }
    }
}
