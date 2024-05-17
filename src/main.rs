use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use serde::{Deserialize, Serialize};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use uuid::Uuid;
use dotenv::dotenv;
use std::env;

// Define a struct to represent the data
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Item {
    id: Uuid,
    name: String,
    description: String,
}

// Create a new item
async fn create_item(pool: web::Data<PgPool>, item: web::Json<ItemCreateRequest>) -> impl Responder {
    let id = Uuid::new_v4();
    let result = sqlx::query!(
        "INSERT INTO items (id, name, description) VALUES ($1, $2, $3)",
        id,
        item.name,
        item.description
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Created().json(Item { id, name: item.name.clone(), description: item.description.clone() }),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

// Get all items
async fn get_items(pool: web::Data<PgPool>) -> impl Responder {
    let result = sqlx::query_as!(Item, "SELECT id, name, description FROM items")
        .fetch_all(pool.get_ref())
        .await;

    match result {
        Ok(items) => HttpResponse::Ok().json(items),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

// Get a specific item by ID
async fn get_item(pool: web::Data<PgPool>, item_id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query_as!(Item, "SELECT id, name, description FROM items WHERE id = $1", *item_id)
        .fetch_one(pool.get_ref())
        .await;

    match result {
        Ok(item) => HttpResponse::Ok().json(item),
        Err(_) => HttpResponse::NotFound().body("Item not found"),
    }
}

// Update an item by ID
async fn update_item(pool: web::Data<PgPool>, item_id: web::Path<Uuid>, item: web::Json<ItemUpdateRequest>) -> impl Responder {
    let result = sqlx::query!(
        "UPDATE items SET name = $1, description = $2 WHERE id = $3",
        item.name,
        item.description,
        *item_id
    )
    .execute(pool.get_ref())
    .await;

    match result {
        Ok(_) => HttpResponse::Ok().json(Item { id: *item_id, name: item.name.clone(), description: item.description.clone() }),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

// Delete an item by ID
async fn delete_item(pool: web::Data<PgPool>, item_id: web::Path<Uuid>) -> impl Responder {
    let result = sqlx::query!("DELETE FROM items WHERE id = $1", *item_id)
        .execute(pool.get_ref())
        .await;

    match result {
        Ok(_) => HttpResponse::Ok().body("Item deleted"),
        Err(_) => HttpResponse::InternalServerError().into(),
    }
}

// Request structs
#[derive(Debug, Deserialize)]
struct ItemCreateRequest {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize)]
struct ItemUpdateRequest {
    name: String,
    description: String,
}

// Main function to configure and run the server
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to create pool");

    // Start HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/items", web::post().to(create_item))
            .route("/items", web::get().to(get_items))
            .route("/items/{id}", web::get().to(get_item))
            .route("/items/{id}", web::put().to(update_item))
            .route("/items/{id}", web::delete().to(delete_item))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}