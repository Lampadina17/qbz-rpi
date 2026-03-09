//! MusicBrainz API client
//!
//! HTTP client with rate limiting and proper User-Agent handling
//! Uses Cloudflare Workers proxy for consistent rate limiting

use reqwest::Client;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

use super::models::*;

/// Proxy URL for MusicBrainz requests
const MUSICBRAINZ_PROXY_URL: &str = "https://qbz-api-proxy.blitzkriegfc.workers.dev/musicbrainz";

/// Direct MusicBrainz API URL (fallback)
const MUSICBRAINZ_API_URL: &str = "https://musicbrainz.org/ws/2";

/// Rate limiter for MusicBrainz API
pub struct RateLimiter {
    last_request: Mutex<Instant>,
    min_interval: Duration,
}

impl RateLimiter {
    /// Create rate limiter for direct MusicBrainz API (1 req/sec)
    pub fn new() -> Self {
        Self::with_interval(Duration::from_millis(1100))
    }

    /// Create rate limiter for proxy (faster, proxy handles actual rate limiting)
    pub fn for_proxy() -> Self {
        Self::with_interval(Duration::from_millis(200))
    }

    /// Create rate limiter with custom interval
    pub fn with_interval(min_interval: Duration) -> Self {
        Self {
            // Start in the past so first request doesn't wait
            last_request: Mutex::new(Instant::now() - Duration::from_secs(2)),
            min_interval,
        }
    }

    pub async fn wait(&self) {
        let mut last = self.last_request.lock().await;
        let elapsed = last.elapsed();
        if elapsed < self.min_interval {
            tokio::time::sleep(self.min_interval - elapsed).await;
        }
        *last = Instant::now();
    }
}

impl Default for RateLimiter {
    fn default() -> Self {
        Self::new()
    }
}

/// MusicBrainz API client configuration
#[derive(Debug, Clone)]
pub struct MusicBrainzConfig {
    /// Whether MusicBrainz integration is enabled
    pub enabled: bool,
    /// Use proxy instead of direct API
    pub use_proxy: bool,
}

impl Default for MusicBrainzConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            use_proxy: true,
        }
    }
}

/// MusicBrainz API client
pub struct MusicBrainzClient {
    client: Client,
    rate_limiter: Arc<RateLimiter>,
    config: Arc<Mutex<MusicBrainzConfig>>,
}

impl Default for MusicBrainzClient {
    fn default() -> Self {
        Self::new()
    }
}

impl MusicBrainzClient {
    pub fn new() -> Self {
        Self::with_config(MusicBrainzConfig::default())
    }

    pub fn with_config(config: MusicBrainzConfig) -> Self {
        let version = env!("CARGO_PKG_VERSION");
        let user_agent = format!(
            "QBZ/{} (https://github.com/vicrodh/qbz; qbz@vicrodh.dev)",
            version
        );

        let client = Client::builder()
            .user_agent(&user_agent)
            .timeout(Duration::from_secs(10))
            .build()
            .unwrap_or_else(|_| Client::new());

        // Use faster rate limiter when using proxy (proxy handles actual rate limiting)
        let rate_limiter = if config.use_proxy {
            RateLimiter::for_proxy()
        } else {
            RateLimiter::new()
        };

        Self {
            client,
            rate_limiter: Arc::new(rate_limiter),
            config: Arc::new(Mutex::new(config)),
        }
    }

    /// Check if MusicBrainz integration is enabled
    pub async fn is_enabled(&self) -> bool {
        self.config.lock().await.enabled
    }

    /// Enable or disable MusicBrainz integration
    pub async fn set_enabled(&self, enabled: bool) {
        self.config.lock().await.enabled = enabled;
    }

    /// Get the base URL based on configuration
    async fn base_url(&self) -> &'static str {
        if self.config.lock().await.use_proxy {
            MUSICBRAINZ_PROXY_URL
        } else {
            MUSICBRAINZ_API_URL
        }
    }

    /// Search recordings by ISRC
    pub async fn search_recording_by_isrc(
        &self,
        isrc: &str,
    ) -> Result<RecordingSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let url = format!(
            "{}/recording?query=isrc:{}&fmt=json&limit=5",
            base_url, isrc
        );

        log::debug!("MusicBrainz recording search by ISRC: {}", isrc);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<RecordingSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search recordings by title and artist
    pub async fn search_recording(
        &self,
        title: &str,
        artist: &str,
    ) -> Result<RecordingSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let query = format!(
            "recording:\"{}\" AND artist:\"{}\"",
            Self::escape_query(title),
            Self::escape_query(artist)
        );
        let url = format!(
            "{}/recording?query={}&fmt=json&limit=5",
            base_url,
            urlencoding::encode(&query)
        );

        log::debug!("MusicBrainz recording search: {} - {}", artist, title);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<RecordingSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search artists by name
    pub async fn search_artist(&self, name: &str) -> Result<ArtistSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let query = format!("artist:\"{}\"", Self::escape_query(name));
        let url = format!(
            "{}/artist?query={}&fmt=json&limit=5",
            base_url,
            urlencoding::encode(&query)
        );

        log::debug!("MusicBrainz artist search: {}", name);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ArtistSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Get artist details with relationships
    pub async fn get_artist_with_relations(
        &self,
        mbid: &str,
    ) -> Result<ArtistFullResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let url = format!("{}/artist/{}?inc=artist-rels+tags&fmt=json", base_url, mbid);

        log::debug!("MusicBrainz artist lookup with relations: {}", mbid);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ArtistFullResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Fetch artist tags only (lightweight, no relations)
    pub async fn get_artist_tags(
        &self,
        mbid: &str,
    ) -> Result<Vec<String>, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let url = format!("{}/artist/{}?inc=tags&fmt=json", base_url, mbid);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(Vec::new());
        }

        let artist: ArtistFullResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))?;

        let mut tags: Vec<_> = artist
            .tags
            .unwrap_or_default()
            .into_iter()
            .filter(|tag| tag.count.unwrap_or(0) > 0)
            .collect();

        // Sort by vote count descending — highest voted tag = primary genre
        tags.sort_by(|a, b| b.count.unwrap_or(0).cmp(&a.count.unwrap_or(0)));

        Ok(tags.into_iter().map(|tag| tag.name.to_lowercase()).collect())
    }

    /// Search artists by tag (genre)
    ///
    /// Returns artists tagged with the given genre on MusicBrainz,
    /// sorted by search relevance. Used for "Listeners also enjoy" discovery.
    pub async fn search_artists_by_tag(
        &self,
        tag: &str,
        limit: usize,
    ) -> Result<ArtistSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let limit = limit.min(100).max(1);
        let query = format!("tag:\"{}\"", Self::escape_query(tag));
        let url = format!(
            "{}/artist?query={}&fmt=json&limit={}",
            base_url,
            urlencoding::encode(&query),
            limit
        );

        log::debug!("MusicBrainz artist search by tag '{}' (limit {})", tag, limit);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ArtistSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search artists by tag AND area (genre + location combined).
    ///
    /// Uses MB Lucene query: `tag:"thrash metal" AND beginarea:"Los Angeles"`
    /// This is much more precise than browse-all-then-filter.
    pub async fn search_artists_by_tag_and_area(
        &self,
        tag: &str,
        area_name: &str,
        limit: usize,
        offset: usize,
    ) -> Result<ArtistSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let limit = limit.min(100).max(1);
        let query = format!(
            "tag:\"{}\" AND (beginarea:\"{}\" OR area:\"{}\")",
            Self::escape_query(tag),
            Self::escape_query(area_name),
            Self::escape_query(area_name),
        );
        let url = format!(
            "{}/artist?query={}&fmt=json&limit={}&offset={}",
            base_url,
            urlencoding::encode(&query),
            limit,
            offset
        );

        log::debug!(
            "MusicBrainz artist search by tag '{}' + area '{}' (limit {}, offset {})",
            tag,
            area_name,
            limit,
            offset
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ArtistSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search releases by barcode (UPC/EAN)
    pub async fn search_release_by_barcode(
        &self,
        barcode: &str,
    ) -> Result<ReleaseSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let url = format!(
            "{}/release?query=barcode:{}&fmt=json&limit=5",
            base_url, barcode
        );

        log::debug!("MusicBrainz release search by barcode: {}", barcode);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ReleaseSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search releases by title and artist
    pub async fn search_release(
        &self,
        title: &str,
        artist: &str,
    ) -> Result<ReleaseSearchResponse, String> {
        self.search_releases_extended(title, artist, None, 5).await
    }

    /// Search releases with extended options
    /// - `title`: Album title to search
    /// - `artist`: Artist name to search
    /// - `catalog_number`: Optional catalog number for more precise matching
    /// - `limit`: Maximum results to return (1-25)
    pub async fn search_releases_extended(
        &self,
        title: &str,
        artist: &str,
        catalog_number: Option<&str>,
        limit: usize,
    ) -> Result<ReleaseSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;

        // Build query - if catalog number provided, prioritize it
        let query = if let Some(catno) = catalog_number.filter(|s| !s.trim().is_empty()) {
            format!(
                "catno:\"{}\" AND artist:\"{}\"",
                Self::escape_query(catno),
                Self::escape_query(artist)
            )
        } else {
            format!(
                "release:\"{}\" AND artist:\"{}\"",
                Self::escape_query(title),
                Self::escape_query(artist)
            )
        };

        let limit = limit.min(25).max(1);
        let url = format!(
            "{}/release?query={}&fmt=json&limit={}",
            base_url,
            urlencoding::encode(&query),
            limit
        );

        log::debug!(
            "MusicBrainz release search: {} - {} (catalog: {:?}, limit: {})",
            artist,
            title,
            catalog_number,
            limit
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ReleaseSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Get full release details including tracks
    /// Fetches media, recordings, artist credits, labels, and tags
    pub async fn get_release_with_tracks(
        &self,
        release_id: &str,
    ) -> Result<ReleaseFullResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        // inc=recordings gets track info, artist-credits for artist info,
        // labels for label/catalog, tags for genres
        let url = format!(
            "{}/release/{}?inc=recordings+artist-credits+labels+tags&fmt=json",
            base_url, release_id
        );

        log::debug!("MusicBrainz release lookup with tracks: {}", release_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ReleaseFullResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Browse artists by area MBID
    ///
    /// Returns artists associated with the given area (city/country).
    /// Uses the MB browse API which is more precise than Lucene area search.
    pub async fn browse_artists_by_area(
        &self,
        area_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<ArtistBrowseResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let limit = limit.min(100).max(1);
        let url = format!(
            "{}/artist?area={}&fmt=json&limit={}&offset={}&inc=tags",
            base_url, area_id, limit, offset
        );

        log::debug!(
            "MusicBrainz browse artists by area {} (limit {}, offset {})",
            area_id, limit, offset
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<ArtistBrowseResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Search for an area by name (to resolve area MBID)
    pub async fn search_area(
        &self,
        name: &str,
        area_type: Option<&str>,
    ) -> Result<AreaSearchResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let query = if let Some(atype) = area_type {
            format!(
                "area:\"{}\" AND type:\"{}\"",
                Self::escape_query(name),
                Self::escape_query(atype)
            )
        } else {
            format!("area:\"{}\"", Self::escape_query(name))
        };

        let url = format!(
            "{}/area?query={}&fmt=json&limit=5",
            base_url,
            urlencoding::encode(&query)
        );

        log::debug!("MusicBrainz area search: {} (type: {:?})", name, area_type);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<AreaSearchResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz response: {}", e))
    }

    /// Look up an area and its parent relationships.
    ///
    /// Used to resolve city → subdivision (e.g., Leyton → England).
    /// Returns the area detail with `relations` that include "part of" links.
    pub async fn get_area_with_relations(
        &self,
        area_id: &str,
    ) -> Result<AreaDetailResponse, String> {
        if !self.is_enabled().await {
            return Err("MusicBrainz integration is disabled".to_string());
        }

        self.rate_limiter.wait().await;

        let base_url = self.base_url().await;
        let url = format!(
            "{}/area/{}?inc=area-rels&fmt=json",
            base_url, area_id
        );

        log::debug!("MusicBrainz area lookup with relations: {}", area_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("MusicBrainz request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err(format!("MusicBrainz API error {}: {}", status, text));
        }

        response
            .json::<AreaDetailResponse>()
            .await
            .map_err(|e| format!("Failed to parse MusicBrainz area response: {}", e))
    }

    /// Resolve a city area to its parent subdivision (state/region).
    ///
    /// Uses MB area-rels "part of" with direction "forward" to go UP:
    ///   Los Angeles → "part of" (forward) → California (subdivision)
    ///   Leyton → "part of" (forward) → Waltham Forest → (forward) → England
    ///
    /// If the direct parent is too granular (district/municipality), walks
    /// up one more level to find a proper subdivision.
    pub async fn resolve_parent_subdivision(
        &self,
        area_id: &str,
    ) -> Result<Option<(String, String)>, String> {
        let detail = self.get_area_with_relations(area_id).await?;

        // Collect PARENT relations only (direction "forward" = "I am part of X")
        let parents: Vec<_> = detail
            .relations
            .as_ref()
            .map(|rels| {
                rels.iter()
                    .filter(|rel| {
                        rel.relation_type == "part of"
                            && rel.direction.as_deref() == Some("forward")
                    })
                    .filter_map(|rel| rel.area.as_ref())
                    .collect()
            })
            .unwrap_or_default();

        if parents.is_empty() {
            log::info!(
                "No parent found for area '{}' ({}), type={:?}",
                detail.name,
                area_id,
                detail.area_type
            );
            return Ok(None);
        }

        // Log all parents for debugging
        for parent in &parents {
            log::info!(
                "  Parent of '{}': '{}' ({}) type={:?}",
                detail.name,
                parent.name,
                parent.id,
                parent.area_type
            );
        }

        // Priority 1: Find a subdivision parent
        if let Some(subdiv) = parents.iter().find(|p| {
            p.area_type
                .as_deref()
                .map(|t| t.eq_ignore_ascii_case("subdivision"))
                .unwrap_or(false)
        }) {
            log::info!(
                "Resolved '{}' → subdivision '{}' ({})",
                detail.name,
                subdiv.name,
                subdiv.id
            );
            return Ok(Some((subdiv.name.clone(), subdiv.id.clone())));
        }

        // Priority 2: Find a non-country, non-city parent (could be a region/state
        // with a different type label)
        if let Some(region) = parents.iter().find(|p| {
            let t = p.area_type.as_deref().unwrap_or("");
            !t.is_empty()
                && !t.eq_ignore_ascii_case("country")
                && !t.eq_ignore_ascii_case("city")
                && !t.eq_ignore_ascii_case("municipality")
        }) {
            log::info!(
                "Resolved '{}' → {} '{}' ({})",
                detail.name,
                region.area_type.as_deref().unwrap_or("unknown"),
                region.name,
                region.id
            );
            return Ok(Some((region.name.clone(), region.id.clone())));
        }

        // Priority 3: If the only parent is a country, check if the original area
        // is already a subdivision — in that case, use it as-is
        if let Some(country_parent) = parents.iter().find(|p| {
            p.area_type
                .as_deref()
                .map(|t| t.eq_ignore_ascii_case("country"))
                .unwrap_or(false)
        }) {
            let own_type = detail.area_type.as_deref().unwrap_or("");
            if own_type.eq_ignore_ascii_case("subdivision") {
                // Area is already a subdivision (e.g., California, England)
                log::info!(
                    "'{}' is already a subdivision (parent: {}), using as-is",
                    detail.name,
                    country_parent.name
                );
                return Ok(None); // Don't resolve, the original name is fine
            }

            // Parent is directly a country — the area might be a district/borough.
            // Try walking up ONE more level from one of the non-country parents.
            // This handles: Leyton → Waltham Forest (district) → ... → England
            // But if only parent is country, just skip resolution.
            log::info!(
                "Only parent of '{}' is country '{}', no intermediate subdivision",
                detail.name,
                country_parent.name
            );
        }

        // Priority 4: If parent is a district/municipality, walk up one more level
        if let Some(district) = parents.first() {
            let parent_type = district.area_type.as_deref().unwrap_or("");
            if parent_type.eq_ignore_ascii_case("district")
                || parent_type.eq_ignore_ascii_case("municipality")
                || parent_type.eq_ignore_ascii_case("city")
            {
                log::info!(
                    "Parent '{}' is a {}, walking up one more level...",
                    district.name,
                    parent_type
                );
                // Walk up one level: resolve the parent's parent (non-recursive)
                let grandparent = self.get_area_with_relations(&district.id).await?;
                let gp_parents: Vec<_> = grandparent
                    .relations
                    .as_ref()
                    .map(|rels| {
                        rels.iter()
                            .filter(|r| {
                                r.relation_type == "part of"
                                    && r.direction.as_deref() == Some("forward")
                            })
                            .filter_map(|r| r.area.as_ref())
                            .collect()
                    })
                    .unwrap_or_default();

                // Find subdivision among grandparents
                if let Some(subdiv) = gp_parents.iter().find(|p| {
                    p.area_type
                        .as_deref()
                        .map(|t| t.eq_ignore_ascii_case("subdivision"))
                        .unwrap_or(false)
                }) {
                    log::info!(
                        "Resolved '{}' → '{}' → subdivision '{}' ({})",
                        detail.name,
                        district.name,
                        subdiv.name,
                        subdiv.id
                    );
                    return Ok(Some((subdiv.name.clone(), subdiv.id.clone())));
                }

                // If no subdivision, use the grandparent district's non-country parent
                if let Some(region) = gp_parents.iter().find(|p| {
                    let t = p.area_type.as_deref().unwrap_or("");
                    !t.is_empty() && !t.eq_ignore_ascii_case("country")
                }) {
                    log::info!(
                        "Resolved '{}' → '{}' → {} '{}' ({})",
                        detail.name,
                        district.name,
                        region.area_type.as_deref().unwrap_or("unknown"),
                        region.name,
                        region.id
                    );
                    return Ok(Some((region.name.clone(), region.id.clone())));
                }
            }
        }

        Ok(None)
    }

    /// Escape special characters in Lucene queries
    fn escape_query(s: &str) -> String {
        s.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace(':', "\\:")
            .replace('(', "\\(")
            .replace(')', "\\)")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('{', "\\{")
            .replace('}', "\\}")
            .replace('^', "\\^")
            .replace('~', "\\~")
            .replace('*', "\\*")
            .replace('?', "\\?")
            .replace('!', "\\!")
            .replace('+', "\\+")
            .replace('-', "\\-")
            .replace('&', "\\&")
            .replace('|', "\\|")
    }
}
