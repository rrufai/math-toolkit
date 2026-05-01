mod app;
mod controllers;

#[tokio::main]
async fn main() {
    loco_rs::cli::main::<app::App>().await.unwrap();
}
