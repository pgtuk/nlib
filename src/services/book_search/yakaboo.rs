// trully a product of gemini
use std::collections::HashMap;
use thiserror::Error;

use super::HTTP_CLIENT;
use crate::models::BookDraft;

#[derive(Error, Debug)]
pub enum YakabooError {
    #[error("Network error while querying Yakaboo: {0}")]
    Network(#[from] reqwest::Error),
}

pub async fn search_yakaboo(query: &str) -> Result<Vec<BookDraft>, YakabooError> {
    let url = "https://www.yakaboo.ua/ua/search";
    let resp = HTTP_CLIENT
        .get(url)
        .query(&[("q", query)])
        .header(
            "User-Agent",
            "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/128.0.0.0 Safari/537.36",
        )
        .header("Accept-Language", "uk-UA,uk;q=0.9,en;q=0.8")
        .send()
        .await?
        .error_for_status()?;

    let html = resp.text().await?;
    Ok(parse_yakaboo_search_results(&html))
}

fn extract_between<'a>(src: &'a str, start: &str, end: &str) -> Option<&'a str> {
    let s_idx = src.find(start)? + start.len();
    let e_idx = src[s_idx..].find(end)?;
    Some(&src[s_idx..s_idx + e_idx])
}

fn extract_url_from_style(chunk: &str) -> Option<String> {
    let url_start = chunk.find("url(")? + 4;
    let url_end = chunk[url_start..].find(')')?;
    let raw =
        chunk[url_start..url_start + url_end].trim_matches(|c| c == '\'' || c == '"' || c == ' ');
    if !raw.ends_with(".svg") && !raw.contains("placeholder") && raw.starts_with("http") {
        Some(raw.to_string())
    } else {
        None
    }
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

fn is_image_path(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".jpg")
        || lower.ends_with(".jpeg")
        || lower.ends_with(".png")
        || lower.ends_with(".webp")
}

fn safe_slice(s: &str, mut start: usize, mut end: usize) -> &str {
    start = start.min(s.len());
    end = end.min(s.len());
    while start > 0 && !s.is_char_boundary(start) {
        start -= 1;
    }
    while end < s.len() && !s.is_char_boundary(end) {
        end += 1;
    }
    if start <= end {
        &s[start..end]
    } else {
        ""
    }
}

fn extract_images_map(html: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();

    for pattern in &["\"image\":\"", "\"small_image\":\""] {
        let mut cursor = 0;
        while let Some(pos) = html[cursor..].find(pattern) {
            let start = cursor + pos + pattern.len();
            let Some(img_end) = html[start..].find('"') else {
                break;
            };
            let raw_img = &html[start..start + img_end];
            cursor = start + img_end;

            if !is_image_path(raw_img) {
                continue;
            }

            let cleaned_img = raw_img
                .replace(r"\u002F", "/")
                .replace(r"\/", "/")
                .replace('\\', "");

            let path_slug = if !cleaned_img.starts_with('/') {
                format!("/{cleaned_img}")
            } else {
                cleaned_img
            };

            let full_url = format!(
                "https://static.yakaboo.ua/media/cloudflare/product/webp/352x340{path_slug}"
            );

            // Fast O(1) sliding window slice around image (+/- 600 bytes)
            let w_start = start.saturating_sub(600);
            let w_end = start + 600;
            let window = safe_slice(html, w_start, w_end);

            for key in &["\"url_path\":\"", "\"url_key\":\"", "\"slug\":\""] {
                let mut k_cursor = 0;
                while let Some(kp) = window[k_cursor..].find(key) {
                    let vs = k_cursor + kp + key.len();
                    let Some(ve) = window[vs..].find('"') else {
                        break;
                    };
                    let val = &window[vs..vs + ve];
                    let slug = if val.ends_with(".html") {
                        val.to_string()
                    } else {
                        format!("{val}.html")
                    };
                    map.insert(slug, full_url.clone());
                    k_cursor = vs + ve;
                }
            }

            if let Some(sku) = extract_between(window, "\"sku\":\"", "\"") {
                map.insert(sku.to_string(), full_url.clone());
            }
            if let Some(parent_sku) = extract_between(window, "\"parentSku\":\"", "\"") {
                map.insert(parent_sku.to_string(), full_url);
            }
        }
    }

    map
}

fn parse_yakaboo_search_results(html_content: &str) -> Vec<BookDraft> {
    let images_map = extract_images_map(html_content);
    let mut results = Vec::new();

    // Skip chunk 0 which contains page header and CSS styles
    for chunk in html_content.split("class=\"category-card").skip(1) {
        // Extract title
        let Some(title_raw) = extract_between(chunk, "category-card__name", "</a>") else {
            continue;
        };
        let title_content = title_raw
            .find('>')
            .map(|i| &title_raw[i + 1..])
            .unwrap_or(title_raw);
        let title = strip_tags(title_content);
        if title.is_empty() {
            continue;
        }

        // Extract author
        let author = extract_between(chunk, "creator-label", "</span>")
            .and_then(|raw| raw.find('>').map(|i| &raw[i + 1..]))
            .map(strip_tags)
            .unwrap_or_default();

        // Extract slug
        let slug = extract_between(chunk, "href=\"/ua/", "\"")
            .or_else(|| extract_between(chunk, "href=\"", "\"").map(|h| h.trim_start_matches('/')))
            .unwrap_or_default();

        let id = extract_between(chunk, "id=\"", "\"").unwrap_or_default();

        // Check 1. Direct inline CSS background-image style (used for digital/downloadable products)
        // Check 2. JSON images map by slug or ID/SKU
        // Check 3. data-src or src attribute on product image
        let cover_url = extract_url_from_style(chunk)
            .or_else(|| images_map.get(slug).cloned())
            .or_else(|| {
                if !id.is_empty() {
                    images_map.get(id).cloned()
                } else {
                    None
                }
            })
            .or_else(|| {
                extract_between(chunk, "data-src=\"", "\"").and_then(|src| {
                    if src.starts_with("http") && !src.contains("placeholder") {
                        Some(src.to_string())
                    } else {
                        None
                    }
                })
            })
            .unwrap_or_default();

        let mut draft = BookDraft::new(title, author);
        if !cover_url.is_empty() {
            draft = draft.with_cover_url(cover_url);
        }
        results.push(draft);
    }

    results
}
