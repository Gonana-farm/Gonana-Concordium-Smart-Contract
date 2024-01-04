mod handlers;

use actix_web::{HttpServer, App};
use handlers::api::{
    list_product,
    get_listings,
    order,
    get_orders
};
use log::LevelFilter;


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::builder().filter_level(LevelFilter::Info).init();


    log::info!("server initialized and running at port 8088");
    HttpServer::new(move || {        
        App::new()
            .service(list_product)
            .service(get_listings)
            .service(order)
            .service(get_orders)
        })
        .bind("0.0.0.0:8088")?
        .run()
        .await
}
