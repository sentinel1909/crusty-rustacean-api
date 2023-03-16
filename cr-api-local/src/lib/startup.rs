//! src/lib/startup.rs

// dependcencies, external and internal
use axum::{
    routing::{get, post, IntoMakeService},
    Router, Server,
};
use hyper::server::conn::AddrIncoming;
use sqlx::PgPool;
use std::net::TcpListener;
use axum_tracing_opentelemetry::{opentelemetry_tracing_layer, response_with_trace_layer};


use crate::routes::health_check::health_check;
use crate::routes::subscriptions::subscribe;

pub type App = Server<AddrIncoming, IntoMakeService<Router>>;

// run function
pub fn run(listener: TcpListener, pool: PgPool) -> hyper::Result<App> {
    // routes and their corresponding handlers
    let app = Router::new()
        .route("/health_check", get(health_check))
        .route("/subscriptions", post(subscribe))
        .layer(response_with_trace_layer())
        .layer(opentelemetry_tracing_layer())
        .with_state(pool);
    let server = axum::Server::from_tcp(listener)?.serve(app.into_make_service());
    Ok(server)
}
