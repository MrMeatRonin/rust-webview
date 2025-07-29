use crate::communicate::RequestHandler;

pub struct WebviewHandler {}

struct Request {}

impl RequestHandler for WebviewHandler {
    fn name(&self) -> &str {
        "webview"
    }

    fn handle(&self, params: wry::Value) {
        
    }
}
