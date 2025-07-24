mod communicate;

use communicate::SessionFactory;
use communicate::SocketServer;

fn main() -> std::io::Result<()> {
    let session_factory = SessionFactory::new();
    let socket_server = SocketServer::new(session_factory);
    socket_server.start(39174)?;
    Ok(())
}
