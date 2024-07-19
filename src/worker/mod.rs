pub fn run_as_worker(port: u16) -> anyhow::Result<()> {
    let addr = format!("127.0.0.1:{}", port);
    let stream = std::net::TcpStream::connect(addr).unwrap();

    let mut server = crate::rpc::server::Server::new(stream);
    let router = crate::WorkerRouter::new();
    server.serve(&router)?;

    Ok(())
}
