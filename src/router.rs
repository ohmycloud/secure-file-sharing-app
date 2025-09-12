use crate::{
    AppState,
    handler::{
        auth::auth_handler, file::file_handle, file_query::get_file_list_handler,
        user::users_handler,
    },
    middleware,
};
use axum::{Extension, Router};
use std::sync::Arc;
use tower_http::trace::TraceLayer;

pub fn create_router(app_state: Arc<AppState>) -> Router {
    let api_router = Router::new()
        .nest("/auth", auth_handler())
        .nest(
            "/users",
            users_handler().layer(axum::middleware::from_fn(middleware::auth)),
        )
        .nest(
            "/file",
            file_handle().layer(axum::middleware::from_fn(middleware::auth)),
        )
        .nest(
            "/list",
            get_file_list_handler().layer(axum::middleware::from_fn(middleware::auth)),
        )
        .layer(TraceLayer::new_for_http())
        .layer(Extension(app_state));

    Router::new().nest("/api", api_router)
}
