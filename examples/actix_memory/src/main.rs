#![allow(missing_docs)]

use std::io;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use multigear::{MemoryStorage, Multer};

async fn upload(
    data: web::Data<Multer<MemoryStorage>>,
    request: HttpRequest,
    payload: web::Payload,
) -> impl Responder {
    let mut multipart = match data.parse(request, payload).await {
        Ok(value) => value,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    };

    let mut stored = Vec::new();

    while let Some(part) = match multipart.next_part().await {
        Ok(value) => value,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    } {
        if part.file_name().is_some() {
            match data.store(part).await {
                Ok(file) => stored.push(file),
                Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
            };
        }
    }

    let total_files = data.storage().len().await;
    let mut body = format!(
        "stored {} file(s) in this request; memory storage now has {} item(s)\n",
        stored.len(),
        total_files
    );
    for file in stored {
        let original_name = file.file_name.as_deref().unwrap_or("<none>");
        body.push_str(&format!(
            "- field={} original={} bytes={} key={}\n",
            file.field_name, original_name, file.size, file.storage_key
        ));
    }

    HttpResponse::Ok().body(body)
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(INDEX_HTML)
}

const INDEX_HTML: &str = r#"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>multigear actix memory upload</title>
</head>
<body>
  <h1>Memory Upload Example</h1>
  <p>Field name: <code>avatar</code> (single file expected)</p>
  <form action="/upload/avatar" method="post" enctype="multipart/form-data">
    <input type="file" name="avatar" />
    <button type="submit">Upload</button>
  </form>
</body>
</html>
"#;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let multer = Multer::builder()
        .single("avatar")
        .storage(MemoryStorage::new())
        .build()
        .expect("multer should build");

    let multer = web::Data::new(multer);
    let bind = ("127.0.0.1", 8081);
    println!(
        "actix-memory-example running at http://{}:{}",
        bind.0, bind.1
    );

    HttpServer::new(move || {
        App::new()
            .app_data(multer.clone())
            .route("/", web::get().to(index))
            .route("/upload/avatar", web::post().to(upload))
    })
    .bind(bind)?
    .run()
    .await
}
