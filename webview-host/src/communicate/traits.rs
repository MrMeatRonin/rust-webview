use serde::{Deserialize, Serialize};

pub trait RequestHandler: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self, params: wry::Value);
}

/**
 * Deserialized from json Str
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    handler: String,
    params: wry::Value,
}
