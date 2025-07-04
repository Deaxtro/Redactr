use actix_web::{web, App, HttpResponse, HttpServer, Responder};
// use psutil::memory;
use redactr::load_rule_configs;
use regex::Regex;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;
// use serde::Serialize;
// use std::{str::FromStr, time::{SystemTime, UNIX_EPOCH}};


// #[derive(Serialize)]
// struct HealthCheck {
//     name: String,
//     status: String,
// }

// #[derive(Serialize)]
// struct HealthStatus {
//     uptime: u64,
//     memory_usage: f32,
//     disk_usage: u64,
//     checks: Vec<HealthCheck>,
// }

// There is an issue with the psutil dependency which causes a conflict between it requiring "memchr" 
// at a different version and another one of the dependencies requiring a different version, can't remember exactly which.
// Will put back the health api when a suitable replacement for psutils is found.
// Leaving it here if someone else wants to try it out.
// Currently using Windows 11 at AMD architecture

// async fn health() -> impl Responder {
//     let mut checks = vec![];
//     let uptime = SystemTime::now()
//         .duration_since(UNIX_EPOCH)?
//         .as_secs();
//     checks.push(HealthCheck {
//         name: "Container uptime".to_string(),
//         status: format!("{} seconds", uptime),
//     });

//     let memory_usage = memory::vritual_memory()?;
//     let memory = memory_usage.percent();
//     checks.push(HealthCheck {
//         name: "Memory usage".to_string(),
//         status: format!("{} %", memory.to_string()),
//     });

//     let disk_usage = psutil::disk::disk_usage("/")?;
//     checks.push(HealthCheck {
//         name: "Disk usage".to_string(),
//         status: format!("{} bytes", disk_usage.total()),
//     });

//     let health_status = HealthStatus {
//         uptime,
//         memory_usage: memory_usage.percent(),
//         disk_usage: disk_usage.total(),
//         checks,
//     };
//     HttpResponse::Ok().json(health_status)
// }

async fn index() -> impl Responder {
    let html = r#"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Redactr</title>
    </head>
    <body>
        <h1>Redactr</h1>
        <p>Redactr is a service that removes personal identifiable information from text.</p>
        <p>It is a HTTP API that accepts a JSON payload with a text field and returns a JSON payload with redacted_text field.</p>
        <p>Endpoints available:</p>
        <ul>
            <li>POST <a href="/redact">/redact</a></li>
            <li>GET <a href="/health">/health</a></li>
        </ul>
    </body>
    </html>"#;
    HttpResponse::Ok().body(html)
}

async fn redact(input_text: web::Json<String>) -> impl Responder {
    let mut rules = load_rule_configs();

    let mut redacted_text = input_text.to_string();
    for rule in &mut rules {
        let regex = Regex::new(rule.pattern.as_str()).unwrap();
        for captures in regex.captures_iter(&input_text) {
            let matched_text = captures.get(0).unwrap().as_str();
            let redacted_match = rule.on_match(matched_text);
            redacted_text = redacted_text.replace(matched_text, &redacted_match);
        }
    }

    HttpResponse::Ok().body(redacted_text)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {

    let logging_level = std::env::var("REDACTR_LOGGING_LEVEL").unwrap_or("info".to_string());
    let max_level = match logging_level.as_str() {
        "trace" => Level::TRACE,
        "debug" => Level::DEBUG,
        "Warn" => Level::WARN,
        "error" => Level::ERROR,
        _ => Level::INFO,
    };

    let address = "127.0.0.1";
    let port = "8080";
    let bind_address = format!("{}:{}", address, port);
    
    let subscriber = FmtSubscriber::builder()
        .with_max_level(max_level)
        .finish();

    tracing::subscriber::set_global_default(subscriber).expect("Failed to set global default");

    info!("Starting redactr service...");
    info!(address=%address, port=%port, "Listening at");

    HttpServer::new(|| {
        App::new()
            .service(web::resource("/").route(web::get().to(index)))
            .service(web::resource("/redact").route(web::post().to(redact)))
            // .service(web::resource("/health").route(web::get().to(health)))
    })
    .bind(bind_address)?
    .run()
    .await
}



#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{http::header::ContentType, test, web, App};
    use pretty_assertions::assert_eq;

    #[actix_web::test]
    async fn test_index_get() {
        let app = test::init_service(App::new().route("/", web::get().to(index))).await;
        let req = test::TestRequest::default()
            .insert_header(ContentType::plaintext())
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
    }

    #[actix_web::test]
    async fn test_redact_post() {
        let app = test::init_service(App::new().route("/redact", web::post().to(redact))).await;
        let req = test::TestRequest::post()
            .uri("/redact")
            .set_json(&serde_json::json!(
                "Alfred Smith and John Doe went to the supermarket."
            ))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_success());
        let body_bytes = test::read_body(resp).await;
        let body_str = std::str::from_utf8(&body_bytes).unwrap();
        assert_eq!(body_str, r#"Person1 and Person2 went to the supermarket."#);
    }

    #[actix_web::test]
    async fn test_redact_post_invalid() {
        let app = test::init_service(App::new().route("/redact", web::post().to(redact))).await;
        let req = test::TestRequest::post()
            .uri("/redact")
            .set_payload("data=Alfred Smith and John Doe went to the supermarket.")
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_client_error());
    }
}