use actix::{Actor, Addr};
use actix_files::{Files, NamedFile};
use actix_web::middleware::Logger;
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use uuid::Uuid;

mod client;
mod msg;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    // start timer server actor
    let server = server::Server::new().start();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(server.clone()))
            .service(web::resource("/").to(index))
            .route("/ws/{id}", web::get().to(timer_route))
            .service(Files::new("/static", "./static"))
            .wrap(Logger::default())
    })
    .workers(2)
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}

async fn index() -> impl Responder {
    NamedFile::open_async("./static/index.html").await.unwrap()
}

async fn timer_route(
    req: HttpRequest,
    stream: web::Payload,
    path: web::Path<u64>,
    srv: web::Data<Addr<server::Server>>,
) -> Result<HttpResponse, Error> {
    let ip = match req.peer_addr() {
        Some(addr) => addr.ip().to_string(),
        None => String::from("-.-.-.-"),
    }
    .to_string();
    let cid = path.into_inner();

    let sid = Uuid::new_v4().to_string();

    ws::start(
        client::Client {
            sid,
            cid,
            ip,
            server: srv.get_ref().clone(),
        },
        &req,
        stream,
    )
}
