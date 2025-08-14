// =====================================================
// FILE: src/server/middleware.rs - CUSTOM MIDDLEWARE
// =====================================================

use crate::server::ServerInfo;
use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage,
};
use futures::future::{ready, Ready};
use std::sync::{Arc, Mutex};

/// Middleware um Server-Info in Requests verfügbar zu machen
pub struct ServerInfoMiddleware {
    server_info: Arc<Mutex<ServerInfo>>,
}

impl ServerInfoMiddleware {
    pub fn new(server_info: Arc<Mutex<ServerInfo>>) -> Self {
        Self { server_info }
    }
}

impl<S, B> Transform<S, ServiceRequest> for ServerInfoMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = ServerInfoMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ServerInfoMiddlewareService {
            service,
            server_info: Arc::clone(&self.server_info),
        }))
    }
}

pub struct ServerInfoMiddlewareService<S> {
    service: S,
    server_info: Arc<Mutex<ServerInfo>>,
}

impl<S, B> Service<ServiceRequest> for ServerInfoMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Server-Info in Request Extensions speichern
        req.extensions_mut().insert(Arc::clone(&self.server_info));

        // Request weiterleiten
        self.service.call(req)
    }
}
