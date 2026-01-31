use leptos::prelude::*;
use visualizer_types::{BlockDetail, BlockListResponse, BlockSummary, HealthResponse, StatsResponse};

const API_BASE: &str = "http://localhost:3000";

async fn fetch_health() -> Result<HealthResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/health"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_stats() -> Result<StatsResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/stats"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_blocks(offset: u64, limit: u64) -> Result<BlockListResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/blocks?offset={offset}&limit={limit}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_block(block_number: u64) -> Result<BlockDetail, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/blocks/{block_number}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

fn format_timestamp(ts: u64) -> String {
    // Simple timestamp formatting
    let secs = ts % 60;
    let mins = (ts / 60) % 60;
    let hours = (ts / 3600) % 24;
    format!("{:02}:{:02}:{:02}", hours, mins, secs)
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..10], &hash[hash.len()-6..])
    } else {
        hash.to_string()
    }
}

#[component]
fn BlockRow(block: BlockSummary, on_click: impl Fn(u64) + 'static) -> impl IntoView {
    let block_number = block.block_number;
    view! {
        <tr
            class="border-b border-gray-700 hover:bg-gray-700 cursor-pointer"
            on:click=move |_| on_click(block_number)
        >
            <td class="px-4 py-3 text-blue-400 font-mono">{"#"}{block.block_number}</td>
            <td class="px-4 py-3 font-mono text-sm text-gray-300">{truncate_hash(&block.block_hash)}</td>
            <td class="px-4 py-3 text-center">{block.transaction_count}</td>
        </tr>
    }
}

#[component]
fn BlockList(on_select: impl Fn(u64) + Clone + Send + 'static) -> impl IntoView {
    let (offset, set_offset) = signal(0u64);
    let limit = 20u64;

    let blocks = LocalResource::new(move || {
        let offset = offset.get();
        async move { fetch_blocks(offset, limit).await }
    });

    view! {
        <div class="bg-gray-800 rounded-lg p-4">
            <h2 class="text-xl font-semibold mb-4">"Blocks"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading blocks..."</p> }>
                {move || {
                    let on_select = on_select.clone();
                    blocks.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                // Clone all data we need
                                let blocks_data = data.blocks.clone();
                                let total = data.total;
                                let has_prev = offset.get() > 0;
                                let has_next = offset.get() + limit < total;

                                view! {
                                    <div>
                                        <table class="w-full text-left">
                                            <thead class="text-gray-400 text-sm">
                                                <tr>
                                                    <th class="px-4 py-2">"Block"</th>
                                                    <th class="px-4 py-2">"Hash"</th>
                                                    <th class="px-4 py-2 text-center">"Txns"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {blocks_data.into_iter().map(|block| {
                                                    let on_select = on_select.clone();
                                                    view! { <BlockRow block=block on_click=move |n| on_select(n) /> }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                        <div class="flex justify-between mt-4 px-4">
                                            <button
                                                class="px-4 py-2 bg-gray-700 rounded disabled:opacity-50"
                                                disabled=move || !has_prev
                                                on:click=move |_| set_offset.update(|o| *o = o.saturating_sub(limit))
                                            >
                                                "Previous"
                                            </button>
                                            <span class="text-gray-400">
                                                {move || offset.get() + 1}"-"{move || (offset.get() + limit).min(total)}" of "{total}
                                            </span>
                                            <button
                                                class="px-4 py-2 bg-gray-700 rounded disabled:opacity-50"
                                                disabled=move || !has_next
                                                on:click=move |_| set_offset.update(|o| *o += limit)
                                            >
                                                "Next"
                                            </button>
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <p class="text-red-400">"Error: " {e.clone()}</p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn BlockDetailView(block_number: u64, on_back: impl Fn() + 'static) -> impl IntoView {
    let block = LocalResource::new(move || async move { fetch_block(block_number).await });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to blocks"
            </button>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading block..."</p> }>
                {move || {
                    block.get().map(|result| {
                        match result.as_ref() {
                            Ok(b) => {
                                // Clone all data we need
                                let block_num = b.block_number;
                                let block_hash = b.block_hash.clone();
                                let parent_hash = b.parent_hash.clone();
                                let state_root = b.state_root.clone();
                                let sequencer = b.sequencer_address.clone();
                                let tx_count = b.transaction_count;
                                let event_count = b.event_count;
                                let gas_used = b.l2_gas_used;
                                let tx_hashes = b.tx_hashes.clone();

                                view! {
                                    <div>
                                        <h2 class="text-2xl font-bold mb-4">"Block #"{block_num}</h2>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div>
                                                <p class="text-gray-400">"Block Hash"</p>
                                                <p class="font-mono text-sm break-all">{block_hash}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Parent Hash"</p>
                                                <p class="font-mono text-sm break-all">{parent_hash}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"State Root"</p>
                                                <p class="font-mono text-sm break-all">{state_root}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Sequencer"</p>
                                                <p class="font-mono text-sm">{sequencer}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Transactions"</p>
                                                <p class="text-blue-400 font-semibold">{tx_count}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Events"</p>
                                                <p class="text-purple-400 font-semibold">{event_count}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"L2 Gas Used"</p>
                                                <p>{gas_used}</p>
                                            </div>
                                        </div>
                                        <div class="mt-6">
                                            <h3 class="text-lg font-semibold mb-2">"Transaction Hashes"</h3>
                                            <div class="space-y-1">
                                                {tx_hashes.into_iter().enumerate().map(|(i, hash)| {
                                                    view! {
                                                        <p class="font-mono text-sm text-gray-300">
                                                            <span class="text-gray-500">{i + 1}". "</span>
                                                            {hash}
                                                        </p>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <p class="text-red-400">"Error: " {e.clone()}</p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn StatsCard() -> impl IntoView {
    let stats = LocalResource::new(|| fetch_stats());

    view! {
        <div class="bg-gray-800 rounded-lg p-4">
            <h2 class="text-lg font-semibold mb-3">"Database Stats"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || {
                    stats.get().map(|result| {
                        match result.as_ref() {
                            Ok(s) => {
                                let latest = s.latest_block.unwrap_or(0);
                                let cols = s.column_count;
                                view! {
                                    <div class="space-y-2 text-sm">
                                        <p>
                                            <span class="text-gray-400">"Latest Block: "</span>
                                            <span class="text-blue-400 font-semibold">
                                                {"#"}{latest}
                                            </span>
                                        </p>
                                        <p>
                                            <span class="text-gray-400">"Columns: "</span>
                                            <span class="text-purple-400">{cols}</span>
                                        </p>
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <p class="text-red-400 text-sm">"Error: " {e.clone()}</p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (selected_block, set_selected_block) = signal::<Option<u64>>(None);

    view! {
        <div class="min-h-screen bg-gray-900 text-white">
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4">
                <h1 class="text-2xl font-bold">"Madara DB Visualizer"</h1>
            </header>

            <div class="flex">
                // Sidebar
                <aside class="w-64 bg-gray-800 border-r border-gray-700 p-4 min-h-screen">
                    <StatsCard />
                </aside>

                // Main content
                <main class="flex-1 p-6">
                    {move || {
                        match selected_block.get() {
                            Some(block_n) => view! {
                                <BlockDetailView
                                    block_number=block_n
                                    on_back=move || set_selected_block.set(None)
                                />
                            }.into_any(),
                            None => view! {
                                <BlockList on_select=move |n| set_selected_block.set(Some(n)) />
                            }.into_any(),
                        }
                    }}
                </main>
            </div>
        </div>
    }
}

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn main() {
    console_error_panic_hook::set_once();
    leptos::mount::mount_to_body(App);
}
