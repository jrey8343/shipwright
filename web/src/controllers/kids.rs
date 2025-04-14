use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use shipwright_db::{
    Entity,
    entities::kids::Kid,
    entities::kids::KidChangeset,
};
use shipwright_ui::view_engine::{View, ViewEngine};

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::kids::KidView,
};

use super::controller::Controller;

pub struct KidController;

#[async_trait]
impl Controller for KidController {
    type Id = String;

    type View = KidView;

    type EntityChangeset = KidChangeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/kids", get(Self::read_all).post(Self::create))
            .route("/kids/batch", post(Self::create_batch))
            .route(
                "/kids/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let kids = Kid::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), KidView::Index(v, kids, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let kid = Kid::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created new kid")),
            Redirect::to(&format!("/kids/{}", kid.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Kid::create_batch(records, &app_state.db_pool).await?;

        Ok((flash.success(&format!("✅ created kids")), Redirect::to("/kids")))
    }

    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let kid = Kid::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), KidView::Show(v, kid, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let kid = Kid::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ updated kid")),
            Redirect::to(&format!("/kids/{}", kid.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _kid = Kid::delete(id, &app_state.db_pool).await?;

        Ok((flash.info(&format!("deleted kid")), Redirect::to("/kids")))
    }
}
