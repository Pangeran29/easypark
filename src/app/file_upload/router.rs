use std::fs;

use axum::{extract::Multipart, routing::post, Router};
use serde::Serialize;
use uuid::Uuid;

use crate::{error::aggregate::Result, extractor::app_json::AppSuccess};

pub fn build() -> Router {
    let router = Router::new().route("/", post(file_upload));
    // .layer(middleware::from_fn(print_request_body));

    Router::new().nest("/file-upload", router)
}

#[derive(Serialize)]
struct FileMetadata {
    name: String,
    file_name: String,
    content_type: String,
    location: String,
}

async fn file_upload(mut multipart: Multipart) -> Result<AppSuccess<FileMetadata>> {
    let mut name = String::from("-");
    let mut file_name = String::from("-");
    let mut content_type = String::from("-");
    let mut location = String::from("-");

    while let Some(field) = multipart.next_field().await.unwrap() {
        name = field.name().unwrap().to_string();
        file_name = format!(
            "{}-{}",
            Uuid::new_v4(),
            field.file_name().unwrap().to_string()
        );
        content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        location = format!("./public/files/{}", file_name);
        fs::write(location.clone(), data)?;
    }

    Ok(AppSuccess(FileMetadata {
        name,
        file_name,
        content_type,
        location,
    }))
}
