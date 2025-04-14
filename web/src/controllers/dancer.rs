use async_trait::async_trait;
use axum::{
    Form, Router,
    extract::{Path, State},
    response::Redirect,
    routing::{get, post},
};
use shipwright_db::{
    Entity,
    entities::dancers::Dancer,
    entities::dancers::DancerChangeset,
};
use shipwright_ui::view_engine::{View, ViewEngine};

use crate::{
    error::Error,
    middlewares::flash::{Flash, IncomingFlashes},
    state::AppState,
    views::dancers::DancerView,
};

use super::controller::Controller;

pub struct DancerController;

#[async_trait]
impl Controller for DancerController {
    type Id = String;

    type View = DancerView;

    type EntityChangeset = DancerChangeset;

    type Error = Error;

    fn router() -> Router<AppState> {
        Router::new()
            .route("/dancers", get(Self::read_all).post(Self::create))
            .route("/dancers/batch", post(Self::create_batch))
            .route(
                "/dancers/{id}",
                get(Self::read_one).put(Self::update).delete(Self::delete),
            )
    }

    async fn read_all(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let dancers = Dancer::load_all(&app_state.db_pool).await?;

        Ok((flashes.clone(), DancerView::Index(v, dancers, flashes)))
    }

    async fn create(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(record): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let dancer = Dancer::create(record, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ created new dancer")),
            Redirect::to(&format!("/dancers/{}", dancer.id)),
        ))
    }

    async fn create_batch(
        flash: Flash,
        State(app_state): State<AppState>,
        Form(records): Form<Vec<Self::EntityChangeset>>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _records = Dancer::create_batch(records, &app_state.db_pool).await?;

        Ok((flash.success(&format!("✅ created dancers")), Redirect::to("/dancers")))
    }

    async fn read_one(
        v: ViewEngine<View>,
        flashes: IncomingFlashes,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(IncomingFlashes, Self::View), Self::Error> {
        let dancer = Dancer::load(id, &app_state.db_pool).await?;

        Ok((flashes.clone(), DancerView::Show(v, dancer, flashes)))
    }

    async fn update(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
        Form(form): Form<Self::EntityChangeset>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let dancer = Dancer::update(id, form, &app_state.db_pool).await?;

        Ok((
            flash.success(&format!("✅ updated dancer")),
            Redirect::to(&format!("/dancers/{}", dancer.id)),
        ))
    }

    async fn delete(
        flash: Flash,
        Path(id): Path<Self::Id>,
        State(app_state): State<AppState>,
    ) -> Result<(Flash, Redirect), Self::Error> {
        let _dancer = Dancer::delete(id, &app_state.db_pool).await?;

        Ok((flash.info(&format!("deleted dancer")), Redirect::to("/dancers")))
    }
}
