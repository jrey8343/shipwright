use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use {{ db_crate_name }}::{
    Entity,
    entities::{{ entity_plural_name }}::{{ entity_struct_name }},
    entities::{{ entity_plural_name}}::{{ entity_struct_name }}Changeset,
};
use shipwright_ui::view_engine::{View, ViewEngine};

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::{{ entity_plural_name }}::{{ entity_struct_name }}View,
};

use super::Controller;

pub struct {{ entity_struct_name }}Controller;

#[async_trait]
impl Controller for {{ entity_struct_name }}Controller {
    type Id = String;

    type View = {{ entity_struct_name }}View;

    type EntityChangeset = {{ entity_struct_name }}Changeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/{{ entity_plural_name }}", get(Self::read_all).post(Self::create))
            .route("/{{ entity_plural_name }}/batch", post(Self::create_batch))
            .route(
                "/{{ entity_plural_name }}/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let {{ entity_plural_name }} = {{ entity_struct_name }}::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), {{ entity_struct_name }}View::Index(v, {{ entity_plural_name }}, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let {{ entity_singular_name }} = {{ entity_struct_name }}::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created new {{ entity_singular_name }}")),
            Redirect::to(&format!("/{{ entity_plural_name }}/{}", {{ entity_singular_name }}.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = {{ entity_struct_name }}::create_batch(records, &app_state.db_pool).await?;

        Ok((flash.success(&format!("✅ created {{ entity_plural_name }}")), Redirect::to("/{{ entity_plural_name }}")))
    }

    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let {{ entity_singular_name }} = {{ entity_struct_name }}::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), {{ entity_struct_name }}View::Show(v, {{ entity_singular_name }}, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let {{ entity_singular_name }} = {{ entity_struct_name }}::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ updated {{ entity_singular_name }}")),
            Redirect::to(&format!("/{{ entity_plural_name }}/{}", {{ entity_singular_name }}.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _{{ entity_singular_name }} = {{ entity_struct_name }}::delete(id, &app_state.db_pool).await?;

        Ok((flash.info(&format!("deleted {{ entity_singular_name }}")), Redirect::to("/{{ entity_plural_name }}")))
    }
}
