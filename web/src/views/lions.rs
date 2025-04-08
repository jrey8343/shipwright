use axum::response::{IntoResponse, Response};
use shipwright_db::entities::lions::Lion;
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{format, middlewares::flash::IncomingFlashes};

pub enum LionView {
    Index(ViewEngine<View>, Vec<Lion>, IncomingFlashes),
    Show(ViewEngine<View>, Lion, IncomingFlashes),
}

impl IntoResponse for LionView {
    fn into_response(self) -> Response {
        match self {
            LionView::Index(ViewEngine(v), lions, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "lions/index.html",
                        json!({ "lions": lions, "flashes": flashes }),
                    )
                    .into_response()
            }
            LionView::Show(ViewEngine(v), lion, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "lions/show.html",
                        json!({ "lion": lion, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
