#![no_std]

mod auth;
mod networking;
mod parser;

use aidoku::{
    prelude::*,
    alloc::{string::String, vec::Vec, format},
    Chapter, FilterValue, Manga, MangaPageResult, Page, Result, Source,
};

struct ComicsTrackerSource;

impl Source for ComicsTrackerSource {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        _page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let (url, is_search) = if let Some(q) = query {
            if q.trim().is_empty() {
                (String::from("/"), false)
            } else {
                let encoded = q.replace(" ", "%20")
                    .replace("&", "%26")
                    .replace("?", "%3F")
                    .replace("#", "%23")
                    .replace("=", "%3D");
                (format!("/api/search?q={}", encoded), true)
            }
        } else {
            (String::from("/"), false)
        };

        let req = networking::create_get_request(&url)?;
        let data = req.data()?;
        
        let (entries, has_next_page) = if is_search {
            parser::parse_manga_list(&data)?
        } else {
            parser::parse_home_page(&data)?
        };

        Ok(MangaPageResult {
            entries,
            has_next_page, 
        })
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let mut updated_manga = manga.clone();
        
        let fallback = format!("https://comics-tracker.net/comics/{}", manga.key);
        let url = manga.url.as_deref().unwrap_or(&fallback);
        let req = networking::create_get_request(url)?;
        let bytes = req.data()?;

        if needs_details {
            parser::update_manga_details(&mut updated_manga, &bytes)?;
        }

        if needs_chapters {
            let chapters = parser::parse_chapter_list(&bytes, &manga.key)?;
            updated_manga.chapters = Some(chapters);
        }

        Ok(updated_manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let parts: Vec<&str> = chapter.key.split('|').collect();
        if parts.len() < 3 {
            let mut pages = Vec::new();
            pages.push(Page {
                content: aidoku::PageContent::Url(String::from("https://placehold.co/800x600.png?text=Veuillez+rafraichir+la+liste+des+chapitres"), None),
                ..Default::default()
            });
            return Ok(pages);
        }
        
        let prefix = parts[0];
        let start: usize = parts[1].parse().unwrap_or(0);
        let end: usize = parts[2].parse().unwrap_or(99999);
        
        use base64::{engine::general_purpose::STANDARD, Engine};
        let prefix64 = STANDARD.encode(prefix);
        
        let url = format!("https://comics-tracker.net/api/r2-proxy/list?prefix64={}", prefix64);
        let req = networking::create_get_request(&url)?
            .header("Accept", "application/json")
            .header("Accept-Encoding", "gzip, deflate");
        
        let bytes = req.data()?;
        
        // Si on reçoit une erreur, invalider le cache et réessayer une fois
        if let Ok(text) = core::str::from_utf8(&bytes) {
            if text.contains("error") || text.contains("unauthorized") || text.contains("Invalid") || text.contains("Missing") {
                auth::invalidate_token_cache();
                // Recréer la requête avec un nouveau token
                let req2 = networking::create_get_request(&url)?
                    .header("Accept", "application/json")
                    .header("Accept-Encoding", "gzip, deflate");
                let bytes2 = req2.data()?;
                return parser::parse_page_list(&bytes2, start, end, prefix);
            }
        }
        
        parser::parse_page_list(&bytes, start, end, prefix)
    }
}

register_source!(ComicsTrackerSource);
