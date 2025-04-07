use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use pdf_extract::extract_text;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Debug, Serialize, Deserialize)]
struct IbanResponse {
    ibans: Vec<IbanMatch>,
    text_preview: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct IbanMatch {
    iban: String,
    confidence: Confidence,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
enum Confidence {
    High,
    Medium,
    Low,
    Fallback,
}

async fn index() -> impl Responder {
    HttpResponse::Ok()
        .content_type("text/html")
        .body(include_str!("../static/index.html"))
}

async fn upload_pdf(mut payload: Multipart) -> impl Responder {
    let mut pdf_data = Vec::new();
    let mut is_pdf = false;

    while let Ok(Some(mut field)) = payload.try_next().await {
        if let Some(content_type) = field.content_type() {
            if content_type.type_() == "application" && content_type.subtype() == "pdf" {
                is_pdf = true;
            }
        }
        while let Some(chunk) = field.next().await {
            if let Ok(data) = chunk {
                pdf_data.extend_from_slice(&data);
            }
        }
    }

    if is_pdf {
        if let Ok(mut temp_file) = NamedTempFile::new() {
            if temp_file.write_all(&pdf_data).is_ok() {
                if let Ok(text) = extract_text(temp_file.path()) {
                    let (ibans, text_preview) = extract_ibans(&text);
                    return HttpResponse::Ok().json(IbanResponse {
                        ibans,
                        text_preview,
                    });
                }
            }
        }
    }

    HttpResponse::BadRequest().json(serde_json::json!({
        "error": "Failed to process PDF"
    }))
}

fn extract_ibans(text: &str) -> (Vec<IbanMatch>, String) {
    let mut unique_ibans: HashMap<String, Confidence> = HashMap::new();

    // Label-based detection (highest confidence)
    let label_pattern = Regex::new(r"IBAN\s*:?\s*([A-Z]{2}\s*[0-9]{2}\s*[0-9\s]{4,32})").unwrap();
    for cap in label_pattern.captures_iter(text) {
        let iban = cap[1].replace(" ", "");
        unique_ibans.entry(iban).or_insert(Confidence::High);
    }

    // Pattern-based detection (medium confidence)
    let pattern = Regex::new(r"[A-Z]{2}\s*[0-9]{2}\s*[0-9]{4}\s*[0-9]{7}(?:\s*[0-9]{3})?").unwrap();
    for cap in pattern.find_iter(text) {
        let iban = cap.as_str().replace(" ", "");
        unique_ibans.entry(iban).or_insert(Confidence::Medium);
    }

    // Simple pattern (low confidence)
    let simple_pattern = Regex::new(r"[A-Z]{2}[0-9]{2}[0-9]{4,}").unwrap();
    for cap in simple_pattern.find_iter(text) {
        let iban = cap.as_str().to_string();
        unique_ibans.entry(iban).or_insert(Confidence::Low);
    }

    // Fallback detection (lowest confidence)
    let fallback_pattern = Regex::new(r"[A-Z]{2}[0-9]{2}[0-9\s]{10,}").unwrap();
    for cap in fallback_pattern.find_iter(text) {
        let iban = cap.as_str().replace(" ", "");
        unique_ibans.entry(iban).or_insert(Confidence::Fallback);
    }

    let mut ibans: Vec<IbanMatch> = unique_ibans
        .into_iter()
        .map(|(iban, confidence)| IbanMatch { iban, confidence })
        .collect();

    // Sort by confidence level (High -> Medium -> Low -> Fallback)
    ibans.sort_by(|a, b| b.confidence.cmp(&a.confidence));

    // Create text preview (first 500 characters)
    let text_preview = text
        .chars()
        .skip_while(|c| c.is_whitespace())
        .take(500)
        .collect::<String>();

    (ibans, text_preview)
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    println!("Server starting at http://localhost:8080");
    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_pdf))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
