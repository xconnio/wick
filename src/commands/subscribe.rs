use xconn::async_::session::Session;

pub async fn handle(_session: &Session) -> Result<(), Box<dyn std::error::Error>> {
    println!("Subcommand 'subscribe' executed");
    Ok(())
}
