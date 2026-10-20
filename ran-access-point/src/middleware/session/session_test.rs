use super::get_user_session;

#[cfg(test)]
use actix_session::{storage::CookieSessionStore, SessionMiddleware};
use actix_web::HttpResponse;
use ran_models::users::ProfileData;

// A helper handler that simply calls get_user_session.
async fn _get_session_handler(session: actix_session::Session) -> HttpResponse {
    match get_user_session(session).await {
        Ok(profile) => HttpResponse::Ok().json(profile),
        Err(resp) => resp,
    }
}

async fn _set_session_handler(session: actix_session::Session) -> HttpResponse {
    let test_profile = ProfileData {
        user_name: "mahadtest".to_string(),
        user_num: 888,
        user_email: "mahadtest".to_string(),
        user_available: 1,
        user_type: 1,
        web_points: 10.00,
        mileage_points: 0.0,
        created_at: chrono::Local::now(),
        last_updated: chrono::Local::now(),
        is_verified: false,
    };
    // Insert data into the session.
    session.insert("user_data", &test_profile).unwrap();
    HttpResponse::Ok().finish()
}

#[actix_web::test]
async fn test_get_user_session_found() {
    // Build our test app with session middleware.
    let key = actix_web::cookie::Key::generate();
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                key.clone(),
            ))
            .route("/set", actix_web::web::get().to(_set_session_handler))
            .route("/get", actix_web::web::get().to(_get_session_handler)),
    )
    .await;

    let set_req = actix_web::test::TestRequest::get().uri("/set").to_request();
    let set_resp = actix_web::test::call_service(&app, set_req).await;
    assert_eq!(set_resp.status(), 200);

    let cookies: Vec<_> = set_resp.response().cookies().map(|c| c.clone()).collect();
    assert!(!cookies.is_empty(), "No cookies found in response");

    let get_req = actix_web::test::TestRequest::get()
        .uri("/get")
        .cookie(cookies[0].clone())
        .to_request();
    let get_resp = actix_web::test::call_service(&app, get_req).await;
    assert_eq!(get_resp.status(), 200);

    let body_bytes = actix_web::test::read_body(get_resp).await;
    let returned_profile: ProfileData = serde_json::from_slice(&body_bytes).unwrap();
    let test_profile = ProfileData {
        user_name: "mahadtest".to_string(),
        user_num: 888,
        user_email: "mahadtest".to_string(),
        user_available: 1,
        user_type: 1,
        web_points: 10.00,
        mileage_points: 0.0,
        created_at: chrono::Local::now(),
        last_updated: chrono::Local::now(),
        is_verified: false,
    };
    assert_eq!(
        (returned_profile.user_name, returned_profile.user_num),
        (test_profile.user_name, test_profile.user_num)
    );
}

#[actix_web::test]
async fn test_get_user_session_not_found() {
    let key = actix_web::cookie::Key::generate();
    let app = actix_web::test::init_service(
        actix_web::App::new()
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                key.clone(),
            ))
            .route("/get", actix_web::web::get().to(_get_session_handler)),
    )
    .await;

    let req = actix_web::test::TestRequest::get().uri("/get").to_request();
    let resp = actix_web::test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
    let body_bytes = actix_web::test::read_body(resp).await;
    let body_str = std::str::from_utf8(&body_bytes).unwrap();
    assert_eq!(body_str, "No session data found");
}
