use actix_web::{middleware::Logger, web::Data, App, HttpServer};
use boinc_accoung_manager_rs::{boinc_api, AppState, DataBase};
use clap::Parser;
use std::fs::File;
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    config: PathBuf,
    database: PathBuf,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    let args = Args::parse();

    let database = DataBase::new(&args.database).unwrap();

    let state = AppState::new(&mut File::open(&args.config).unwrap(), database).unwrap();

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(Data::new(state.clone()))
            .service(boinc_api::get_project_config)
            .service(boinc_api::rpc_endpoint)
            .service(boinc_api::proxy_root_route)
            .service(boinc_api::proxy_scheduler_route)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
