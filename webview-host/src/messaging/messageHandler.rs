pub struct MessageHandler {
    connection: Connection,
    // Other handler state
}

impl MessageHandler {
    pub async fn run(&mut self) {
        while let Ok(msg) = self.connection.receive_message().await {
            match msg {
                Message::InvocationRequest { session_id, method, params } => {
                    self.handle_request(session_id, method, params).await;
                }
                Message::InvocationResponse { session_id, result } => {
                    self.handle_response(session_id, result).await;
                }
                Message::ServerPush { session_id, event, data } => {
                    self.handle_server_push(session_id, event, data).await;
                }
                Message::ServerPushAck { session_id } => {
                    self.handle_push_ack(session_id).await;
                }
            }
        }
    }
    
    async fn handle_request(&self, session_id: Uuid, method: String, params: Vec<u8>) {
        // Process request and send response
        let result = process_method(method, params).await;
        self.connection.send_message(Message::InvocationResponse {
            session_id,
            result,
        }).await.unwrap();
    }
    
    async fn handle_response(&self, session_id: Uuid, result: Result<Vec<u8>, String>) {
        if let Some(session) = self.session_manager.get_session(session_id).await {
            session.response_tx.send(result.unwrap()).await.unwrap();
        }
    }
    
    // Similar handlers for ServerPush and ServerPushAck
}