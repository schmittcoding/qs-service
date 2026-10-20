pub mod campaigns;
pub mod suppression;
pub mod templates;
pub mod ws;

/// JWT-protected endpoints (templates + campaign CRUD + suppression management).
/// Registered inside the JwtMiddleware scope in admin/mod.rs.
pub fn configure(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/templates").configure(templates::configure),
    )
    .service(
        actix_web::web::scope("/campaigns").configure(campaigns::configure),
    )
    .service(
        actix_web::web::scope("/suppression").configure(suppression::configure),
    );
}

/// WebSocket endpoint — registered OUTSIDE the JwtMiddleware scope because
/// browsers cannot send custom headers on WebSocket upgrades. Authentication
/// is handled inside the handler via the `?token=<JWT>` query parameter.
pub fn configure_ws(cfg: &mut actix_web::web::ServiceConfig) {
    cfg.service(
        actix_web::web::scope("/campaigns").service(ws::campaign_progress_ws),
    );
}
