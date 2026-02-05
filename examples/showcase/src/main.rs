// SPDX-License-Identifier: Apache-2.0
// Copyright 2026 web-mech

//! Server entry point for the Leptos Store Examples Showcase

#[cfg(feature = "ssr")]
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    use actix_files::Files;
    use actix_web::*;
    use leptos::prelude::*;
    use leptos_actix::{LeptosRoutes, generate_route_list};
    use showcase::components::App;

    if std::env::var("LEPTOS_OUTPUT_NAME").is_err() {
        unsafe {
            std::env::set_var("LEPTOS_OUTPUT_NAME", "showcase");
            std::env::set_var("LEPTOS_SITE_ROOT", "target/site");
            std::env::set_var("LEPTOS_SITE_PKG_DIR", "pkg");
            std::env::set_var("LEPTOS_SITE_ADDR", "127.0.0.1:3100");
        }
    }

    let conf = get_configuration(None).expect("Failed to load Leptos configuration");
    let addr = conf.leptos_options.site_addr;

    println!("╔══════════════════════════════════════════════════════════════╗");
    println!("║         Leptos Store - Examples Showcase                     ║");
    println!("╠══════════════════════════════════════════════════════════════╣");
    println!("║  Open http://{} in your browser              ║", addr);
    println!("╚══════════════════════════════════════════════════════════════╝");

    HttpServer::new(move || {
        let leptos_options = &conf.leptos_options;
        let site_root = leptos_options.site_root.clone();
        let routes = generate_route_list(App);

        App::new()
            .service(Files::new("/pkg", format!("{site_root}/pkg")))
            .leptos_routes(routes, {
                let leptos_options = leptos_options.clone();
                move || {
                    view! {
                        <!DOCTYPE html>
                        <html lang="en">
                            <head>
                                <meta charset="utf-8"/>
                                <meta name="viewport" content="width=device-width, initial-scale=1"/>
                                <link rel="preconnect" href="https://fonts.googleapis.com"/>
                                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="anonymous"/>
                                <link href="https://fonts.googleapis.com/css2?family=Inter:wght@400;500;600;700&family=JetBrains+Mono:wght@400;500&display=swap" rel="stylesheet"/>
                                <AutoReload options=leptos_options.clone() />
                                <HydrationScripts options=leptos_options.clone() />
                                <leptos_meta::MetaTags/>
                            </head>
                            <body>
                                <App/>
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
    eprintln!("  cargo run --features ssr");
}
