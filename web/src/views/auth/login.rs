use axum::response::{IntoResponse, Response};
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{
    format::{self},
    middlewares::flash::IncomingFlashes,
};

pub enum LoginView {
    Index(ViewEngine<View>, IncomingFlashes, Option<String>),
}

impl IntoResponse for LoginView {
    fn into_response(self) -> Response {
        match self {
            LoginView::Index(ViewEngine(v), IncomingFlashes { flashes, .. }, next) => {
                format::render()
                    .view(
                        &v,
                        "auth/login/index.html",
                        json!({ "flashes": flashes, "next": next}),
                    )
                    .into_response()
            }
        }
    }
}
