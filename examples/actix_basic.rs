#![allow(missing_docs)]

#[cfg(feature = "actix")]
use actix_web::{App, HttpRequest, HttpResponse, Responder, web};
#[cfg(feature = "actix")]
use futures::StreamExt;
#[cfg(feature = "actix")]
use rust_multer::{MemoryStorage, Multer};

#[cfg(feature = "actix")]
async fn upload(request: HttpRequest, payload: web::Payload) -> impl Responder {
    let multer = Multer::new(MemoryStorage::new());
    let mut multipart = match rust_multer::actix::multipart_from_request(&multer, &request, payload)
    {
        Ok(value) => value,
        Err(err) => return HttpResponse::BadRequest().body(err.to_string()),
    };

    let mut count = 0usize;
    while let Some(item) = multipart.next().await {
        if let Err(err) = item {
            return HttpResponse::BadRequest().body(err.to_string());
        }
        count += 1;
    }

    HttpResponse::Ok().body(format!("parsed {count} multipart parts"))
}

#[cfg(feature = "actix")]
fn main() {
    let _app = App::new().route("/upload", web::post().to(upload));
}

#[cfg(not(feature = "actix"))]
fn main() {
    println!("Enable the `actix` feature to run this example.");
}
