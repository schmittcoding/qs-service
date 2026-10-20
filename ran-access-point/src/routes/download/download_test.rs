#[cfg(test)]
mod tests {
    use actix_web::{test, web, App, HttpResponse, Responder};
    use serde::{Deserialize, Serialize};
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::Mutex;

    // Dummy DownloadItem type.
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct DownloadItem {
        pub id: i32,
        pub url: String,
    }

    // Dummy DownloadQuery type.
    #[derive(Deserialize)]
    pub struct DownloadQuery {
        // Add any fields if needed
    }

    // Application state holding the cache.
    pub struct AppState {
        pub download_data: Mutex<HashMap<i32, Vec<DownloadItem>>>,
    }

    // Define a trait for fetching download links.
    #[async_trait::async_trait]
    pub trait WebDownloadTrait {
        async fn get_download_links(&self)
            -> Result<Vec<DownloadItem>, Box<dyn std::error::Error>>;
    }

    // Test implementation of the trait.
    pub struct TestWebDB {
        pub return_error: bool,
        pub data: Vec<DownloadItem>,
    }

    #[async_trait::async_trait]
    impl WebDownloadTrait for TestWebDB {
        async fn get_download_links(
            &self,
        ) -> Result<Vec<DownloadItem>, Box<dyn std::error::Error>> {
            if self.return_error {
                Err("test error".into())
            } else {
                Ok(self.data.clone())
            }
        }
    }

    // DBState wrapping our web database.
    pub struct DBState {
        pub web_db: Box<dyn WebDownloadTrait + Send + Sync>,
    }

    // The endpoint we are testing.

    pub async fn download_links(
        states: web::Data<Arc<AppState>>,
        db: web::Data<Arc<DBState>>,
        _query: web::Query<DownloadQuery>,
    ) -> impl Responder {
        let mut cache = states.download_data.lock().await;

        if let Some(download_links) = cache.get(&0) {
            return HttpResponse::Ok().json(json!({ "links": download_links }));
        } else {
            log::warn!("⚠️ Shop cache is empty. Fetching from database...");
        }

        match db.web_db.get_download_links().await {
            Ok(data) => {
                let download_data: Vec<DownloadItem> = data;
                cache.insert(0, download_data.clone());
                HttpResponse::Ok().json(json!({ "links": download_data }))
            }
            Err(err) => {
                log::error!("❌ Failed to download links from database: {:?}", err);
                HttpResponse::InternalServerError()
                    .json(json!({ "error": "Failed to load shop data" }))
            }
        }
    }

    // Test when cache is pre-populated.
    #[actix_web::test]
    async fn test_download_links_cache_hit() {
        // Pre-populate the cache with a known value.
        let mut initial_cache = HashMap::new();
        let cached_items = vec![DownloadItem {
            id: 1,
            url: "http://example.com".into(),
        }];
        initial_cache.insert(0, cached_items.clone());
        let app_state = Arc::new(AppState {
            download_data: Mutex::new(initial_cache),
        });

        // DBState is not used in a cache hit, but must be provided.
        let db_state = Arc::new(DBState {
            web_db: Box::new(TestWebDB {
                return_error: false,
                data: vec![],
            }),
        });

        // Initialize test app with our route.
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .app_data(web::Data::new(db_state))
                .service(web::resource("/get-download-links").route(web::get().to(download_links))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/get-download-links")
            .to_request();

        // Use call_service to get a ServiceResponse.
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body_bytes = test::read_body(resp).await;
        let json_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json_value, json!({ "links": cached_items }));
    }

    // Test when cache is empty and DB returns data.
    #[actix_web::test]
    async fn test_download_links_cache_miss_db_success() {
        // Start with an empty cache.
        let app_state = Arc::new(AppState {
            download_data: Mutex::new(HashMap::new()),
        });

        let db_data = vec![DownloadItem {
            id: 2,
            url: "http://test.com".into(),
        }];
        let db_state = Arc::new(DBState {
            web_db: Box::new(TestWebDB {
                return_error: false,
                data: db_data.clone(),
            }),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state.clone()))
                .app_data(web::Data::new(db_state.clone()))
                .service(web::resource("/get-download-links").route(web::get().to(download_links))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/get-download-links")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body_bytes = test::read_body(resp).await;
        let json_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json_value, json!({ "links": db_data }));

        let app_statex = app_state.clone();
        // Verify that the cache was updated.
        let cache = app_statex.download_data.lock().await;
        assert_eq!(cache.get(&0).unwrap(), &db_data);
    }

    // Test when DB returns an error.
    #[actix_web::test]
    async fn test_download_links_db_error() {
        let app_state = Arc::new(AppState {
            download_data: Mutex::new(HashMap::new()),
        });

        let db_state = Arc::new(DBState {
            web_db: Box::new(TestWebDB {
                return_error: true,
                data: vec![],
            }),
        });

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(app_state))
                .app_data(web::Data::new(db_state))
                .service(web::resource("/get-download-links").route(web::get().to(download_links))),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/get-download-links")
            .to_request();

        let resp = test::call_service(&app, req).await;
        // Expect an Internal Server Error.
        assert_eq!(resp.status(), 500);

        let body_bytes = test::read_body(resp).await;
        let json_value: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();
        assert_eq!(json_value, json!({ "error": "Failed to load shop data" }));
    }
}
