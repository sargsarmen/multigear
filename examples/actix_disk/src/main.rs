#![allow(missing_docs)]

use std::io;

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use multigear::{DiskStorage, FilenameStrategy, Multer};

async fn upload(
    data: web::Data<Multer<DiskStorage>>,
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

    let mut body = format!("stored {} file(s)\n", stored.len());
    for file in stored {
        let path = file
            .path
            .as_ref()
            .map(|value| value.display().to_string())
            .unwrap_or_else(|| "<none>".to_owned());
        let original_name = file.file_name.as_deref().unwrap_or("<none>");
        body.push_str(&format!(
            "- field={} original={} bytes={} path={}\n",
            file.field_name, original_name, file.size, path
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
  <title>multigear actix disk upload</title>
</head>
<body>
  <h1>Disk Upload Example</h1>
  <p>Field name: <code>files</code> (multiple allowed)</p>
  <form action="/upload" method="post" enctype="multipart/form-data">
    <input type="file" name="files" multiple />
    <button type="submit">Upload</button>
  </form>
</body>
</html>
"#;

#[actix_web::main]
async fn main() -> io::Result<()> {
    let storage = DiskStorage::builder()
        .destination(std::env::temp_dir().join("multigear-actix-disk"))
        .filename(FilenameStrategy::Random)
        .build()
        .expect("disk storage should build");

    let multer = Multer::builder()
        .array("files", 10)
        .max_file_size(64 * 1024 * 1024)
        .storage(storage)
        .build()
        .expect("multer should build");

    let multer = web::Data::new(multer);
    let bind = ("127.0.0.1", 8080);
    println!("actix-disk-example running at http://{}:{}", bind.0, bind.1);

    HttpServer::new(move || {
        App::new()
            .app_data(multer.clone())
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload))
    })
    .bind(bind)?
    .run()
    .await
}
