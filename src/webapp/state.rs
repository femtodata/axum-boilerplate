use std::{collections::HashMap, ops::Deref, sync::Arc};

use axum::extract::FromRef;
use axum_extra::extract::cookie::Key;
use diesel::{
    PgConnection,
    r2d2::{ConnectionManager, Pool},
};
use tera::Tera;

// AppState shenanigans, because CookieJar
#[derive(Clone)]
pub struct AppState(pub Arc<InnerState>);

pub struct InnerState {
    pub tera: Tera,
    pub key: Key,
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

// automatically get to InnerState
impl Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.0.key.clone()
    }
}

impl FromRef<AppState> for Tera {
    fn from_ref(state: &AppState) -> Self {
        state.0.tera.clone()
    }
}

impl FromRef<AppState> for Pool<ConnectionManager<PgConnection>> {
    fn from_ref(state: &AppState) -> Self {
        state.0.pool.clone()
    }
}
