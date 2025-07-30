use anyhow::Ok;
use serde::{Deserialize, Serialize};

use crate::communicate::RequestHandler;

pub struct WebviewHandler {}

#[derive(Debug, Serialize, Deserialize)]
struct Invoke {
    method: String,
}

impl RequestHandler for WebviewHandler {
    fn name(&self) -> &str {
        "webview"
    }

    fn handle(&self, params: wry::Value) -> anyhow::Result<()> {
        let invoke: Invoke = serde_json::from_value(params)?;
        
        Ok(())
    }
}
