use viz;

use viz_core::Result;
use viz_utils::pretty_env_logger;

#[tokio::main]
async fn main() -> Result {
    pretty_env_logger::init();

    viz::new().listen("127.0.0.1:8000").await
}
