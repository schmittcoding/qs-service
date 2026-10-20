use actix_web::web;

pub mod utils;

pub mod admin;

pub mod auth;
pub mod checks;
mod download;
mod item_shop;
mod mileage_shop;
pub mod news;
mod payments;
mod rankings;
mod transactions;
pub mod user;

mod analytics;
mod characters;
pub mod events;

// Add the metrics module
pub mod metrics;

pub fn core_route(cfg: &mut web::ServiceConfig) {
    let api_version = std::env::var("API_VERSION")
        .expect("API_VERSION must be set")
        .parse::<u16>()
        .expect("Invalid API_VERSION");


    cfg
        .service(
        web::scope(&format!("/api/v{}", api_version))
            .service(web::scope("/auth").configure(auth::configure))
            .service(web::scope("/user").configure(user::configure))
            .service(web::scope("/payments").configure(payments::configure))
            .service(web::scope("/news").configure(news::configure))
            .service(web::scope("/item-shop").configure(item_shop::configure))
            .service(web::scope("/mileage-shop").configure(mileage_shop::configure))
            .service(web::scope("/downloads").configure(download::configure))
            .service(web::scope("/rankings").configure(rankings::configure))
            .service(web::scope("/analytics").configure(analytics::configure))
            .service(web::scope("/transactions").configure(transactions::configure))
            .service(web::scope("/characters").configure(characters::configure))
            .service(web::scope("/events").configure(events::configure)),
        )
        // Add metrics endpoints
        .service(metrics::get_metrics);
}

pub fn admin_route(cfg: &mut web::ServiceConfig) {
    let api_version = std::env::var("API_VERSION")
        .expect("API_VERSION must be set")
        .parse::<u16>()
        .expect("Invalid API_VERSION");

    cfg
        .service(
    web::scope(&format!("/v{}", api_version))
                .configure(admin::configure)
                .service(web::scope("/checks").configure(checks::configure))
        )
        // Add metrics endpoints
        .service(metrics::get_metrics);
}
