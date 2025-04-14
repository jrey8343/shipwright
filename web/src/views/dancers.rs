use axum::response::{IntoResponse, Response};
use shipwright_db::entities::dancers::Dancer;
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{format, middlewares::flash::IncomingFlashes};

pub enum DancerView {
    Index(ViewEngine<View>, Vec<Dancer>, IncomingFlashes),
    Show(ViewEngine<View>, Dancer, IncomingFlashes),
}

impl IntoResponse for DancerView {
    fn into_response(self) -> Response {
        match self {
            DancerView::Index(ViewEngine(v), dancers, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "dancers/index.html",
                        json!({ "dancers": dancers, "flashes": flashes }),
                    )
                    .into_response()
            }
            DancerView::Show(ViewEngine(v), dancer, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "dancers/show.html",
                        json!({ "dancer": dancer, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
