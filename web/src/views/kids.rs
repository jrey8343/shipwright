use axum::response::{IntoResponse, Response};
use shipwright_db::entities::kids::Kid;
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{format, middlewares::flash::IncomingFlashes};

pub enum KidView {
    Index(ViewEngine<View>, Vec<Kid>, IncomingFlashes),
    Show(ViewEngine<View>, Kid, IncomingFlashes),
}

impl IntoResponse for KidView {
    fn into_response(self) -> Response {
        match self {
            KidView::Index(ViewEngine(v), kids, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "kids/index.html",
                        json!({ "kids": kids, "flashes": flashes }),
                    )
                    .into_response()
            }
            KidView::Show(ViewEngine(v), kid, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "kids/show.html",
                        json!({ "kid": kid, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
