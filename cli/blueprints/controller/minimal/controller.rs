use crate::{error::Error, format, state::SharedAppState};
use axum::{extract::State, response::Response};
use my_app_ui::{View, ViewEngine};
use serde_json::json;

#[axum::debug_handler]
pub async fn action(
    State(app_state): State<SharedAppState>,
    ViewEngine(v): ViewEngine<View>,
) -> Result<Response, Error> {
    todo!("Implement action handler");
    format::view(&v, todo!("path within /templates"), json!({}))
}
