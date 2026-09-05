use std::process::Command;

use super::BookSearchError;
use crate::models::BookDraft;

pub async fn search_yakaboo(query: &str) -> Result<Vec<BookDraft>, BookSearchError> {
    let q = query.to_string();

    let output_str = relm4::spawn_blocking(move || -> Result<String, std::io::Error> {
        let url = "https://api2.yakaboo.ua/api/catalog/vue_storefront_catalog_2/product/_search?from=0&size=8";

        let payload = serde_json::json!({
            "query": {
                "bool": {
                    "should": [
                        { "match_phrase": { "name": { "query": q, "boost": 3 } } },
                        { "match": { "name": { "query": q, "fuzziness": "auto", "operator": "OR" } } },
                        { "prefix": { "name": { "value": q } } },
                        { "match": { "meta_keywords": { "query": q, "boost": 2 } } }
                    ],
                    "minimum_should_match": 1
                }
            },
            "min_score": 1,
            "track_total_hits": true
        });

        let payload_str = payload.to_string();

        let output = Command::new("curl")
            .arg("-s")
            .arg("-X")
            .arg("POST")
            .arg(url)
            .arg("-H")
            .arg("Content-Type: application/json")
            .arg("-H")
            .arg("Accept: application/json")
            .arg("-H")
            .arg("User-Agent: Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0")
            .arg("-H")
            .arg("Referer: https://www.yakaboo.ua/")
            .arg("-H")
            .arg("Origin: https://www.yakaboo.ua")
            .arg("--data")
            .arg(&payload_str)
            .output()?;

        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    })
    .await
    .map_err(|e| BookSearchError::Provider(e.to_string()))??;

    let parsed: serde_json::Value = serde_json::from_str(&output_str)
        .map_err(|e| BookSearchError::Provider(format!("JSON parse error: {e}")))?;

    let mut results = Vec::new();
    if let Some(hits) = parsed["hits"]["hits"].as_array() {
        for hit in hits {
            let src = &hit["_source"];
            let name = src["name"].as_str().unwrap_or_default().trim().to_string();
            if name.is_empty() {
                continue;
            }

            let author = extract_author(src);

            let img = src["image"]
                .as_str()
                .or_else(|| src["small_image"].as_str())
                .or_else(|| src["thumbnail"].as_str())
                .unwrap_or_default();

            let cover_url = if !img.is_empty() {
                if img.starts_with("http") {
                    img.to_string()
                } else {
                    let clean = img.trim_start_matches('/');
                    format!(
                        "https://static.yakaboo.ua/media/cloudflare/product/webp/352x340/{clean}"
                    )
                }
            } else {
                String::new()
            };

            let description = src["short_description"]
                .as_str()
                .or_else(|| src["description"].as_str())
                .map(strip_tags)
                .unwrap_or_default();

            let mut draft = BookDraft::new(name, author).with_description(description);
            if !cover_url.is_empty() {
                draft = draft.with_cover_url(cover_url);
            }

            results.push(draft);
        }
    }

    println!("[Yakaboo Curl] Parsed {} books", results.len());
    Ok(results)
}

fn extract_author(src: &serde_json::Value) -> String {
    for key in &[
        "author",
        "author_name",
        "author_label",
        "creator",
        "authors",
        "book_author",
        "author_full_name",
    ] {
        if let Some(s) = src[key].as_str() {
            if !s.is_empty() {
                return s.to_string();
            }
        }
        if let Some(arr) = src[key].as_array() {
            let names: Vec<String> = arr
                .iter()
                .filter_map(|item| {
                    item.as_str()
                        .or_else(|| item["name"].as_str())
                        .or_else(|| item["label"].as_str())
                        .map(str::to_string)
                })
                .collect();
            if !names.is_empty() {
                return names.join(", ");
            }
        }
    }

    if let Some(attrs) = src["custom_attributes"].as_array() {
        for attr in attrs {
            if attr["attribute_code"] == "author" || attr["attribute_code"] == "author_name" {
                if let Some(v) = attr["value"].as_str() {
                    return v.to_string();
                }
            }
        }
    }

    String::new()
}

fn strip_tags(src: &str) -> String {
    let mut out = String::with_capacity(src.len());
    let mut in_tag = false;
    for ch in src.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.trim().to_string()
}
