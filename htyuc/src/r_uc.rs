use htycommons::db::get_uc_db_url;
use htycommons::web::{get_uc_port, launch_rocket};
use dotenv::dotenv;
use crate::uc_rocket;

#[tokio::main]
pub async fn main() {
    dotenv().ok();

    let rocket = launch_rocket(get_uc_port(), uc_rocket(&get_uc_db_url()));
    rocket.await;
}
