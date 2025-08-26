use crate::server::types::ServerContext;
use std::sync::OnceLock;

static SHARED_CONTEXT: OnceLock<ServerContext> = OnceLock::new();

pub fn get_shared_context() -> &'static ServerContext {
    SHARED_CONTEXT.get_or_init(|| ServerContext::new())
}
