use serde::{Deserialize, Serialize};

pub trait RequestHandler: Send + Sync {
    fn name(&self) -> &str;
    fn handle(&self, params: wry::Value) -> anyhow::Result<()>;
}

/**
 * Deserialized from json Str, a request must be responsed with a request-ack shared the id.
 */
#[derive(Debug, Serialize, Deserialize)]
pub struct Request {
    id: String,
    handler: String,
    params: wry::Value,
}
