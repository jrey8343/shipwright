use axum::response::{IntoResponse, Response};
use {{ db_crate_name }}::entities::{{ entity_plural_name }}::{{ entity_struct_name }};
use shipwright_ui::view_engine::{View, ViewEngine};
use serde_json::json;

use crate::{format, middlewares::flash::IncomingFlashes};

pub enum {{ entity_struct_name }}View {
    Index(ViewEngine<View>, Vec<{{ entity_struct_name }}>, IncomingFlashes),
    Show(ViewEngine<View>, {{ entity_struct_name }}, IncomingFlashes),
}

impl IntoResponse for {{ entity_struct_name }}View {
    fn into_response(self) -> Response {
        match self {
            {{ entity_struct_name }}View::Index(ViewEngine(v), {{ entity_plural_name }}, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "{{ entity_plural_name }}/index.html",
                        json!({ "{{ entity_plural_name }}": {{ entity_plural_name }}, "flashes": flashes }),
                    )
                    .into_response()
            }
            {{ entity_struct_name }}View::Show(ViewEngine(v), {{ entity_singular_name }}, IncomingFlashes { flashes, .. }) => {
                format::render()
                    .view(
                        &v,
                        "{{ entity_plural_name }}/show.html",
                        json!({ "{{ entity_singular_name }}": {{ entity_singular_name }}, "flashes": flashes }),
                    )
                    .into_response()
            }
        }
    }
}
