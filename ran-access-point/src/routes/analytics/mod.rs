use actix_web::{web:: { scope }};

mod users;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service( scope("/users").configure(users::configure));
}
