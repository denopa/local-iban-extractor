use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use futures::{StreamExt, TryStreamExt};
use log::{debug, info};
use pdf_extract::extract_text;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Serialize, Deserialize)]
struct IbanMatch {
    iban: String,
    confidence: Confidence,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum Confidence {
    High,     // Label-based detection
    Medium,   // Pattern-based detection
    Low,      // Simple pattern
    Fallback, // Loose pattern
}

#[derive(Serialize, Deserialize)]
struct IbanResponse {
    ibans: Vec<IbanMatch>,
    text_preview: String,
}

async fn extract_ibans(pdf_data: &[u8]) -> (Vec<IbanMatch>, String) {
    let mut ibans_map: HashMap<String, Confidence> = HashMap::new();
    let mut text_preview = String::new();

    // Create a temporary file to store the PDF
    if let Ok(mut temp_file) = NamedTempFile::new() {
        if temp_file.write_all(pdf_data).is_ok() {
            if let Ok(text) = extract_text(temp_file.path()) {
                // Create text preview (first 500 characters)
                text_preview = text
                    .chars()
                    .skip_while(|c| c.is_whitespace())
                    .take(500)
                    .collect::<String>();
                debug!("Extracted text from PDF: {}", text);

                // 1. Label-based detection (High confidence)
                let label_pattern =
                    Regex::new(r"(?i)IBAN\s*:?\s*([A-Z]{2}\s*[0-9]{2}\s*[0-9\s]{4,32})").unwrap();
                for cap in label_pattern.captures_iter(&text) {
                    if let Some(iban_match) = cap.get(1) {
                        let iban = iban_match.as_str().replace(" ", "");
                        if iban.len() >= 15 && iban.len() <= 34 {
                            ibans_map.insert(iban, Confidence::High);
                        }
                    }
                }

                // 2. Pattern-based detection (Medium confidence)
                let pattern =
                    Regex::new(r"[A-Z]{2}\s*[0-9]{2}\s*[0-9]{4}\s*[0-9]{7}(?:\s*[0-9]{3})?")
                        .unwrap();
                for cap in pattern.find_iter(&text) {
                    let iban = cap.as_str().replace(" ", "");
                    if !ibans_map.contains_key(&iban) {
                        ibans_map.insert(iban, Confidence::Medium);
                    }
                }

                // 3. Simple pattern (Low confidence)
                let simple_pattern = Regex::new(r"[A-Z]{2}[0-9]{2}[0-9]{4,}").unwrap();
                for cap in simple_pattern.find_iter(&text) {
                    let iban = cap.as_str().replace(" ", "");
                    if !ibans_map.contains_key(&iban) {
                        ibans_map.insert(iban, Confidence::Low);
                    }
                }

                // 4. Fallback pattern (Fallback confidence)
                let fallback_pattern = Regex::new(r"[A-Z]{2}[0-9]{2}[0-9\s]{10,}").unwrap();
                for cap in fallback_pattern.find_iter(&text) {
                    let iban = cap.as_str().replace(" ", "");
                    if !ibans_map.contains_key(&iban) {
                        ibans_map.insert(iban, Confidence::Fallback);
                    }
                }

                debug!(
                    "Found {} unique IBANs with various confidence levels",
                    ibans_map.len()
                );
                for (iban, confidence) in &ibans_map {
                    debug!("IBAN: {} (Confidence: {:?})", iban, confidence);
                }
            } else {
                debug!("Failed to extract text from PDF");
            }
        } else {
            debug!("Failed to write PDF data to temp file");
        }
    } else {
        debug!("Failed to create temp file");
    }

    let ibans: Vec<IbanMatch> = ibans_map
        .into_iter()
        .map(|(iban, confidence)| IbanMatch { iban, confidence })
        .collect();

    (ibans, text_preview)
}

async fn upload_pdf(mut payload: Multipart) -> impl Responder {
    let mut ibans = Vec::new();
    let mut text_preview = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        if let Some(content_type) = field.content_type() {
            debug!(
                "Received file with content type: {}/{}",
                content_type.type_(),
                content_type.subtype()
            );
            if content_type.type_() == "application" && content_type.subtype() == "pdf" {
                let mut data = Vec::new();
                while let Some(chunk) = field.next().await {
                    if let Ok(chunk_data) = chunk {
                        data.extend_from_slice(&chunk_data);
                    }
                }
                debug!("Received PDF data of size: {} bytes", data.len());

                let (extracted_ibans, preview) = extract_ibans(&data).await;
                ibans = extracted_ibans;
                text_preview = preview;
                break;
            }
        }
    }

    HttpResponse::Ok().json(IbanResponse {
        ibans,
        text_preview,
    })
}

async fn index() -> impl Responder {
    HttpResponse::Ok().body(include_str!("../static/index.html"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("debug"));
    info!("Server starting at http://localhost:8080");

    HttpServer::new(|| {
        App::new()
            .route("/", web::get().to(index))
            .route("/upload", web::post().to(upload_pdf))
            .service(actix_files::Files::new("/static", "./static").show_files_listing())
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
