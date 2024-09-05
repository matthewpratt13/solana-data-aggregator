// RESTful API to expose the aggregated data

// Responsibilities:
// * Provide a RESTful API to expose aggregated data.
// * Support queries for transaction history, account details, etc.

// Implementation:
// * Use `actix-web` to create a RESTful API server.

use crate::{data_processing::TransactionData, data_storage::get_all_transactions};

use actix_web::{web, App, HttpServer, Responder};
use sqlx::PgPool;

use std::sync::Arc;

/// Handler to get all transactions.
async fn get_transactions(db: web::Data<Arc<PgPool>>) -> impl Responder {
    match get_all_transactions(&db).await {
        Ok(transactions) => web::Json(transactions),
        Err(_) => web::Json(Vec::<TransactionData>::new()),
    }
}

#[actix_web::main]
pub async fn main(db: Arc<PgPool>) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .route("/transactions", web::get().to(get_transactions))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
