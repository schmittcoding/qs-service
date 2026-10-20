use std::sync::Arc;

use actix_web::{post, web, HttpResponse, Responder};
use ran_db_core::webmanager::news::WebNewsTrait;
use ran_models::news::{CreateNewsRequest, NewsItem};

use crate::database::DBState;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(create_news);
}

#[post("/create-news")]
async fn create_news(
    db: web::Data<Arc<DBState>>,
    state: web::Data<Arc<crate::states::AppState>>,
    req: web::Json<CreateNewsRequest>,
) -> impl Responder {
    tracing::info!("Creating news article");
    let news_data = req.into_inner();

    match db.web_db.create_news_data(news_data).await {
        Ok(_) => {
            tokio::spawn(async move {
                let mut cache = state.news_data.lock().await;
                cache.clear();
                let news_data = db.web_db.get_news_data().await.unwrap();
                let news_items: Vec<NewsItem> =
                    news_data.into_iter().map(|data| data.into()).collect();
                cache.insert(0, news_items);
            });

            HttpResponse::Ok().json("News article created successfully")
        }
        Err(e) => HttpResponse::InternalServerError()
            .json(format!("Failed to create news article: {:?}", e)),
    }
}
