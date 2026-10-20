use actix_session::Session;
use actix_web::HttpResponse;
use ran_models::users::ProfileData;

mod session_test;

pub async fn get_user_session(session: Session) -> Result<ProfileData, HttpResponse> {
    match session.get::<ProfileData>("user_data") {
        Ok(Some(user_profile)) => Ok(user_profile),
        Ok(None) => {
            println!("No session data found");
            Err(HttpResponse::Unauthorized().body("No session data found"))
        }
        Err(e) => {
            eprintln!("Failed to retrieve session data: {}", e);
            Err(HttpResponse::InternalServerError().body("Failed to retrieve session data"))
        }
    }
}
