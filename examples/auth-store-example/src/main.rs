// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Server entry point for the Auth Store Example
//!
//! This runs the SSR server with Actix Web.

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use auth_store_example::{AuthStore, components::App};
    use leptos::prelude::*;
    use leptos_actix::{LeptosRoutes, generate_route_list};
    use leptos_store::context::provide_store;

    // Set defaults for manual mode if env vars aren't set (cargo-leptos sets these)
    // SAFETY: We're single-threaded at this point before any async work starts
    if std::env::var("LEPTOS_OUTPUT_NAME").is_err() {
        unsafe {
            std::env::set_var("LEPTOS_OUTPUT_NAME", "auth-store-example");
            std::env::set_var("LEPTOS_SITE_ROOT", "target/site");
            std::env::set_var("LEPTOS_SITE_PKG_DIR", "pkg");
            std::env::set_var("LEPTOS_SITE_ADDR", "127.0.0.1:3000");
        }
    }

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    println!("🚀 Auth Store Example (SSR Mode)");
    println!("   Listening on http://{}", addr);
    println!("   Site root: {}", conf.leptos_options.site_root);
    println!("   Output name: {}", conf.leptos_options.output_name);

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone();

        // Generate the list of routes
        let routes = generate_route_list(App);

        actix_web::App::new()
            // Serve static files from the `pkg` directory
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            // Leptos routes
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    // Create store for this request
                    let store = AuthStore::new();

                    // Serialize store state for hydration
                    let hydration_data = {
                        let json = serde_json::to_string(&store.get_state_untracked())
                            .unwrap_or_default();
                        let escaped = json.replace("</script>", "<\\/script>");
                        format!(
                            r#"<script id="__leptos_store_auth_store" type="application/json">{}</script>"#,
                            escaped
                        )
                    };

                    // Provide store to context
                    provide_store(store);

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
    // This binary requires the `ssr` feature
    eprintln!("This binary requires the `ssr` feature. Run with:");
    eprintln!("  cargo run --features ssr");
}
