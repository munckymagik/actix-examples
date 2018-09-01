extern crate actix;
extern crate actix_web;
extern crate dotenv;
extern crate env_logger;
extern crate futures;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate tera;

use actix::prelude::SyncArbiter;
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::middleware::Logger;
use actix_web::{dev::Resource, fs, http, server, App};
use dotenv::dotenv;
use tera::Tera;

mod api;
mod db;
mod model;
mod schema;
mod session;

static SESSION_SIGNING_KEY: &[u8] = &[0; 32];
const NUM_DB_THREADS: usize = 3;

fn main() {
    dotenv().ok();

    std::env::set_var("RUST_LOG", "actix_todo=debug,actix_web=info");
    env_logger::init();

    // Start the Actix system
    let system = actix::System::new("todo-app");

    let pool = db::init_pool();
    let addr = SyncArbiter::start(NUM_DB_THREADS, move || db::DbExecutor(pool.clone()));

    let app = move || {
        debug!("Compiling templates");
        let tera: Tera = compile_templates!("templates/**/*");

        debug!("Constructing the App");
        App::with_state(api::AppState {
            template: tera,
            db: addr.clone(),
        }).middleware(Logger::default())
            .middleware(SessionStorage::new(
                CookieSessionBackend::signed(SESSION_SIGNING_KEY).secure(false),
            ))
            .route("/", http::Method::GET, api::index)
            .resource("/todo/{id}", |r: &mut Resource<_>| {
                r.post().with(api::update_with_reinterpreted_method)
            })
            .route("/todo", http::Method::POST, api::create)
            .handler(
                "/static",
                fs::StaticFiles::new("static/").expect("new static files failed"),
            )
            .default_resource(|r: &mut Resource<_>| r.f(|_| api::not_found()))
    };

    debug!("Starting server");

    server::new(app).bind("localhost:8088").unwrap().start();

    // Run actix system, this method actually starts all async processes
    let _ = system.run();
}
