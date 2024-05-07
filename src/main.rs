mod endpoints;
mod tool {
    pub mod api_result;
}

use std::env;

use log::LevelFilter;
use log4rs::{
    config::{Appender, Root},
    encode::pattern::PatternEncoder,
    Config,
};

#[macro_use]
extern crate rocket;

#[rocket::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let log_file_path = env::var("LOG_FILE").expect("did not find LOG_FILE env variable");
    let port = env::var("PORT")
        .expect("did not find PORT env variable")
        .parse::<i32>()
        .expect("why you give me PORT that not a number? are you ok?");

    let requests = log4rs::append::file::FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{h({d(%Y-%m-%d %H:%M:%S)(utc)} - {l}: {m}{n})}",
        )))
        .build(log_file_path)
        .unwrap();

    let config = Config::builder()
        .appender(Appender::builder().build("out", Box::new(requests)))
        .build(Root::builder().appender("out").build(LevelFilter::Debug))
        .unwrap();

    log4rs::init_config(config).unwrap();
    log::info!("Start program");

    let figment = rocket::Config::figment().merge(("port", port));

    let _rocket = rocket::custom(figment)
        .mount("/api", endpoints::endpoints())
        .launch()
        .await?;

    Ok(())
}
