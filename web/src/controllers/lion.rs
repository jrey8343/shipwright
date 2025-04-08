use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use shipwright_db::{Entity, entities::lions::Lion, entities::lions::LionChangeset};
use shipwright_ui::view_engine::{View, ViewEngine};

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::lions::LionView,
};

use super::Controller;

pub struct LionController;

#[async_trait]
impl Controller for LionController {
    type Id = i64;

    type View = LionView;

    type EntityChangeset = LionChangeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/lions", get(Self::read_all).post(Self::create))
            .route("/lions/batch", post(Self::create_batch))
            .route(
                "/lions/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let lions = Lion::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), LionView::Index(v, lions, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let lion = Lion::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created new lion")),
            Redirect::to(&format!("/lions/{}", lion.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Lion::create_batch(records, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created lions")),
            Redirect::to("/lions"),
        ))
    }

    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let lion = Lion::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), LionView::Show(v, lion, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let lion = Lion::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ updated lion")),
            Redirect::to(&format!("/lions/{}", lion.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _lion = Lion::delete(id, &app_state.db_pool).await?;

        Ok((flash.info(&format!("deleted lion")), Redirect::to("/lions")))
    }
}
