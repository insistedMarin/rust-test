use axum::{Router,routing::get,response::Html};
use std::{net::SocketAddr, time::Duration};
use tokio::signal;
use deadpool_redis::{Config,Runtime};
use std::env;
use tokio::sync::oneshot;


#[tokio::main]
async fn main(){
    let (shutdown_tx,shutdown_rx) = oneshot::channel();

    tokio::spawn(async move {
        signal::ctrl_c().await.unwrap();
        let _ = shutdown_tx.send(());
    });

    tokio::spawn(async move {
        let client = reqwest::Client::new();
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if client.get("http://localhost:8080/")
            .send()
            .await
            .is_err()
            {
                println!("❌ Failed to connect to server");
                break;
            }
        }
    });
    // redis
    let redis_pool= init_redis_pool().await;
    if let Err(e) = test_redis(&redis_pool).await {
        eprintln!("❌ Redis connection failed: {}", e);
        return;
    }


    // service
    let app = Router::new()
    .route("/",get(handler))
    .route("/health",get(health_check));

    let addr = SocketAddr::from(([0, 0, 0, 0],8080));
    println!("Server listening on {}",addr);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener,app)
    .with_graceful_shutdown(async move{
        let _ = shutdown_rx.await;
        
        perform_redis_cleanup(&redis_pool).await;
        println!("⚠️ Receive shutdown signal...");
    })
    .await
    .unwrap();
}

async fn handler() -> Html<&'static str> {
    Html("<h1>Hello, Axum!</h1>")
}

async fn health_check() -> impl axum::response::IntoResponse {
    axum::http::StatusCode::OK
}

// async fn shutdown_signal(pool: deadpool_redis::Pool){
//     let ctrl_c = async{
//         signal::ctrl_c()
//         .await
//         .expect("failed to install Ctrl+C handler");
//     };
//     #[cfg(unix)]
//     let terminate = async{
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//         .expect("failed to install signal handler")
//         .recv()
//         .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select!{
//         _=ctrl_c=>{
//             println!("cmd signal received, shutting down...");
//         }
//         _=terminate=>{
//             println!("unix signal received, shutting down...");
//         }
//     }
//     println!("Shutting down gracefully...");
//     perform_redis_cleanup(&pool).await;
//     println!("Server shutdown complete");

// }

async fn init_redis_pool() -> deadpool_redis::Pool{
    let redis_url = env::var("REDIS_URL")
    .unwrap_or("redis://127.0.0.1".to_string()); 
    let cfg = Config::from_url(&redis_url);
    cfg.create_pool(Some(Runtime::Tokio1))
    .expect("Failed to create Redis pool")
}

async fn test_redis(pool: &deadpool_redis::Pool) -> Result<(), String> {
    let mut conn = pool.get().await.map_err(|e| e.to_string())?;
    let pong: String = redis::cmd("PING")
    .query_async(&mut conn)
    .await
    .map_err(|e| e.to_string())?;

if pong != "PONG" {
    return Err("Unexpected response".into());
}

println!("✅ Redis connected successfully");
Ok(())
}

async fn perform_redis_cleanup(pool: &deadpool_redis::Pool) {
    let mut conn = match pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            eprintln!("⚠️ Failed to get Redis connection for cleanup: {}", e);
            return;
        }
    };
    if let Err(e) = redis::cmd("SET")
        .arg("rust-test")
        .arg("graceful-shutdown")
        .query_async::<_, String>(&mut conn)
        .await
    {
        eprintln!("⚠️ Failed to clean Redis keys: {}", e);
    } else {
        println!("🗑️ Cleaned Redis temporary keys");
    }
}


