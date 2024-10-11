use anyhow::Result;
use clap::Parser;
use lighthouse_client::{protocol::Authentication, Lighthouse, LIGHTHOUSE_URL};
use tracing::info;
use std::path::Path;

mod player;

#[derive(Parser)]
#[command(version)]
struct Args {
    /// The username.
    #[arg(short, long, env = "LIGHTHOUSE_USER")]
    username: String,
    /// The API token.
    #[arg(short, long, env = "LIGHTHOUSE_TOKEN")]
    token: String,
    /// The server URL.
    #[arg(long, env = "LIGHTHOUSE_URL", default_value = LIGHTHOUSE_URL)]
    url: String,
    /// The path to the video file to play.
    path: String,
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    _ = dotenvy::dotenv();

    let args = Args::parse();
    let auth = Authentication::new(&args.username, &args.token);

    let lh = Lighthouse::connect_with_tokio_to(&args.url, auth).await?;
    info!("Connected to the Lighthouse server");

    player::run(Path::new(&args.path), lh).await
}
