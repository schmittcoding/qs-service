

use std::env;

use actix_web::web::Data;

use crate::services::jwt::{JwtConfig, JwtMiddleware};

mod download;
pub mod email;
pub mod events;
mod items;
mod news;
mod orders;
mod recharge;
mod shop;
mod purchases;
mod item_bank;
mod reports;
pub mod item_shop;
mod mileage;

pub mod auth;
pub mod characters;
pub mod players;
pub mod dashboard;
pub mod profile;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    let jwt_config = JwtConfig {
        secret: env::var("SESSION_SECRET").expect("PLEASE SET SESSION_SECRET"),
        expiration_minutes: 1440,
    };

    let jwt_data = Data::new(jwt_config); // Arc internally

    cfg
        .app_data(jwt_data.clone())
        .service(actix_web::web::scope("/auth").configure(auth::configure))
        // WebSocket endpoints that do their own auth via ?token= query param —
        // must be outside JwtMiddleware because browsers cannot send custom
        // headers on WebSocket upgrade requests.
        .service(actix_web::web::scope("/email-ws").configure(email::configure_ws))
        .service(actix_web::web::scope("")
            .wrap(JwtMiddleware::new(jwt_data.get_ref().clone()))
            .service(actix_web::web::scope("/downloads").configure(download::configure))
            .service(actix_web::web::scope("/item-shop").configure(item_shop::configure))
            .service(actix_web::web::scope("/shop").configure(shop::configure))
            .service(actix_web::web::scope("/news").configure(news::configure))
            .service(actix_web::web::scope("/recharge").configure(recharge::configure))
            .service(actix_web::web::scope("/reports").configure(reports::configure))
            .service(actix_web::web::scope("/players").configure(players::configure))
            .service(actix_web::web::scope("/characters").configure(characters::configure))
            .service(actix_web::web::scope("/items").configure(items::configure))
            .service(actix_web::web::scope("/orders").configure(orders::configure))
            .service(actix_web::web::scope("/purchases").configure(purchases::configure))
            .service(actix_web::web::scope("/item-bank").configure(item_bank::configure))
            .service(actix_web::web::scope("/events").configure(events::configure))
            .service(actix_web::web::scope("/dashboard").configure(dashboard::configure))
            .service(actix_web::web::scope("/profile").configure(profile::configure))
            .service(actix_web::web::scope("/mileage").configure(mileage::configure))
            .service(actix_web::web::scope("/email").configure(email::configure))
        );
}
