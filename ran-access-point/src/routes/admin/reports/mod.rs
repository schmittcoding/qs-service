mod players;
mod transactions;

pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg
        .service(actix_web::web::scope("/transactions").configure(transactions::configure));
}
