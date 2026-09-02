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

fn extract_images_map(html: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    let pattern = "\"url_path\":\"";
    let mut cursor = 0;
    while let Some(pos) = html[cursor..].find(pattern) {
        let start = cursor + pos + pattern.len();
        let Some(path_end) = html[start..].find('"') else {
            break;
        };
        let url_path = &html[start..start + path_end];
        cursor = start + path_end;

        let end_idx = html[cursor..]
            .char_indices()
            .nth(300)
            .map(|(i, _)| cursor + i)
            .unwrap_or(html.len());
        let lookahead = &html[cursor..end_idx];

        let img_marker = "\"image\":\"";
        if let Some(img_pos) = lookahead.find(img_marker) {
            let img_start = img_pos + img_marker.len();
            if let Some(img_end) = lookahead[img_start..].find('"') {
                let raw_img = &lookahead[img_start..img_start + img_end];
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
                map.insert(url_path.to_string(), full_url);
            }
        }
    }
    map
}

fn parse_yakaboo_search_results(html_content: &str) -> Vec<BookDraft> {
    let images_map = extract_images_map(html_content);
    let mut results = Vec::new();

    // Skip the first chunk (which contains the site header and CSS styles before the first card)
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

        // Extract slug and look up cover URL
        let slug = extract_between(chunk, "href=\"/ua/", "\"")
            .or_else(|| extract_between(chunk, "href=\"", "\"").map(|h| h.trim_start_matches('/')))
            .unwrap_or_default();

        let cover_url = images_map.get(slug).cloned().unwrap_or_default();

        let mut draft = BookDraft::new(title, author);
        if !cover_url.is_empty() {
            draft = draft.with_cover_url(cover_url);
        }
        results.push(draft);
    }

    results
}
