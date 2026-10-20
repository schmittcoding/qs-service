mod entries;
mod catalog;
mod dispatch;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(actix_web::web::scope("/entries").configure(entries::configure))
        .service(actix_web::web::scope("/catalog").configure(catalog::configure))
        .service(actix_web::web::scope("/dispatches").configure(dispatch::configure));
}
