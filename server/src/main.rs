use axum::body::{boxed, Body};
use axum::http::{Response, StatusCode};
use axum::{response::IntoResponse, routing::get, Router};
use clap::Parser;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::path::PathBuf;
use std::str::FromStr;
use tokio::fs;
use tower::{ServiceBuilder, ServiceExt};
use tower_http::services::ServeDir;
use tower_http::trace::TraceLayer;

#[derive(Parser, Debug)]
#[clap(name = "server")]
struct Opt {
    /// Log level
    #[clap(short, long, default_value = "info")]
    log_level: String,

    /// Address to serve the app from
    #[clap(short, long, default_value = "127.0.0.1")]
    addr: String,

    /// Port to serve the app from
    #[clap(short, long, default_value = "8080")]
    port: u16,

    /// Directory where static content is located
    #[clap(long, default_value = "../dist")]
    static_dir: String,
}

#[tokio::main]
async fn main() {
    let opt = Opt::parse();

    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", format!("{},hyper=info,mio=info", opt.log_level))
    }
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/api/hello", get(hello))
        .fallback(get(|req| async move {
            match ServeDir::new(&opt.static_dir).oneshot(req).await {
                Ok(res) => match res.status() {
                    StatusCode::NOT_FOUND => {
                        let index_path = PathBuf::from(&opt.static_dir).join("index.html");

                        let index_content = match fs::read_to_string(index_path).await {
                            Ok(index_content) => index_content,
                            Err(_) => {
                                return Response::builder()
                                    .status(StatusCode::NOT_FOUND)
                                    .body(boxed(Body::from("index file not found")))
                                    .unwrap()
                            }
                        };
                        Response::builder()
                            .status(StatusCode::OK)
                            .body(boxed(Body::from(index_content)))
                            .unwrap()
                    }
                    _ => res.map(boxed),
                },
                Err(err) => Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(boxed(Body::from(format!("error: {err}"))))
                    .expect("error response"),
            }
        }))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http()));

    let sock_addr = SocketAddr::from((
        IpAddr::from_str(opt.addr.as_str()).unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST)),
        opt.port,
    ));

    log::info!("listening on http://{}", sock_addr);

    axum::Server::bind(&sock_addr)
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}

async fn hello() -> impl IntoResponse {
    "It works"
}
