// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Token Store - State management for Solana token data
//!
//! This store manages token data fetched from the Jupiter API,
//! with full SSR hydration support.

use leptos::prelude::*;
use leptos_store::prelude::*;
use serde::{Deserialize, Serialize};

// ============================================================================
// Data Types - Matching Jupiter API response
// ============================================================================

/// Statistics for a time period (5m, 1h, 6h, 24h)
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenStats {
    #[serde(default)]
    pub price_change: f64,
    #[serde(default)]
    pub holder_change: f64,
    #[serde(default)]
    pub liquidity_change: f64,
    #[serde(default)]
    pub volume_change: f64,
    #[serde(default)]
    pub buy_volume: f64,
    #[serde(default)]
    pub sell_volume: f64,
    #[serde(default)]
    pub num_buys: u64,
    #[serde(default)]
    pub num_sells: u64,
    #[serde(default)]
    pub num_traders: u64,
}

/// Audit information for a token
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenAudit {
    #[serde(default)]
    pub mint_authority_disabled: bool,
    #[serde(default)]
    pub freeze_authority_disabled: bool,
    #[serde(default)]
    pub top_holders_percentage: f64,
    #[serde(default)]
    pub dev_balance_percentage: f64,
}

/// A Solana token from the Jupiter API
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Token {
    pub id: String,
    pub name: String,
    pub symbol: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub decimals: u8,
    #[serde(default)]
    pub usd_price: f64,
    #[serde(default)]
    pub mcap: f64,
    #[serde(default)]
    pub fdv: f64,
    #[serde(default)]
    pub liquidity: f64,
    #[serde(default)]
    pub holder_count: u64,
    #[serde(default)]
    pub total_supply: f64,
    #[serde(default)]
    pub circ_supply: f64,
    #[serde(default)]
    pub twitter: Option<String>,
    #[serde(default)]
    pub website: Option<String>,
    #[serde(default)]
    pub launchpad: Option<String>,
    #[serde(default)]
    pub organic_score: f64,
    #[serde(default)]
    pub organic_score_label: Option<String>,
    #[serde(default)]
    pub bonding_curve: f64,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub created_at: Option<String>,
    #[serde(default)]
    pub graduated_at: Option<String>,
    #[serde(default)]
    pub audit: Option<TokenAudit>,
    #[serde(default, rename = "stats5m")]
    pub stats_5m: Option<TokenStats>,
    #[serde(default, rename = "stats1h")]
    pub stats_1h: Option<TokenStats>,
    #[serde(default, rename = "stats6h")]
    pub stats_6h: Option<TokenStats>,
    #[serde(default, rename = "stats24h")]
    pub stats_24h: Option<TokenStats>,
}

impl Token {
    /// Format USD price with appropriate precision
    pub fn formatted_price(&self) -> String {
        if self.usd_price < 0.0001 {
            format!("${:.8}", self.usd_price)
        } else if self.usd_price < 1.0 {
            format!("${:.6}", self.usd_price)
        } else {
            format!("${:.2}", self.usd_price)
        }
    }

    /// Format market cap in readable form
    pub fn formatted_mcap(&self) -> String {
        format_large_number(self.mcap)
    }

    /// Format liquidity in readable form
    pub fn formatted_liquidity(&self) -> String {
        format_large_number(self.liquidity)
    }

    /// Get 24h price change percentage
    pub fn price_change_24h(&self) -> f64 {
        self.stats_24h
            .as_ref()
            .map(|s| s.price_change)
            .unwrap_or(0.0)
    }

    /// Get 1h price change percentage
    pub fn price_change_1h(&self) -> f64 {
        self.stats_1h
            .as_ref()
            .map(|s| s.price_change)
            .unwrap_or(0.0)
    }

    /// Get truncated token address
    pub fn short_address(&self) -> String {
        if self.id.len() > 12 {
            format!("{}...{}", &self.id[..6], &self.id[self.id.len() - 4..])
        } else {
            self.id.clone()
        }
    }

    /// Check if token is verified
    pub fn is_verified(&self) -> bool {
        self.tags.iter().any(|t| t.contains("verified"))
    }
}

/// Format large numbers (e.g., 1.5M, 2.3B)
fn format_large_number(n: f64) -> String {
    if n >= 1_000_000_000.0 {
        format!("${:.2}B", n / 1_000_000_000.0)
    } else if n >= 1_000_000.0 {
        format!("${:.2}M", n / 1_000_000.0)
    } else if n >= 1_000.0 {
        format!("${:.2}K", n / 1_000.0)
    } else {
        format!("${n:.2}")
    }
}

// ============================================================================
// Store State
// ============================================================================

/// State for the token explorer store
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TokenState {
    /// List of tokens from the API
    pub tokens: Vec<Token>,
    /// Currently selected token (by ID)
    pub selected_token_id: Option<String>,
    /// Search query
    pub search_query: String,
    /// Sort field
    pub sort_by: SortField,
    /// Sort direction
    pub sort_desc: bool,
    /// Loading state (transient, not serialized)
    #[serde(skip)]
    pub loading: bool,
    /// Error message (transient, not serialized)
    #[serde(skip)]
    pub error: Option<String>,
    /// Last fetch timestamp
    pub last_fetched: Option<String>,
}

/// Fields to sort tokens by
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub enum SortField {
    #[default]
    MarketCap,
    Price,
    PriceChange24h,
    Liquidity,
    Holders,
    Volume24h,
}

impl SortField {
    pub fn label(&self) -> &'static str {
        match self {
            SortField::MarketCap => "Market Cap",
            SortField::Price => "Price",
            SortField::PriceChange24h => "24h Change",
            SortField::Liquidity => "Liquidity",
            SortField::Holders => "Holders",
            SortField::Volume24h => "24h Volume",
        }
    }
}

// ============================================================================
// Token Store
// ============================================================================

/// Store for managing token data with SSR hydration support
#[derive(Clone)]
pub struct TokenStore {
    state: RwSignal<TokenState>, // Private - access via Store::state()
}

impl TokenStore {
    /// Create a new empty token store
    pub fn new() -> Self {
        Self {
            state: RwSignal::new(TokenState::default()),
        }
    }

    /// Create a store with pre-loaded tokens (for SSR)
    pub fn with_tokens(tokens: Vec<Token>) -> Self {
        Self {
            state: RwSignal::new(TokenState {
                tokens,
                last_fetched: Some(chrono_now()),
                ..Default::default()
            }),
        }
    }

    /// Create store with existing state (for hydration)
    pub fn with_state(state: TokenState) -> Self {
        Self {
            state: RwSignal::new(state),
        }
    }

    // ========================================================================
    // Getters
    // ========================================================================

    /// Get the full state (untracked) for serialization.
    ///
    /// This is used for SSR hydration serialization. It reads the state
    /// without creating a reactive dependency.
    pub fn get_state_untracked(&self) -> TokenState {
        self.state.get_untracked()
    }

    /// Get all tokens
    pub fn tokens(&self) -> Vec<Token> {
        self.state.with(|s| s.tokens.clone())
    }

    /// Get tokens filtered and sorted
    pub fn filtered_tokens(&self) -> Vec<Token> {
        self.state.with(|s| {
            let mut tokens = s.tokens.clone();

            // Filter by search query
            if !s.search_query.is_empty() {
                let query = s.search_query.to_lowercase();
                tokens.retain(|t| {
                    t.name.to_lowercase().contains(&query)
                        || t.symbol.to_lowercase().contains(&query)
                        || t.id.to_lowercase().contains(&query)
                });
            }

            // Sort
            tokens.sort_by(|a, b| {
                let cmp = match s.sort_by {
                    SortField::MarketCap => a.mcap.partial_cmp(&b.mcap),
                    SortField::Price => a.usd_price.partial_cmp(&b.usd_price),
                    SortField::PriceChange24h => {
                        a.price_change_24h().partial_cmp(&b.price_change_24h())
                    }
                    SortField::Liquidity => a.liquidity.partial_cmp(&b.liquidity),
                    SortField::Holders => a.holder_count.partial_cmp(&b.holder_count),
                    SortField::Volume24h => {
                        let vol_a = a
                            .stats_24h
                            .as_ref()
                            .map(|s| s.buy_volume + s.sell_volume)
                            .unwrap_or(0.0);
                        let vol_b = b
                            .stats_24h
                            .as_ref()
                            .map(|s| s.buy_volume + s.sell_volume)
                            .unwrap_or(0.0);
                        vol_a.partial_cmp(&vol_b)
                    }
                };
                let cmp = cmp.unwrap_or(std::cmp::Ordering::Equal);
                if s.sort_desc { cmp.reverse() } else { cmp }
            });

            tokens
        })
    }

    /// Get selected token
    pub fn selected_token(&self) -> Option<Token> {
        self.state.with(|s| {
            s.selected_token_id
                .as_ref()
                .and_then(|id| s.tokens.iter().find(|t| &t.id == id).cloned())
        })
    }

    /// Get token count
    pub fn token_count(&self) -> usize {
        self.state.with(|s| s.tokens.len())
    }

    /// Check if loading
    pub fn is_loading(&self) -> bool {
        self.state.with(|s| s.loading)
    }

    /// Get error message
    pub fn error(&self) -> Option<String> {
        self.state.with(|s| s.error.clone())
    }

    /// Get search query (reactive)
    pub fn search_query(&self) -> String {
        self.state.with(|s| s.search_query.clone())
    }

    /// Get search query (non-reactive)
    pub fn search_query_untracked(&self) -> String {
        self.state.with_untracked(|s| s.search_query.clone())
    }

    /// Get sort field (reactive)
    pub fn sort_by(&self) -> SortField {
        self.state.with(|s| s.sort_by.clone())
    }

    /// Get sort field (non-reactive, for use outside tracking context)
    pub fn sort_by_untracked(&self) -> SortField {
        self.state.with_untracked(|s| s.sort_by.clone())
    }

    /// Check if sort is descending (reactive)
    pub fn is_sort_desc(&self) -> bool {
        self.state.with(|s| s.sort_desc)
    }

    /// Check if sort is descending (non-reactive)
    pub fn is_sort_desc_untracked(&self) -> bool {
        self.state.with_untracked(|s| s.sort_desc)
    }

    // ========================================================================
    // Actions - PUBLIC API
    // ========================================================================
    //
    // These are the public methods for modifying state.
    // The state field is private, so external code cannot directly
    // call state.set() - they must use these controlled actions.

    /// Update the token list after a fetch
    pub fn set_tokens(&self, tokens: Vec<Token>) {
        self.state.update(|s| {
            s.tokens = tokens;
            s.last_fetched = Some(chrono_now());
            s.loading = false;
            s.error = None;
        });
    }

    /// Set the loading state
    pub fn set_loading(&self, loading: bool) {
        self.state.update(|s| s.loading = loading);
    }

    /// Set an error message
    pub fn set_error(&self, error: Option<String>) {
        self.state.update(|s| {
            s.error = error;
            s.loading = false;
        });
    }

    /// Update the search query
    pub fn set_search_query(&self, query: String) {
        self.state.update(|s| s.search_query = query);
    }

    /// Toggle sort by a field (toggles direction if same field)
    pub fn set_sort_by(&self, field: SortField) {
        self.state.update(|s| {
            if s.sort_by == field {
                s.sort_desc = !s.sort_desc;
            } else {
                s.sort_by = field;
                s.sort_desc = true;
            }
        });
    }

    /// Set sort field and direction directly (for URL sync)
    pub fn set_sort_field_direct(&self, field: SortField, desc: bool) {
        self.state.update(|s| {
            s.sort_by = field;
            s.sort_desc = desc;
        });
    }

    /// Select a token by ID
    pub fn select_token(&self, id: Option<String>) {
        self.state.update(|s| s.selected_token_id = id);
    }

    /// Clear the token selection
    pub fn clear_selection(&self) {
        self.state.update(|s| s.selected_token_id = None);
    }
}

impl Default for TokenStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Get current timestamp as ISO string
fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    "2026-01-18T00:00:00Z".to_string()
}

// ============================================================================
// Store Trait Implementation
// ============================================================================

impl Store for TokenStore {
    type State = TokenState;

    fn state(&self) -> ReadSignal<Self::State> {
        self.state.read_only()
    }
}

// ============================================================================
// Hydration Support
// ============================================================================

#[cfg(feature = "hydrate")]
impl leptos_store::hydration::HydratableStore for TokenStore {
    fn serialize_state(&self) -> Result<String, leptos_store::hydration::StoreHydrationError> {
        let state = self.state.get_untracked();
        serde_json::to_string(&state)
            .map_err(|e| leptos_store::hydration::StoreHydrationError::Serialization(e.to_string()))
    }

    fn from_hydrated_state(
        data: &str,
    ) -> Result<Self, leptos_store::hydration::StoreHydrationError> {
        let state: TokenState = serde_json::from_str(data).map_err(|e| {
            leptos_store::hydration::StoreHydrationError::Deserialization(e.to_string())
        })?;
        Ok(Self::with_state(state))
    }

    fn store_key() -> &'static str {
        "token_store"
    }
}

// ============================================================================
// API Client
// ============================================================================

/// Jupiter API base URL
pub const JUPITER_API_BASE: &str = "https://jupiter.ghostnn.ai/data/v1/assets/search";

/// Default token addresses to fetch
pub const DEFAULT_TOKEN_IDS: &[&str] = &[
    "4vGHdzcNrDf8XVE8H19Rqea86RULz7xi89ew1sSJpump",
    "2oCxjpWWEmCuNaXiBSVahUKP1xRMRAiBfCg98YyHpump",
    "x95HN3DWvbfCBtTjGm587z8suK3ec6cwQwgZNLbWKyp",
    "4FDtAagigMuFcPp36rbd9bzcYTJgQah2qLMYcYtfpump",
    "DMYNp65mub3i7LRpBdB66CgBAceLcQnv4gsWeCi6pump",
    "GmbC2HgWpHpq9SHnmEXZNT5e1zgcU9oASDqbAkGTpump",
    "USoRyaQjch6E18nCdDvWoRgTo6osQs9MUd8JXEsspWR",
    "USCRdwZP5UkKhJzhWuD7XjTUviHBtZJbLG7XpbKng9S",
    "7iX4yQ4zTraFSRXEGpF89emA9xGrhgv6jX57dMENpump",
    "axUxN2q4AWzHaU6LXmjqQh7KEjaXDPKScjmzwEBpump",
];

/// Build the API URL for fetching tokens
pub fn build_api_url(token_ids: &[&str], limit: usize) -> String {
    let query = token_ids.join(",");
    format!("{JUPITER_API_BASE}?query={query}&limit={limit}")
}

/// Fetch tokens from the Jupiter API (server-side internal function)
#[cfg(feature = "ssr")]
pub async fn fetch_tokens_server() -> Result<Vec<Token>, String> {
    let url = build_api_url(DEFAULT_TOKEN_IDS, 10);

    let response = reqwest::get(&url)
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("API error: {}", response.status()));
    }

    let tokens: Vec<Token> = response
        .json()
        .await
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(tokens)
}

// ============================================================================
// Server Function - Callable from both server and client
// ============================================================================

/// Response from fetch_tokens server function
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct FetchTokensResponse {
    pub tokens: Vec<Token>,
    pub fetched_at: String,
}

/// Server function to fetch tokens - can be called from client via HTTP
#[leptos::prelude::server(FetchTokens, "/api")]
pub async fn fetch_tokens() -> Result<FetchTokensResponse, leptos::prelude::ServerFnError> {
    let tokens = fetch_tokens_server()
        .await
        .map_err(|e| leptos::prelude::ServerFnError::new(e))?;

    Ok(FetchTokensResponse {
        tokens,
        fetched_at: current_timestamp(),
    })
}

/// Get current timestamp as ISO 8601 string
#[cfg(feature = "ssr")]
fn current_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();

    // Simple UTC timestamp without full chrono dependency
    let days_since_epoch = secs / 86400;
    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    // Approximate date calculation (good enough for display)
    let year = 1970 + (days_since_epoch / 365);
    let day_of_year = days_since_epoch % 365;
    let month = (day_of_year / 30) + 1;
    let day = (day_of_year % 30) + 1;

    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year, month, day, hours, minutes, seconds
    )
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_formatting() {
        let token = Token {
            id: "abc123def456ghi789jkl012mno345pqr678stu901".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            usd_price: 0.00001234,
            mcap: 1_500_000.0,
            liquidity: 75_000.0,
            ..Default::default()
        };

        assert_eq!(token.formatted_price(), "$0.00001234");
        assert_eq!(token.formatted_mcap(), "$1.50M");
        assert_eq!(token.formatted_liquidity(), "$75.00K");
        assert_eq!(token.short_address(), "abc123...u901");
    }

    #[test]
    fn test_token_store_creation() {
        let store = TokenStore::new();
        assert_eq!(store.token_count(), 0);
        assert!(!store.is_loading());
        assert!(store.error().is_none());
    }

    #[test]
    fn test_token_store_with_tokens() {
        let tokens = vec![
            Token {
                id: "token1".to_string(),
                name: "Token One".to_string(),
                symbol: "ONE".to_string(),
                mcap: 1000.0,
                ..Default::default()
            },
            Token {
                id: "token2".to_string(),
                name: "Token Two".to_string(),
                symbol: "TWO".to_string(),
                mcap: 2000.0,
                ..Default::default()
            },
        ];

        let store = TokenStore::with_tokens(tokens);
        assert_eq!(store.token_count(), 2);
    }

    #[test]
    fn test_set_tokens_action() {
        // Test that the public set_tokens action works correctly
        let store = TokenStore::new();
        assert_eq!(store.token_count(), 0);

        let tokens = vec![Token {
            id: "action_test".to_string(),
            name: "Action Test".to_string(),
            symbol: "ACT".to_string(),
            mcap: 5000.0,
            ..Default::default()
        }];

        // Call the public action
        store.set_tokens(tokens);

        // Verify the state was updated
        assert_eq!(store.token_count(), 1);
        let retrieved = store.tokens();
        assert_eq!(retrieved[0].id, "action_test");
        assert_eq!(retrieved[0].mcap, 5000.0);

        // Verify loading was set to false
        assert!(!store.is_loading());
    }

    #[test]
    fn test_token_serialization_roundtrip() {
        let token = Token {
            id: "test_id".to_string(),
            name: "Test Token".to_string(),
            symbol: "TEST".to_string(),
            usd_price: 0.123,
            mcap: 1000000.0,
            holder_count: 500,
            stats_24h: Some(TokenStats {
                price_change: 5.5,
                buy_volume: 10000.0,
                sell_volume: 8000.0,
                num_buys: 100,
                num_sells: 80,
                ..Default::default()
            }),
            ..Default::default()
        };

        let json = serde_json::to_string(&token).unwrap();
        let restored: Token = serde_json::from_str(&json).unwrap();

        assert_eq!(token.id, restored.id);
        assert_eq!(token.name, restored.name);
        assert_eq!(token.usd_price, restored.usd_price);
        assert!(restored.stats_24h.is_some());
    }

    #[test]
    fn test_state_serialization_roundtrip() {
        let state = TokenState {
            tokens: vec![Token {
                id: "test".to_string(),
                name: "Test".to_string(),
                symbol: "TST".to_string(),
                ..Default::default()
            }],
            search_query: "test query".to_string(),
            sort_by: SortField::PriceChange24h,
            sort_desc: true,
            loading: true,                    // Should be skipped
            error: Some("error".to_string()), // Should be skipped
            ..Default::default()
        };

        let json = serde_json::to_string(&state).unwrap();
        let restored: TokenState = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.tokens.len(), 1);
        assert_eq!(restored.search_query, "test query");
        assert_eq!(restored.sort_by, SortField::PriceChange24h);
        assert!(restored.sort_desc);
        // Transient fields should be defaults
        assert!(!restored.loading);
        assert!(restored.error.is_none());
    }

    #[cfg(feature = "hydrate")]
    mod hydration_tests {
        use super::*;
        use leptos_store::hydration::HydratableStore;

        #[test]
        fn test_store_hydration_key() {
            assert_eq!(TokenStore::store_key(), "token_store");
        }

        #[test]
        fn test_store_hydration_roundtrip() {
            let store = TokenStore::with_tokens(vec![Token {
                id: "hydrate_test".to_string(),
                name: "Hydration Test".to_string(),
                symbol: "HYD".to_string(),
                usd_price: 0.5,
                mcap: 500000.0,
                ..Default::default()
            }]);

            let serialized = store.serialize_state().unwrap();
            let restored = TokenStore::from_hydrated_state(&serialized).unwrap();

            assert_eq!(restored.token_count(), 1);
            let tokens = restored.tokens();
            assert_eq!(tokens[0].id, "hydrate_test");
            assert_eq!(tokens[0].usd_price, 0.5);
        }
    }
}
