// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Server entry point for the Token Explorer Example
//!
//! Fetches token data from Jupiter API on each request and renders with SSR.
//! Client-side polling updates the data every 30 seconds.

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::*;
    use leptos_actix::{LeptosRoutes, generate_route_list};
    use token_explorer_example::{components::App, token_store::*};

    // Set defaults for manual mode
    if std::env::var("LEPTOS_OUTPUT_NAME").is_err() {
        unsafe {
            std::env::set_var("LEPTOS_OUTPUT_NAME", "token-explorer-example");
            std::env::set_var("LEPTOS_SITE_ROOT", "target/site");
            std::env::set_var("LEPTOS_SITE_PKG_DIR", "pkg");
            std::env::set_var("LEPTOS_SITE_ADDR", "127.0.0.1:3005");
        }
    }

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    println!("🪙 Token Explorer Example (SSR Mode)");
    println!("   Listening on http://{}", addr);
    println!("   Tokens are fetched fresh on each request");
    println!("   Client polls every 30 seconds for updates");

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone();

        let routes = generate_route_list(App);

        actix_web::App::new()
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    // Note: We use a resource to fetch tokens per-request
                    // The actual fetching happens in the App component via create_resource

                    // Create an empty store - it will be populated by the resource
                    let store = TokenStore::new();

                    // Serialize empty store state for hydration
                    // The client will immediately fetch fresh data
                    let hydration_data = {
                        let json = serde_json::to_string(&store.get_state_untracked())
                            .unwrap_or_default();
                        let escaped = json.replace("</script>", "<\\/script>");
                        format!(
                            r#"<script id="__leptos_store_token_store" type="application/json">{}</script>"#,
                            escaped
                        )
                    };

                    // Provide store to context
                    leptos_store::context::provide_store(store);

                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone() />
                                <leptos_meta::MetaTags/>
                            </head>
                            <body>
                                <App/>
                                <div inner_html=hydration_data />
                            </body>
                        </html>
                    }
                }
            })
            .app_data(web::Data::new(leptos_options.clone()))
    })
    .bind(&addr)?
    .run()
    .await
}

#[cfg(not(feature = "ssr"))]
fn main() {
    eprintln!("This binary requires the `ssr` feature. Run with:");
    eprintln!("  cargo leptos watch");
}
