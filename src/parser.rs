use aidoku::{
    alloc::{vec::Vec, format, string::String},
    ContentRating, Manga, MangaStatus, Page, Result,
};
use serde_json::Value;

pub fn urlencode(s: &str) -> String {
    let mut encoded = String::new();
    for c in s.bytes() {
        match c {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' | b'/' | b'?' | b'=' | b'&' | b':' => {
                encoded.push(c as char);
            }
            _ => {
                encoded.push_str(&format!("%{:02X}", c));
            }
        }
    }
    encoded
}

pub fn parse_home_page(html_bytes: &[u8]) -> Result<(Vec<Manga>, bool)> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    if let Some(root) = get_next_data(html_bytes) {
        if let Some(props) = root.get("props").and_then(|p| p.get("pageProps")) {
            // Parser topEngagement
            if let Some(items) = props.get("topEngagement").and_then(|v| v.as_array()) {
                parse_items_array(items, &mut mangas);
            }
            // Et/Ou popularRuns si besoin
            if let Some(items) = props.get("popularRuns").and_then(|v| v.as_array()) {
                parse_items_array(items, &mut mangas);
            }
        }
    }
    
    // Enlever les doublons (certains éléments peuvent être dans topEngagement ET popularRuns)
    mangas.dedup_by(|a, b| a.key == b.key);
    
    Ok((mangas, false)) // Pas de pagination sur la page d'accueil
}

fn parse_items_array(items: &[Value], mangas: &mut Vec<Manga>) {
    for item in items {
        let id = item.get("id").and_then(|v| v.as_str()).unwrap_or_default();
        let title = item.get("french_title").and_then(|v| v.as_str()).unwrap_or_default();
        let fallback_title = item.get("title").and_then(|v| v.as_str()).unwrap_or_default();
        
        let final_title = if title.is_empty() { fallback_title } else { title };
        let image = item.get("image").and_then(|v| v.as_str()).unwrap_or_default();
        
        if id.is_empty() {
            continue;
        }

        let mut cover_url = if image.starts_with("http") {
            String::from(image)
        } else if image.starts_with("/api/image-proxy") {
            format!("https://comics-tracker.net{}", image)
        } else {
            format!("https://comics-tracker.net/api/image-proxy/image/{}?w=400", image)
        };
        cover_url = urlencode(&cover_url);

        let item_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("french_edition");
        let manga_url = match item_type {
            "french_edition" => format!("https://comics-tracker.net/comics/{}", id),
            _ => format!("https://comics-tracker.net/comics/{}", id),
        };

        mangas.push(Manga {
            key: String::from(id),
            title: String::from(final_title),
            url: Some(manga_url),
            cover: Some(cover_url),
            status: MangaStatus::Unknown,
            content_rating: ContentRating::Safe,
            ..Default::default()
        });
    }
}


pub fn parse_manga_list(json_bytes: &[u8]) -> Result<(Vec<Manga>, bool)> {
    let mut mangas: Vec<Manga> = Vec::new();
    let mut has_more = false;

    if let Ok(root) = serde_json::from_slice::<Value>(json_bytes) {
        if let Some(has_more_val) = root.get("has_more").and_then(|v| v.as_bool()) {
            has_more = has_more_val;
        }

        if let Some(items) = root.get("items").and_then(|v| v.as_array()) {
            parse_items_array(items, &mut mangas);
        }
    }

    Ok((mangas, has_more))
}

pub fn get_next_data(html_bytes: &[u8]) -> Option<Value> {
    let html = core::str::from_utf8(html_bytes).ok()?;
    let start_marker = r#"<script id="__NEXT_DATA__" type="application/json">"#;
    let end_marker = "</script>";
    
    let start = html.find(start_marker)? + start_marker.len();
    let remaining = &html[start..];
    let end = remaining.find(end_marker)?;
    let json_str = &remaining[..end];
    
    serde_json::from_str(json_str).ok()
}

pub fn update_manga_details(manga: &mut Manga, bytes: &[u8]) -> Result<()> {
    if let Some(root) = get_next_data(bytes) {
        if let Some(props) = root.get("props").and_then(|p| p.get("pageProps")) {
            // Extrait la description si elle existe
            let run = props.get("run").or(props.get("edition"));
            if let Some(r) = run {
                if let Some(desc) = r.get("description").and_then(|d| d.as_str()) {
                    manga.description = Some(String::from(desc));
                }
                if let Some(title) = r.get("title").and_then(|d| d.as_str()) {
                    manga.title = String::from(title);
                }
            }
        }
    }
    manga.status = MangaStatus::Completed;
    Ok(())
}

pub fn parse_chapter_list(bytes: &[u8], _manga_key: &str) -> Result<Vec<aidoku::Chapter>> {
    let mut chapters = Vec::new();
    
    if let Some(root) = get_next_data(bytes) {
        if let Some(props) = root.get("props").and_then(|p| p.get("pageProps")) {
            // Cas d'un run: on liste les frenchEditions
            if let Some(run) = props.get("run") {
                if let Some(sections) = run.get("sections").and_then(|s| s.as_array()) {
                    let mut chapter_num = 1.0;
                    for section in sections {
                        if let Some(editions) = section.get("frenchEditions").and_then(|e| e.as_array()) {
                            for edition in editions {
                                let id = edition.get("id").and_then(|i| i.as_str()).unwrap_or_default();
                                let title = edition.get("french_title").and_then(|t| t.as_str()).unwrap_or_default();
                                let link = edition.get("link").and_then(|l| l.as_str()).unwrap_or_default();
                                
                                if id.is_empty() {
                                    continue;
                                }
                                
                                // Si l'édition contient des issues individuelles, les éclater
                                if let (Some(issue_ids), Some(labels)) = (
                                    edition.get("issue_ids").and_then(|i| i.as_array()),
                                    edition.get("labels").and_then(|l| l.as_array())
                                ) {
                                    let table = edition.get("table_content").and_then(|t| t.as_array());
                                    for (i, _issue_id) in issue_ids.iter().enumerate() {
                                        let issue_title = labels.get(i).and_then(|l| l.as_str()).unwrap_or(title);
                                        let start = table.and_then(|t| t.get(i)).and_then(|v| v.as_u64()).unwrap_or(0);
                                        let end = table.and_then(|t| t.get(i + 1)).and_then(|v| v.as_u64()).unwrap_or(99999);
                                        
                                        chapters.push(aidoku::Chapter {
                                            key: format!("{}|{}|{}", link, start, end),
                                            title: Some(String::from(issue_title)),
                                            chapter_number: Some(chapter_num),
                                            url: Some(format!("https://comics-tracker.net/comics/{}", id)),
                                            ..Default::default()
                                        });
                                        chapter_num += 1.0;
                                    }
                                } else {
                                    // Pas d'issues: un seul chapitre pour l'édition entière
                                    chapters.push(aidoku::Chapter {
                                        key: format!("{}|0|99999", link),
                                        title: Some(String::from(title)),
                                        chapter_number: Some(chapter_num),
                                        url: Some(format!("https://comics-tracker.net/comics/{}", id)),
                                        ..Default::default()
                                    });
                                    chapter_num += 1.0;
                                }
                            }
                        }
                    }
                }
            }
            
            // Cas d'une edition: on liste les issues (chapitres)
            if let Some(edition) = props.get("edition") {
                let link = edition.get("link").and_then(|l| l.as_str()).unwrap_or_default();
                if let (Some(issue_ids), Some(labels)) = (
                    edition.get("issue_ids").and_then(|i| i.as_array()),
                    edition.get("labels").and_then(|l| l.as_array())
                ) {
                    let table = edition.get("table_content").and_then(|t| t.as_array());
                    let mut chapter_num = 1.0;
                    for (i, issue_id) in issue_ids.iter().enumerate() {
                        let id = issue_id.as_str().unwrap_or_default();
                        let title = labels.get(i).and_then(|l| l.as_str()).unwrap_or_default();
                        
                        let start = table.and_then(|t| t.get(i)).and_then(|v| v.as_u64()).unwrap_or(0);
                        let end = table.and_then(|t| t.get(i + 1)).and_then(|v| v.as_u64()).unwrap_or(99999);
                        
                        if !id.is_empty() {
                            chapters.push(aidoku::Chapter {
                                key: format!("{}|{}|{}", link, start, end),
                                title: Some(String::from(title)),
                                chapter_number: Some(chapter_num),
                                url: Some(format!("https://comics-tracker.net/issue/{}", id)),
                                ..Default::default()
                            });
                            chapter_num += 1.0;
                        }
                    }
                } else {
                    // S'il n'y a pas d'issues, on ajoute l'edition comme seul chapitre
                    let id = edition.get("id").and_then(|i| i.as_str()).unwrap_or_default();
                    let title = edition.get("french_title").and_then(|t| t.as_str()).unwrap_or_default();
                    
                    if !id.is_empty() {
                        chapters.push(aidoku::Chapter {
                            key: format!("{}|0|99999", link),
                            title: Some(String::from(title)),
                            chapter_number: Some(1.0),
                            url: Some(format!("https://comics-tracker.net/comics/{}", id)),
                            ..Default::default()
                        });
                    }
                }
            }
        }
    }
    
    // On inverse car Aidoku attend généralement le chapitre le plus récent en premier
    chapters.reverse();
    Ok(chapters)
}

pub fn parse_page_list(bytes: &[u8], start: usize, end: usize, prefix: &str) -> Result<Vec<Page>> {
    let mut pages = Vec::new();
    
    if let Ok(json) = serde_json::from_slice::<Value>(bytes) {
        let mut image_urls = Vec::new();
        
        // Helper function to recursively find image URLs
        fn find_images(val: &Value, urls: &mut Vec<String>) {
            match val {
                Value::String(s) => {
                    let s_lower = s.to_lowercase();
                    if s_lower.ends_with(".webp") || s_lower.ends_with(".jpg") || s_lower.ends_with(".jpeg") || s_lower.ends_with(".png") || s_lower.ends_with(".avif") {
                        urls.push(String::from(s));
                    }
                },
                Value::Array(arr) => {
                    for item in arr {
                        find_images(item, urls);
                    }
                },
                Value::Object(obj) => {
                    for (_, value) in obj {
                        find_images(value, urls);
                    }
                },
                _ => {}
            }
        }
        
        find_images(&json, &mut image_urls);
        
        // Sort alphabetically to ensure correct order
        image_urls.sort();
        
        // Remove duplicates if any
        image_urls.dedup();

        let start_idx = if start > 0 { start - 1 } else { 0 };
        let end_idx = if end > 0 { end - 1 } else { 999999 };

        for (i, img) in image_urls.into_iter().enumerate() {
            if i >= start_idx && i < end_idx {
                let img_key = if img.starts_with(prefix) { img.clone() } else { format!("{}{}", prefix, img) };
                
                let mut url = if img.starts_with("http") {
                    img
                } else if img.starts_with("/api/image-proxy") {
                    format!("https://comics-tracker.net{}", img)
                } else {
                    format!("https://comics-tracker.net/api/image-proxy/image/{}", img_key)
                };
                
                // Encoder l'URL complète pour éviter les plantages iOS
                url = urlencode(&url);
                
                pages.push(Page {
                    content: aidoku::PageContent::Url(url, None),
                    ..Default::default()
                });
            }
        }
    }
    
    if pages.is_empty() {
        let text = String::from_utf8_lossy(bytes);
        let safe_text: String = text.chars()
            .filter(|c| c.is_ascii_alphanumeric() || *c == ' ')
            .take(50)
            .collect();
        let safe_text = safe_text.replace(" ", "+");
        
        pages.push(Page {
            content: aidoku::PageContent::Url(format!("https://placehold.co/800x600.png?text=API+Error+{}", safe_text), None),
            ..Default::default()
        });
    }
    
    Ok(pages)
}
