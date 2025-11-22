// Usage: https://docs.rs/log/0.4.14/log/
// todo: Use this module to wrap more info collected from upside
use tracing::{info, debug, Level};

use std::{env};
use std::io::stdout;
use time::macros::format_description;
use time::UtcOffset;
use tracing_subscriber::fmt::time::OffsetTime;
use tracing_subscriber::fmt::writer::MakeWriterExt;

pub fn info(msg: &str) {
    match env::var("print_debug") {
        Ok(_) => {
            println!("---info--- {}", msg.to_string())
        }
        Err(_) => {
            info!("---info--- {}", msg.to_string())
        }
    }
}

pub fn warn(msg: &str) {
    match env::var("print_debug") {
        Ok(_) => {
            println!("---warn--- {}", msg)
        }
        Err(_) => {
            debug!("---warn--- {}", msg.to_string())
        }
    }
}


pub fn debug(msg: &str) {
    match env::var("print_debug") {
        Ok(_) => {
            println!("---debug--- {}", msg)
        }
        Err(_) => {
            debug!("---debug--- {}", msg.to_string())
        }
    }
}

pub fn logger_init() -> () {
    let conf_logger_level = env::var("LOGGER_LEVEL").expect("no logger level setting!");
    // log4rs::init_file("config/log4rs.yaml", Default::default()).unwrap()

    let mut logger_level = Level::DEBUG;

    if conf_logger_level.eq("INFO") {
        logger_level = Level::INFO;
    } else if conf_logger_level.eq("WARN") {
        logger_level = Level::WARN;
    } else if conf_logger_level.eq("ERROR") {
        logger_level = Level::ERROR;
    } else if conf_logger_level.eq("TRACE") {
        logger_level = Level::TRACE;
    }


    // let timer = LocalTime::new(time::format_description::well_known::Rfc3339);
    // let timer = UtcTime::new(time::format_description::well_known::Rfc3339);
    // https://rustcc.cn/article?id=66e2a76e-8c65-42f7-a773-66dff1a2a21e
    let local_time = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).unwrap_or_else(|_| UtcOffset::from_hms(0, 0, 0).expect("Failed to create UTC offset")),
        format_description!("[year]-[month]-[day] [hour]:[minute]:[second].[subsecond digits:3]"),
    );


    // https://github.com/tokio-rs/tracing/blob/master/examples/examples/appender-multifile.rs
    // 不要使用non_blocking的appender，否则会异步输出日志。
    let _file_appender = tracing_appender::rolling::daily("/tmp", "huiwing.log").with_max_level(tracing::Level::TRACE);


    let _ = tracing_subscriber::fmt()
        .with_timer(local_time)
        .with_ansi(false)
        .with_max_level(logger_level)
        .with_file(true) // display source file name
        .with_thread_names(true)
        .with_line_number(true)
        .with_target(false)
        // .with_writer(file_appender)
        .with_writer(stdout)
        .init();

}
