use crate::webserver::routes::ws_route;
use crate::webserver::static_files;
use actix_cors::Cors;
pub use actix_web::dev::Server;
use actix_web::dev::ServerHandle;
use actix_web::middleware::Logger;
use actix_web::{web, App, HttpServer};
use std::net::SocketAddr;

const STATIC_PATH_SEGMENT: &str = "static";

pub fn webserver(addr: SocketAddr) -> Result<(HttpServerStateManager, Server), String> {
    let http_server: Server = match HttpServer::new(move || {
        let file_map = web::Data::new(static_files::cache_static_files());

        let cors = Cors::default()
            .allow_any_origin()
            .allowed_methods(vec!["GET"])
            // .allowed_headers(vec![header::CONTENT_TYPE, header::ACCEPT])
            .max_age(3600);

        App::new()
            .app_data(file_map)
            .wrap(cors)
            .wrap(Logger::default())
            .service(
                web::scope(format!("/{}", STATIC_PATH_SEGMENT).as_str())
                    .route("/{path:.*}", web::get().to(static_files::static_handler)),
            )
            .service(web::scope("/ws").route("/{path:.*}", web::get().to(ws_route)))
            .service(web::redirect("/", format!("/{}/", STATIC_PATH_SEGMENT)))
    })
    .bind(addr)
    {
        Ok(bound) => bound,
        Err(_) => {
            return Err("Server could not be started, probably port blocked".into());
        }
    }
    .disable_signals()
    .run();
    info!("Webserver started successfully");

    let manager = HttpServerStateManager {
        handle: http_server.handle(),
    };

    return Ok((manager, http_server));
}

pub struct HttpServerStateManager {
    handle: ServerHandle,
}
impl HttpServerStateManager {
    pub async fn stop_gracefully(&self) {
        let handle = self.handle.clone();
        handle.stop(true).await;
        info!("Server shut down gracefully");
    }
}
