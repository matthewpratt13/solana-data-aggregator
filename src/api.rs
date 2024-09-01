// RESTful API to expose the aggregated data

// Responsibilities:
// * Provide a RESTful API to expose aggregated data.
// * Support queries for transaction history, account details, etc.

// Implementation:
// * Use `actix-web` to create a RESTful API server.

use crate::data_processing::TransactionData;
use crate::data_storage::Database;

use actix_web::{web, App, HttpServer, Responder};

/// Handler to get all transactions.
async fn get_transactions(db: web::Data<Database>) -> impl Responder {
    match db.get_all_transactions().await {
        Ok(transactions) => web::Json(transactions),
        Err(_) => web::Json(Vec::<TransactionData>::new()),
    }
}

#[actix_web::main]
pub async fn main(db: Database) -> std::io::Result<()> {
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(db.clone()))
            .route("/transactions", web::get().to(get_transactions))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
