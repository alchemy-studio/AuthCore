// use htycommons::logger::logger_init;
use htycommons::web::{get_uc_port, launch_rocket};
use htyuc::uc_rocket;
use dotenv::dotenv;
use htycommons::db::get_uc_db_url;
use htycommons::logger::logger_init;

#[tokio::main]
async fn main() {
    dotenv().ok();

    // let log_file = rolling::hourly("./logs", "htyuc");
    // let (non_blocking, _guard) = tracing_appender::non_blocking(log_file);
    //
    // tracing_subscriber::fmt()
    //     .with_writer(non_blocking)
    //     .with_ansi(false)
    //     .with_max_level(tracing::Level::DEBUG)
    //     .with_file(true)
    //     .with_thread_names(true)
    //     .with_line_number(true)
    //     .with_target(false)
    //     .init();

    logger_init();

    launch_rocket(get_uc_port(), uc_rocket(&get_uc_db_url())).await;

    // this is reachable only after `Shutdown::notify()` or `Ctrl+C`.
    println!("Rocket: deorbit.");
}
