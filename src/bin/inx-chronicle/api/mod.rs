// Copyright 2022 IOTA Stiftung
// SPDX-License-Identifier: Apache-2.0

//! Contains routes that can be used to access data stored by Chronicle
//! as well as the health of the application and analytics.

mod extractors;

#[cfg(feature = "stardust")]
pub mod stardust;

mod error;
mod secret_key;
#[macro_use]
mod responses;
mod auth;
mod config;
mod router;
mod routes;

use axum::{Extension, Server};
use chronicle::db::MongoDb;
use hyper::Method;
use tower_http::{
    catch_panic::CatchPanicLayer,
    cors::{Any, CorsLayer},
    trace::TraceLayer,
};
use tracing::{info, log::trace};

pub use self::{
    config::ApiConfig,
    error::{ApiError, ConfigError},
    secret_key::SecretKey,
};
use self::{config::ApiData, routes::routes};
use crate::shutdown::ShutdownSignal;

pub const DEFAULT_PAGE_SIZE: usize = 100;

/// The result of a request to the api
pub type ApiResult<T> = Result<T, ApiError>;

/// The Chronicle API actor
#[derive(Debug)]
pub struct ApiWorker {
    db: MongoDb,
    api_data: ApiData,
}

impl ApiWorker {
    /// Create a new Chronicle API actor from a mongo connection.
    pub fn new(db: &MongoDb, config: &ApiConfig) -> Result<Self, ConfigError> {
        Ok(Self {
            db: db.clone(),
            api_data: config.clone().try_into()?,
        })
    }

    pub async fn start(&self, shutdown: ShutdownSignal) -> Result<(), ApiError> {
        info!("Starting API server on port `{}`", self.api_data.port);
        let port = self.api_data.port;
        let routes = routes()
            .layer(Extension(self.db.clone()))
            .layer(Extension(self.api_data.clone()))
            .layer(CatchPanicLayer::new())
            .layer(TraceLayer::new_for_http())
            .layer(
                CorsLayer::new()
                    .allow_origin(self.api_data.allow_origins.clone())
                    .allow_methods(vec![Method::GET, Method::OPTIONS])
                    .allow_headers(Any)
                    .allow_credentials(false),
            );

        let _ = Server::bind(&([0, 0, 0, 0], port).into())
            .serve(routes.into_make_service())
            .with_graceful_shutdown(shutdown.listen())
            .await?;

        // TODO: consider sending shutdown signal, depending on error or cause for shutdown.
        trace!("Shutting down API worker.");

        Ok(())
    }
}
