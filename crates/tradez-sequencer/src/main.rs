mod server;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    server::launch_server()?;
    Ok(())
}