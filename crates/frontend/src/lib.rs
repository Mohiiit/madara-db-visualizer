use leptos::prelude::*;
use visualizer_types::{HealthResponse, StatsResponse};

async fn fetch_health() -> Result<HealthResponse, String> {
    let response = gloo_net::http::Request::get("http://localhost:3000/api/health")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<HealthResponse>()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_stats() -> Result<StatsResponse, String> {
    let response = gloo_net::http::Request::get("http://localhost:3000/api/stats")
        .send()
        .await
        .map_err(|e| e.to_string())?;

    response
        .json::<StatsResponse>()
        .await
        .map_err(|e| e.to_string())
}

#[component]
fn App() -> impl IntoView {
    let health = LocalResource::new(|| fetch_health());
    let stats = LocalResource::new(|| fetch_stats());

    view! {
        <div class="min-h-screen bg-gray-900 text-white">
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <h1 class="text-2xl font-bold">"Madara DB Visualizer"</h1>
            </header>
            <main class="p-6">
                <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
                    // Status card
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h2 class="text-xl font-semibold mb-4">"Status"</h2>
                        <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                            {move || {
                                health.get().map(|result| {
                                    match &*result {
                                        Ok(h) => view! {
                                            <p class="text-green-400">
                                                "API Status: " {h.status.clone()}
                                            </p>
                                        }.into_any(),
                                        Err(e) => view! {
                                            <p class="text-red-400">
                                                "Error: " {e.clone()}
                                            </p>
                                        }.into_any(),
                                    }
                                })
                            }}
                        </Suspense>
                    </div>

                    // Stats card
                    <div class="bg-gray-800 rounded-lg p-6">
                        <h2 class="text-xl font-semibold mb-4">"Database Stats"</h2>
                        <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                            {move || {
                                stats.get().map(|result| {
                                    match &*result {
                                        Ok(s) => {
                                            let block_display = match s.latest_block {
                                                Some(n) => format!("#{}", n),
                                                None => "Unknown".to_string(),
                                            };
                                            view! {
                                                <div class="space-y-2">
                                                    <p>
                                                        <span class="text-gray-400">"Database: "</span>
                                                        <span class="text-white font-mono text-sm">{s.db_path.clone()}</span>
                                                    </p>
                                                    <p>
                                                        <span class="text-gray-400">"Latest Block: "</span>
                                                        <span class="text-blue-400 font-semibold">{block_display}</span>
                                                    </p>
                                                    <p>
                                                        <span class="text-gray-400">"Column Families: "</span>
                                                        <span class="text-purple-400 font-semibold">{s.column_count}</span>
                                                    </p>
                                                </div>
                                            }.into_any()
                                        },
                                        Err(e) => view! {
                                            <p class="text-red-400">
                                                "Error: " {e.clone()}
                                            </p>
                                        }.into_any(),
                                    }
                                })
                            }}
                        </Suspense>
                    </div>
                </div>

                // Columns list
                <div class="mt-6 bg-gray-800 rounded-lg p-6">
                    <h2 class="text-xl font-semibold mb-4">"Column Families"</h2>
                    <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                        {move || {
                            stats.get().map(|result| {
                                match &*result {
                                    Ok(s) => {
                                        let columns = s.columns.clone();
                                        view! {
                                            <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-2">
                                                {columns.into_iter().map(|col| {
                                                    let is_bonsai = col.starts_with("bonsai");
                                                    let bg_class = if is_bonsai {
                                                        "bg-purple-900/50 border-purple-700"
                                                    } else {
                                                        "bg-gray-700 border-gray-600"
                                                    };
                                                    view! {
                                                        <div class={format!("px-3 py-2 rounded border text-sm font-mono {}", bg_class)}>
                                                            {col}
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    },
                                    Err(_) => view! {
                                        <p class="text-gray-400">"Could not load columns"</p>
                                    }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </main>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
