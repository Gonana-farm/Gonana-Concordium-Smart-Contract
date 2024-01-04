mod handlers;

use actix_web::{HttpServer, App};
use handlers::api::{list_product,get_listings};
use log::LevelFilter;




#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();


    log::info!("server initialized and running at port 8080");
    log::info!("http://127.0.0.1:8080");
    HttpServer::new(move || {        
        App::new()
            .service(list_product)
            .service(get_listings)
        })
        .bind("127.0.0.1:8080")?
        .run()
        .await
}
