use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    // Client -> Server
    InvocationRequest {
        session_id: String,
        method: String,
        params: Vec<u8>, // Serialized parameters
    },
    // Server -> Client
    InvocationResponse {
        session_id: String,
        result: Result<Vec<u8>, String>, // Serialized result or error
    },
    // Server -> Client (reversed role)
    ServerPush {
        session_id: String,
        event: String,
        data: Vec<u8>,
    },
    // Client -> Server (reversed role)
    ServerPushAck {
        session_id: String,
    },
}
