use leptos::prelude::*;
use std::sync::OnceLock;
use visualizer_types::{
    BlockDetail, BlockListResponse, BlockSummary, ClassListResponse, ClassResponse,
    ColumnFamilyInfo, ColumnFamilyListResponse, ColumnFamilySchemaInfo, ContractListResponse,
    ContractResponse, ContractStorageResponse, FilteredTransactionsResponse, IndexStatusResponse,
    IndexedTransactionInfo, KeyListResponse, QueryRequest, QueryResult, RawKeyValueResponse,
    SchemaCategoriesResponse, SchemaCategoryInfo, SchemaColumnFamiliesResponse, SearchResponse,
    StateDiffResponse, StatsResponse, TableInfo, TableListResponse, TransactionDetail,
    TransactionListResponse, TransactionSummary,
};
use wasm_bindgen::prelude::*;

// Default API URL for deployed version (Render.com free tier)
const DEFAULT_API_URL: &str = "https://madara-db-visualizer.onrender.com";

static API_BASE: OnceLock<String> = OnceLock::new();

/// Get the API base URL, checking query params and localStorage first
fn get_api_base() -> &'static str {
    API_BASE.get_or_init(|| {
        if let Some(window) = web_sys::window() {
            // Check URL query param: ?api=https://...
            if let Ok(search) = window.location().search() {
                for param in search.trim_start_matches('?').split('&') {
                    if let Some(url) = param.strip_prefix("api=") {
                        if !url.is_empty() {
                            // Save to localStorage
                            if let Ok(Some(storage)) = window.local_storage() {
                                let _ = storage.set_item("api_url", url);
                            }
                            return url.to_string();
                        }
                    }
                }
            }
            // Check localStorage
            if let Ok(Some(storage)) = window.local_storage() {
                if let Ok(Some(url)) = storage.get_item("api_url") {
                    if !url.is_empty() {
                        return url;
                    }
                }
            }
            // Check if running locally
            if let Ok(host) = window.location().hostname() {
                if host == "localhost" || host == "127.0.0.1" {
                    return "".to_string();
                }
            }
        }
        DEFAULT_API_URL.to_string()
    })
}

fn download_json(data: &str, filename: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Ok(a) = document.create_element("a") {
                let blob_options = web_sys::BlobPropertyBag::new();
                blob_options.set_type("application/json");
                if let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(
                    &js_sys::Array::of1(&JsValue::from_str(data)),
                    &blob_options,
                ) {
                    if let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) {
                        let _ = a.set_attribute("href", &url);
                        let _ = a.set_attribute("download", filename);
                        if let Ok(a) = a.dyn_into::<web_sys::HtmlElement>() {
                            a.click();
                        }
                        let _ = web_sys::Url::revoke_object_url(&url);
                    }
                }
            }
        }
    }
}

fn copy_to_clipboard(text: &str) {
    if let Some(window) = web_sys::window() {
        let clipboard = window.navigator().clipboard();
        let _ = clipboard.write_text(text);
    }
}

async fn fetch_stats() -> Result<StatsResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/stats", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_blocks(offset: u64, limit: u64) -> Result<BlockListResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/blocks?offset={offset}&limit={limit}",
        get_api_base()
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_block(block_number: u64) -> Result<BlockDetail, String> {
    gloo_net::http::Request::get(&format!("{}/api/blocks/{block_number}", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_block_transactions(block_number: u64) -> Result<TransactionListResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/blocks/{block_number}/transactions",
        get_api_base()
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_transaction(
    block_number: u64,
    tx_index: usize,
) -> Result<TransactionDetail, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/blocks/{block_number}/transactions/{tx_index}",
        get_api_base()
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_contracts(limit: usize) -> Result<ContractListResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/contracts?limit={limit}", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_contract(address: String) -> Result<ContractResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/contracts/{address}", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_contract_storage(
    address: String,
    limit: usize,
) -> Result<ContractStorageResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/contracts/{address}/storage?limit={limit}",
        get_api_base()
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_classes(limit: usize) -> Result<ClassListResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/classes?limit={limit}", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_class(class_hash: String) -> Result<ClassResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/classes/{class_hash}", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_state_diff(block_number: u64) -> Result<StateDiffResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/blocks/{block_number}/state-diff",
        get_api_base()
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_search(query: String) -> Result<SearchResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/search?q={}",
        get_api_base(),
        urlencoding::encode(&query)
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

async fn fetch_index_status() -> Result<IndexStatusResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/index/status", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_filtered_transactions(
    status: Option<String>,
    block_from: Option<u64>,
    block_to: Option<u64>,
    limit: usize,
) -> Result<FilteredTransactionsResponse, String> {
    let mut url = format!("{}/api/index/transactions?limit={limit}", get_api_base());
    if let Some(s) = status {
        url.push_str(&format!("&status={}", urlencoding::encode(&s)));
    }
    if let Some(from) = block_from {
        url.push_str(&format!("&block_from={}", from));
    }
    if let Some(to) = block_to {
        url.push_str(&format!("&block_to={}", to));
    }
    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

// Schema API functions

async fn fetch_cf_schema(cf_name: &str) -> Result<ColumnFamilySchemaInfo, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/schema/column-families/{}",
        get_api_base(),
        urlencoding::encode(cf_name)
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

// Raw data API functions

async fn fetch_column_families() -> Result<ColumnFamilyListResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/raw/cf", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_cf_keys(
    cf_name: &str,
    limit: usize,
    offset: usize,
    prefix: Option<String>,
) -> Result<KeyListResponse, String> {
    let mut url = format!(
        "{}/api/raw/cf/{}/keys?limit={}&offset={}",
        get_api_base(),
        urlencoding::encode(cf_name),
        limit,
        offset
    );
    if let Some(p) = prefix {
        if !p.is_empty() {
            url.push_str(&format!("&prefix={}", urlencoding::encode(&p)));
        }
    }
    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_raw_key_value(cf_name: &str, key_hex: &str) -> Result<RawKeyValueResponse, String> {
    gloo_net::http::Request::get(&format!(
        "{}/api/raw/cf/{}/key/{}",
        get_api_base(),
        urlencoding::encode(cf_name),
        urlencoding::encode(key_hex)
    ))
    .send()
    .await
    .map_err(|e| e.to_string())?
    .json()
    .await
    .map_err(|e| e.to_string())
}

// Schema API functions

async fn fetch_schema_categories() -> Result<SchemaCategoriesResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/schema/categories", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_schema_column_families(
    category: Option<&str>,
) -> Result<SchemaColumnFamiliesResponse, String> {
    let url = match category {
        Some(cat) => format!(
            "{}/api/schema/column-families?category={}",
            get_api_base(),
            urlencoding::encode(cat)
        ),
        None => format!("{}/api/schema/column-families", get_api_base()),
    };
    gloo_net::http::Request::get(&url)
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

// SQL Console API functions

async fn fetch_index_tables() -> Result<TableListResponse, String> {
    gloo_net::http::Request::get(&format!("{}/api/index/tables", get_api_base()))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn execute_sql_query(sql: String, params: Vec<String>) -> Result<QueryResult, String> {
    let request = QueryRequest { sql, params };
    let response = gloo_net::http::Request::post(&format!("{}/api/index/query", get_api_base()))
        .header("Content-Type", "application/json")
        .body(serde_json::to_string(&request).map_err(|e| e.to_string())?)
        .map_err(|e| e.to_string())?
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if response.ok() {
        response.json().await.map_err(|e| e.to_string())
    } else {
        let error_text = response
            .text()
            .await
            .unwrap_or_else(|_| "Unknown error".to_string());
        Err(error_text)
    }
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..10], &hash[hash.len() - 6..])
    } else {
        hash.to_string()
    }
}

#[component]
fn CopyButton(text: String) -> impl IntoView {
    let (copied, set_copied) = signal(false);
    let text_clone = text.clone();
    view! {
        <button
            class="ml-2 px-2 py-1 text-xs bg-gray-700 hover:bg-gray-600 rounded text-gray-400 hover:text-white"
            on:click=move |_| {
                copy_to_clipboard(&text_clone);
                set_copied.set(true);
                // Reset after 2 seconds
                let set_copied = set_copied.clone();
                leptos::task::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(2000).await;
                    set_copied.set(false);
                });
            }
        >
            {move || if copied.get() { "Copied!" } else { "Copy" }}
        </button>
    }
}

#[component]
fn ExportButton(data: String, filename: String) -> impl IntoView {
    view! {
        <button
            class="px-3 py-1 text-sm bg-green-600 hover:bg-green-700 rounded text-white"
            on:click=move |_| download_json(&data, &filename)
        >
            "Export JSON"
        </button>
    }
}

/// Page state for navigation
#[derive(Clone, Debug)]
enum Page {
    BlockList,
    BlockDetail { block_number: u64 },
    TransactionDetail { block_number: u64, tx_index: usize },
    StateDiff { block_number: u64 },
    ContractList,
    ContractDetail { address: String },
    ClassList,
    ClassDetail { class_hash: String },
    AdvancedFilters,
    RawData,
    RawKeyDetail { cf_name: String, key_hex: String },
    Schema,
    SchemaDetail { cf_name: String },
    SqlConsole,
}

#[component]
fn BlockRow(block: BlockSummary, on_click: impl Fn(u64) + 'static) -> impl IntoView {
    let block_number = block.block_number;
    view! {
        <tr
            class="hover:bg-slate-800/40 cursor-pointer transition-colors"
            on:click=move |_| on_click(block_number)
        >
            <td class="px-4 py-3 text-sky-300 font-mono font-semibold">{"#"}{block.block_number}</td>
            <td class="px-4 py-3 font-mono text-xs sm:text-sm text-slate-200">{truncate_hash(&block.block_hash)}</td>
            <td class="px-4 py-3 text-center text-slate-200">{block.transaction_count}</td>
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
        <div class="space-y-4">
            <div class="flex items-end justify-between gap-4">
                <div>
                    <h2 class="text-xl font-semibold tracking-tight">"Blocks"</h2>
                    <p class="text-sm text-slate-400">"Latest blocks from the database"</p>
                </div>
            </div>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading blocks..."</p> }>
                {move || {
                    let on_select = on_select.clone();
                    blocks.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let blocks_data = data.blocks.clone();
                                let total = data.total;
                                let has_prev = offset.get() > 0;
                                let has_next = offset.get() + limit < total;

                                view! {
                                    <div>
                                        <div class="overflow-x-auto rounded-xl border border-slate-800">
                                        <table class="min-w-full text-left text-sm">
                                            <thead class="bg-slate-900/60 text-slate-300 text-xs sticky top-0 backdrop-blur">
                                                <tr>
                                                    <th class="px-4 py-3 font-semibold tracking-wider">"Block"</th>
                                                    <th class="px-4 py-3 font-semibold tracking-wider">"Hash"</th>
                                                    <th class="px-4 py-3 font-semibold tracking-wider text-center">"Txns"</th>
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-slate-800">
                                                {blocks_data.into_iter().map(|block| {
                                                    let on_select = on_select.clone();
                                                    view! { <BlockRow block=block on_click=move |n| on_select(n) /> }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                        </div>
                                        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-3 mt-4">
                                            <button
                                                class="inline-flex items-center justify-center rounded-lg bg-slate-800 px-4 py-2 text-sm text-slate-100 ring-1 ring-slate-700 hover:bg-slate-700 disabled:opacity-50 transition-colors"
                                                disabled=move || !has_prev
                                                on:click=move |_| set_offset.update(|o| *o = o.saturating_sub(limit))
                                            >
                                                "Previous"
                                            </button>
                                            <span class="text-xs text-slate-400">
                                                {move || offset.get() + 1}"-"{move || (offset.get() + limit).min(total)}" of "{total}
                                            </span>
                                            <button
                                                class="inline-flex items-center justify-center rounded-lg bg-slate-800 px-4 py-2 text-sm text-slate-100 ring-1 ring-slate-700 hover:bg-slate-700 disabled:opacity-50 transition-colors"
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
fn TransactionRow(
    tx: TransactionSummary,
    on_click: impl Fn((u64, usize)) + 'static,
) -> impl IntoView {
    let block_number = tx.block_number;
    let tx_index = tx.tx_index;
    let status_class = if tx.status == "SUCCEEDED" {
        "text-green-400"
    } else {
        "text-red-400"
    };

    view! {
        <tr
            class="border-b border-gray-700 hover:bg-gray-700 cursor-pointer"
            on:click=move |_| on_click((block_number, tx_index))
        >
            <td class="px-4 py-3 text-gray-400">{tx.tx_index}</td>
            <td class="px-4 py-3 font-mono text-sm text-blue-400">{truncate_hash(&tx.tx_hash)}</td>
            <td class="px-4 py-3">
                <span class="px-2 py-1 text-xs rounded bg-gray-700">{tx.tx_type}</span>
            </td>
            <td class={format!("px-4 py-3 {}", status_class)}>{tx.status}</td>
        </tr>
    }
}

#[component]
fn BlockDetailView(
    block_number: u64,
    on_back: impl Fn() + 'static,
    on_tx_select: impl Fn((u64, usize)) + Clone + Send + 'static,
    on_state_diff: impl Fn(u64) + Clone + Send + Sync + 'static,
) -> impl IntoView {
    let block = LocalResource::new(move || async move { fetch_block(block_number).await });
    let transactions =
        LocalResource::new(move || async move { fetch_block_transactions(block_number).await });
    let on_state_diff = std::sync::Arc::new(on_state_diff);

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
                    let on_state_diff = on_state_diff.clone();
                    block.get().map(|result| {
                        match result.as_ref() {
                            Ok(b) => {
                                let on_state_diff = on_state_diff.clone();
                                let block_num = b.block_number;
                                let block_hash = b.block_hash.clone();
                                let block_hash_copy = b.block_hash.clone();
                                let parent_hash = b.parent_hash.clone();
                                let parent_hash_copy = b.parent_hash.clone();
                                let state_root = b.state_root.clone();
                                let state_root_copy = b.state_root.clone();
                                let sequencer = b.sequencer_address.clone();
                                let sequencer_copy = b.sequencer_address.clone();
                                let tx_count = b.transaction_count;
                                let event_count = b.event_count;
                                let gas_used = b.l2_gas_used;
                                let export_data = serde_json::to_string_pretty(&*b).unwrap_or_default();
                                let export_filename = format!("block_{}.json", block_num);

                                view! {
                                    <div>
                                        <div class="flex justify-between items-center mb-4">
                                            <h2 class="text-2xl font-bold">"Block #"{block_num}</h2>
                                            <ExportButton data=export_data filename=export_filename />
                                        </div>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                            <div>
                                                <p class="text-gray-400">"Block Hash"</p>
                                                <div class="flex items-center">
                                                    <p class="font-mono text-sm break-all">{block_hash}</p>
                                                    <CopyButton text=block_hash_copy />
                                                </div>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Parent Hash"</p>
                                                <div class="flex items-center">
                                                    <p class="font-mono text-sm break-all">{parent_hash}</p>
                                                    <CopyButton text=parent_hash_copy />
                                                </div>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"State Root"</p>
                                                <div class="flex items-center">
                                                    <p class="font-mono text-sm break-all">{state_root}</p>
                                                    <CopyButton text=state_root_copy />
                                                </div>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Sequencer"</p>
                                                <div class="flex items-center">
                                                    <p class="font-mono text-sm">{sequencer}</p>
                                                    <CopyButton text=sequencer_copy />
                                                </div>
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
                                        <div class="mt-4">
                                            <button
                                                class="px-4 py-2 bg-yellow-600 hover:bg-yellow-700 rounded text-white"
                                                on:click=move |_| on_state_diff(block_num)
                                            >
                                                "View State Diff"
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

            <div class="mt-6">
                <h3 class="text-lg font-semibold mb-4">"Transactions"</h3>
                <Suspense fallback=move || view! { <p class="text-gray-400">"Loading transactions..."</p> }>
                    {move || {
                        let on_tx_select = on_tx_select.clone();
                        transactions.get().map(|result| {
                            match result.as_ref() {
                                Ok(data) => {
                                    let txs = data.transactions.clone();
                                    if txs.is_empty() {
                                        view! {
                                            <p class="text-gray-500">"No transactions in this block"</p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <table class="w-full text-left">
                                                <thead class="text-gray-400 text-sm">
                                                    <tr>
                                                        <th class="px-4 py-2">"#"</th>
                                                        <th class="px-4 py-2">"Hash"</th>
                                                        <th class="px-4 py-2">"Type"</th>
                                                        <th class="px-4 py-2">"Status"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {txs.into_iter().map(|tx| {
                                                        let on_tx_select = on_tx_select.clone();
                                                        view! { <TransactionRow tx=tx on_click=move |t| on_tx_select(t) /> }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        }.into_any()
                                    }
                                },
                                Err(e) => view! {
                                    <p class="text-red-400">"Error loading transactions: " {e.clone()}</p>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn TransactionDetailView(
    block_number: u64,
    tx_index: usize,
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let tx =
        LocalResource::new(move || async move { fetch_transaction(block_number, tx_index).await });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to block"
            </button>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading transaction..."</p> }>
                {move || {
                    tx.get().map(|result| {
                        match result.as_ref() {
                            Ok(t) => {
                                let tx_hash = t.tx_hash.clone();
                                let tx_hash_copy = t.tx_hash.clone();
                                let tx_type = t.tx_type.clone();
                                let status = t.status.clone();
                                let revert_reason = t.revert_reason.clone();
                                let block_num = t.block_number;
                                let idx = t.tx_index;
                                let actual_fee = t.actual_fee.clone();
                                let fee_unit = t.fee_unit.clone();
                                let sender = t.sender_address.clone();
                                let sender_copy = t.sender_address.clone();
                                let nonce = t.nonce.clone();
                                let version = t.version.clone();
                                let calldata = t.calldata.clone();
                                let signature = t.signature.clone();
                                let events = t.events.clone();
                                let export_data = serde_json::to_string_pretty(&*t).unwrap_or_default();
                                let export_filename = format!("tx_{}.json", truncate_hash(&t.tx_hash));

                                let status_class = if status == "SUCCEEDED" {
                                    "text-green-400"
                                } else {
                                    "text-red-400"
                                };

                                view! {
                                    <div>
                                        <div class="flex justify-between items-center mb-4">
                                            <h2 class="text-2xl font-bold">"Transaction"</h2>
                                            <ExportButton data=export_data filename=export_filename />
                                        </div>
                                        <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-6">
                                            <div class="col-span-1 md:col-span-2">
                                                <p class="text-gray-400">"Transaction Hash"</p>
                                                <div class="flex items-center flex-wrap">
                                                    <p class="font-mono text-sm break-all text-blue-400">{tx_hash}</p>
                                                    <CopyButton text=tx_hash_copy />
                                                </div>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Block"</p>
                                                <p class="text-blue-400 font-semibold">{"#"}{block_num}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Index"</p>
                                                <p>{idx}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Type"</p>
                                                <p><span class="px-2 py-1 text-sm rounded bg-gray-700">{tx_type}</span></p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Status"</p>
                                                <p class={status_class}>{status.clone()}</p>
                                            </div>
                                            {revert_reason.map(|reason| view! {
                                                <div class="col-span-2">
                                                    <p class="text-gray-400">"Revert Reason"</p>
                                                    <p class="text-red-400 font-mono text-sm">{reason}</p>
                                                </div>
                                            })}
                                            <div>
                                                <p class="text-gray-400">"Fee"</p>
                                                <p class="font-mono">{actual_fee}" "{fee_unit}</p>
                                            </div>
                                            {sender.map(|s| {
                                                let s_copy = sender_copy.clone().unwrap_or_default();
                                                view! {
                                                    <div class="col-span-1 md:col-span-2">
                                                        <p class="text-gray-400">"Sender Address"</p>
                                                        <div class="flex items-center flex-wrap">
                                                            <p class="font-mono text-sm break-all">{s}</p>
                                                            <CopyButton text=s_copy />
                                                        </div>
                                                    </div>
                                                }
                                            })}
                                            {nonce.map(|n| view! {
                                                <div>
                                                    <p class="text-gray-400">"Nonce"</p>
                                                    <p class="font-mono">{n}</p>
                                                </div>
                                            })}
                                            {version.map(|v| view! {
                                                <div>
                                                    <p class="text-gray-400">"Version"</p>
                                                    <p>{v}</p>
                                                </div>
                                            })}
                                        </div>

                                        {if !calldata.is_empty() {
                                            view! {
                                                <div class="mb-4">
                                                    <h3 class="text-lg font-semibold mb-2">"Calldata ("{calldata.len()}" items)"</h3>
                                                    <div class="bg-gray-900 rounded p-3 max-h-48 overflow-y-auto">
                                                        {calldata.into_iter().enumerate().map(|(i, item)| {
                                                            view! {
                                                                <p class="font-mono text-xs text-gray-300">
                                                                    <span class="text-gray-500">{i}": "</span>
                                                                    {item}
                                                                </p>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        {if !signature.is_empty() {
                                            view! {
                                                <div class="mb-4">
                                                    <h3 class="text-lg font-semibold mb-2">"Signature ("{signature.len()}" parts)"</h3>
                                                    <div class="bg-gray-900 rounded p-3 max-h-32 overflow-y-auto">
                                                        {signature.into_iter().enumerate().map(|(i, item)| {
                                                            view! {
                                                                <p class="font-mono text-xs text-gray-300 break-all">
                                                                    <span class="text-gray-500">{i}": "</span>
                                                                    {item}
                                                                </p>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        {if !events.is_empty() {
                                            view! {
                                                <div class="mb-4">
                                                    <h3 class="text-lg font-semibold mb-2">"Events ("{events.len()}")"</h3>
                                                    <div class="space-y-2">
                                                        {events.into_iter().enumerate().map(|(i, evt)| {
                                                            view! {
                                                                <div class="bg-gray-900 rounded p-3">
                                                                    <p class="text-sm font-semibold text-purple-400 mb-1">
                                                                        "Event "{i + 1}
                                                                    </p>
                                                                    <p class="text-xs text-gray-400">
                                                                        "From: "
                                                                        <span class="font-mono text-gray-300">{truncate_hash(&evt.from_address)}</span>
                                                                    </p>
                                                                    <p class="text-xs text-gray-400">
                                                                        "Keys: "{evt.keys.len()}" | Data: "{evt.data.len()}
                                                                    </p>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}
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

// Contract components

#[component]
fn ContractRow(contract: ContractResponse, on_click: impl Fn(String) + 'static) -> impl IntoView {
    let address = contract.address.clone();
    let address_for_click = address.clone();
    let class_hash = contract
        .class_hash
        .clone()
        .unwrap_or_else(|| "None".to_string());
    let nonce = contract
        .nonce
        .map(|n| n.to_string())
        .unwrap_or_else(|| "-".to_string());

    view! {
        <tr
            class="border-b border-gray-700 hover:bg-gray-700 cursor-pointer"
            on:click=move |_| on_click(address_for_click.clone())
        >
            <td class="px-4 py-3 font-mono text-sm text-blue-400">{truncate_hash(&address)}</td>
            <td class="px-4 py-3 font-mono text-sm text-gray-300">{truncate_hash(&class_hash)}</td>
            <td class="px-4 py-3 text-center">{nonce}</td>
        </tr>
    }
}

#[component]
fn ContractList(on_select: impl Fn(String) + Clone + Send + 'static) -> impl IntoView {
    let contracts = LocalResource::new(|| async move { fetch_contracts(50).await });

    view! {
        <div class="bg-gray-800 rounded-lg p-4">
            <h2 class="text-xl font-semibold mb-4">"Contracts"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading contracts..."</p> }>
                {move || {
                    let on_select = on_select.clone();
                    contracts.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let contracts_data = data.contracts.clone();
                                let total = data.total;

                                if contracts_data.is_empty() {
                                    view! {
                                        <p class="text-gray-500">"No contracts found"</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <table class="w-full text-left">
                                                <thead class="text-gray-400 text-sm">
                                                    <tr>
                                                        <th class="px-4 py-2">"Address"</th>
                                                        <th class="px-4 py-2">"Class Hash"</th>
                                                        <th class="px-4 py-2 text-center">"Nonce"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {contracts_data.into_iter().map(|contract| {
                                                        let on_select = on_select.clone();
                                                        view! { <ContractRow contract=contract on_click=move |a| on_select(a) /> }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                            <p class="text-gray-400 text-sm mt-2 px-4">"Showing "{total}" contracts"</p>
                                        </div>
                                    }.into_any()
                                }
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
fn ContractDetailView(address: String, on_back: impl Fn() + 'static) -> impl IntoView {
    let addr = address.clone();
    let addr2 = address.clone();
    let contract = LocalResource::new(move || {
        let addr = addr.clone();
        async move { fetch_contract(addr).await }
    });
    let storage = LocalResource::new(move || {
        let addr = addr2.clone();
        async move { fetch_contract_storage(addr, 50).await }
    });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to contracts"
            </button>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading contract..."</p> }>
                {move || {
                    contract.get().map(|result| {
                        match result.as_ref() {
                            Ok(c) => {
                                let addr = c.address.clone();
                                let class_hash = c.class_hash.clone();
                                let nonce = c.nonce;

                                view! {
                                    <div>
                                        <h2 class="text-2xl font-bold mb-4">"Contract"</h2>
                                        <div class="grid grid-cols-2 gap-4 mb-6">
                                            <div class="col-span-2">
                                                <p class="text-gray-400">"Address"</p>
                                                <p class="font-mono text-sm break-all text-blue-400">{addr}</p>
                                            </div>
                                            <div class="col-span-2">
                                                <p class="text-gray-400">"Class Hash"</p>
                                                {match class_hash {
                                                    Some(hash) => view! {
                                                        <p class="font-mono text-sm break-all text-purple-400">
                                                            {hash}
                                                        </p>
                                                    }.into_any(),
                                                    None => view! {
                                                        <p class="text-gray-500">"None"</p>
                                                    }.into_any(),
                                                }}
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Nonce"</p>
                                                <p>{nonce.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string())}</p>
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

            <div class="mt-6">
                <h3 class="text-lg font-semibold mb-4">"Storage"</h3>
                <Suspense fallback=move || view! { <p class="text-gray-400">"Loading storage..."</p> }>
                    {move || {
                        storage.get().map(|result| {
                            match result.as_ref() {
                                Ok(data) => {
                                    let entries = data.entries.clone();
                                    if entries.is_empty() {
                                        view! {
                                            <p class="text-gray-500">"No storage entries"</p>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <div class="bg-gray-900 rounded p-3 max-h-96 overflow-y-auto">
                                                {entries.into_iter().map(|entry| {
                                                    view! {
                                                        <div class="border-b border-gray-700 py-2">
                                                            <p class="font-mono text-xs text-gray-400 break-all">
                                                                "Key: "{entry.key}
                                                            </p>
                                                            <p class="font-mono text-xs text-gray-300 break-all">
                                                                "Value: "{entry.value}
                                                            </p>
                                                        </div>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </div>
                                        }.into_any()
                                    }
                                },
                                Err(e) => view! {
                                    <p class="text-red-400">"Error loading storage: " {e.clone()}</p>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

// Class components

#[component]
fn ClassRow(class: ClassResponse, on_click: impl Fn(String) + 'static) -> impl IntoView {
    let class_hash = class.class_hash.clone();
    let class_hash_for_click = class_hash.clone();
    let class_type = class.class_type.clone();

    let type_color = match class_type.as_str() {
        "SIERRA" => "text-green-400",
        "LEGACY" => "text-yellow-400",
        _ => "text-gray-400",
    };

    view! {
        <tr
            class="border-b border-gray-700 hover:bg-gray-700 cursor-pointer"
            on:click=move |_| on_click(class_hash_for_click.clone())
        >
            <td class="px-4 py-3 font-mono text-sm text-purple-400">{truncate_hash(&class_hash)}</td>
            <td class={format!("px-4 py-3 {}", type_color)}>{class_type}</td>
        </tr>
    }
}

#[component]
fn ClassList(on_select: impl Fn(String) + Clone + Send + 'static) -> impl IntoView {
    let classes = LocalResource::new(|| async move { fetch_classes(50).await });

    view! {
        <div class="bg-gray-800 rounded-lg p-4">
            <h2 class="text-xl font-semibold mb-4">"Classes"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading classes..."</p> }>
                {move || {
                    let on_select = on_select.clone();
                    classes.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let classes_data = data.classes.clone();
                                let total = data.total;

                                if classes_data.is_empty() {
                                    view! {
                                        <p class="text-gray-500">"No classes found"</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <table class="w-full text-left">
                                                <thead class="text-gray-400 text-sm">
                                                    <tr>
                                                        <th class="px-4 py-2">"Class Hash"</th>
                                                        <th class="px-4 py-2">"Type"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {classes_data.into_iter().map(|class| {
                                                        let on_select = on_select.clone();
                                                        view! { <ClassRow class=class on_click=move |h| on_select(h) /> }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                            <p class="text-gray-400 text-sm mt-2 px-4">"Showing "{total}" classes"</p>
                                        </div>
                                    }.into_any()
                                }
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
fn ClassDetailView(class_hash: String, on_back: impl Fn() + 'static) -> impl IntoView {
    let hash = class_hash.clone();
    let class = LocalResource::new(move || {
        let hash = hash.clone();
        async move { fetch_class(hash).await }
    });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to classes"
            </button>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading class..."</p> }>
                {move || {
                    class.get().map(|result| {
                        match result.as_ref() {
                            Ok(c) => {
                                let hash = c.class_hash.clone();
                                let class_type = c.class_type.clone();
                                let compiled_hash = c.compiled_class_hash.clone();

                                let type_color = match class_type.as_str() {
                                    "SIERRA" => "text-green-400",
                                    "LEGACY" => "text-yellow-400",
                                    _ => "text-gray-400",
                                };

                                view! {
                                    <div>
                                        <h2 class="text-2xl font-bold mb-4">"Class"</h2>
                                        <div class="grid grid-cols-2 gap-4">
                                            <div class="col-span-2">
                                                <p class="text-gray-400">"Class Hash"</p>
                                                <p class="font-mono text-sm break-all text-purple-400">{hash}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Type"</p>
                                                <p class={type_color}>{class_type}</p>
                                            </div>
                                            <div>
                                                <p class="text-gray-400">"Compiled Class Hash"</p>
                                                <p class="font-mono text-sm break-all">
                                                    {compiled_hash.unwrap_or_else(|| "-".to_string())}
                                                </p>
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

// State Diff View

#[component]
fn StateDiffView(block_number: u64, on_back: impl Fn() + 'static) -> impl IntoView {
    let state_diff =
        LocalResource::new(move || async move { fetch_state_diff(block_number).await });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to block"
            </button>

            <h2 class="text-2xl font-bold mb-4">"State Diff for Block #"{block_number}</h2>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading state diff..."</p> }>
                {move || {
                    state_diff.get().map(|result| {
                        match result.as_ref() {
                            Ok(diff) => {
                                let deployed = diff.deployed_contracts.clone();
                                let storage = diff.storage_diffs.clone();
                                let declared = diff.declared_classes.clone();
                                let nonces = diff.nonces.clone();
                                let replaced = diff.replaced_classes.clone();

                                view! {
                                    <div class="space-y-6">
                                        // Deployed Contracts
                                        {if !deployed.is_empty() {
                                            view! {
                                                <div>
                                                    <h3 class="text-lg font-semibold mb-2 text-green-400">"Deployed Contracts ("{deployed.len()}")"</h3>
                                                    <div class="bg-gray-900 rounded p-3 space-y-2">
                                                        {deployed.into_iter().map(|d| {
                                                            view! {
                                                                <div class="border-b border-gray-700 pb-2">
                                                                    <p class="font-mono text-xs text-blue-400 break-all">{d.address}</p>
                                                                    <p class="font-mono text-xs text-gray-400">"Class: "{truncate_hash(&d.class_hash)}</p>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        // Storage Diffs
                                        {if !storage.is_empty() {
                                            let total_entries: usize = storage.iter().map(|s| s.storage_entries.len()).sum();
                                            view! {
                                                <div>
                                                    <h3 class="text-lg font-semibold mb-2 text-yellow-400">"Storage Changes ("{storage.len()}" contracts, "{total_entries}" entries)"</h3>
                                                    <div class="space-y-3">
                                                        {storage.into_iter().map(|s| {
                                                            let entries = s.storage_entries.clone();
                                                            let addr = s.address.clone();
                                                            view! {
                                                                <div class="bg-gray-900 rounded p-3">
                                                                    <p class="font-mono text-sm text-blue-400 mb-2 break-all">{addr}</p>
                                                                    <div class="max-h-32 overflow-y-auto">
                                                                        {entries.into_iter().map(|e| {
                                                                            view! {
                                                                                <div class="text-xs border-b border-gray-700 py-1">
                                                                                    <span class="text-gray-400 font-mono">{truncate_hash(&e.key)}</span>
                                                                                    " → "
                                                                                    <span class="text-gray-300 font-mono">{e.value}</span>
                                                                                </div>
                                                                            }
                                                                        }).collect::<Vec<_>>()}
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        // Declared Classes
                                        {if !declared.is_empty() {
                                            view! {
                                                <div>
                                                    <h3 class="text-lg font-semibold mb-2 text-purple-400">"Declared Classes ("{declared.len()}")"</h3>
                                                    <div class="bg-gray-900 rounded p-3 space-y-2">
                                                        {declared.into_iter().map(|d| {
                                                            view! {
                                                                <div class="border-b border-gray-700 pb-2">
                                                                    <p class="font-mono text-xs text-purple-400 break-all">{d.class_hash}</p>
                                                                    <p class="font-mono text-xs text-gray-400">"Compiled: "{truncate_hash(&d.compiled_class_hash)}</p>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        // Nonce Updates
                                        {if !nonces.is_empty() {
                                            view! {
                                                <div>
                                                    <h3 class="text-lg font-semibold mb-2 text-cyan-400">"Nonce Updates ("{nonces.len()}")"</h3>
                                                    <div class="bg-gray-900 rounded p-3 space-y-1">
                                                        {nonces.into_iter().map(|n| {
                                                            view! {
                                                                <div class="text-xs">
                                                                    <span class="font-mono text-blue-400">{truncate_hash(&n.contract_address)}</span>
                                                                    " → "
                                                                    <span class="font-mono text-cyan-400">{n.nonce}</span>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        // Replaced Classes
                                        {if !replaced.is_empty() {
                                            view! {
                                                <div>
                                                    <h3 class="text-lg font-semibold mb-2 text-orange-400">"Replaced Classes ("{replaced.len()}")"</h3>
                                                    <div class="bg-gray-900 rounded p-3 space-y-2">
                                                        {replaced.into_iter().map(|r| {
                                                            view! {
                                                                <div class="border-b border-gray-700 pb-2">
                                                                    <p class="font-mono text-xs text-blue-400 break-all">{r.contract_address}</p>
                                                                    <p class="font-mono text-xs text-gray-400">"New class: "{truncate_hash(&r.class_hash)}</p>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}

                                        // Show empty message if no changes
                                        {if diff.deployed_contracts.is_empty() && diff.storage_diffs.is_empty() && diff.declared_classes.is_empty() && diff.nonces.is_empty() && diff.replaced_classes.is_empty() {
                                            view! {
                                                <p class="text-gray-500">"No state changes in this block"</p>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}
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

// Advanced Filters View

#[component]
fn IndexedTransactionRow(
    tx: IndexedTransactionInfo,
    on_click: impl Fn((u64, u64)) + 'static,
) -> impl IntoView {
    let block_number = tx.block_number;
    let tx_index = tx.tx_index;
    let status_class = if tx.status == "SUCCEEDED" {
        "text-green-400"
    } else {
        "text-red-400"
    };

    view! {
        <tr
            class="border-b border-gray-700 hover:bg-gray-700 cursor-pointer"
            on:click=move |_| on_click((block_number, tx_index))
        >
            <td class="px-4 py-3 text-blue-400 font-mono">{"#"}{tx.block_number}</td>
            <td class="px-4 py-3 text-gray-400">{tx.tx_index}</td>
            <td class="px-4 py-3 font-mono text-sm text-blue-400">{truncate_hash(&tx.tx_hash)}</td>
            <td class="px-4 py-3">
                <span class="px-2 py-1 text-xs rounded bg-gray-700">{tx.tx_type}</span>
            </td>
            <td class={format!("px-4 py-3 {}", status_class)}>{tx.status.clone()}</td>
            <td class="px-4 py-3 font-mono text-sm text-gray-400">
                {tx.sender_address.clone().map(|s| truncate_hash(&s)).unwrap_or_else(|| "-".to_string())}
            </td>
        </tr>
    }
}

#[component]
fn AdvancedFiltersView(
    on_tx_select: impl Fn((u64, u64)) + Clone + Send + 'static,
) -> impl IntoView {
    let (status_filter, set_status_filter) = signal::<Option<String>>(None);
    let (block_from, set_block_from) = signal::<Option<u64>>(None);
    let (block_to, set_block_to) = signal::<Option<u64>>(None);
    let (trigger, set_trigger) = signal(0u32);

    let transactions = LocalResource::new(move || {
        let _ = trigger.get();
        let status = status_filter.get();
        let from = block_from.get();
        let to = block_to.get();
        async move { fetch_filtered_transactions(status, from, to, 100).await }
    });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <h2 class="text-2xl font-bold mb-4">"Advanced Transaction Filters"</h2>

            // Filter controls
            <div class="grid grid-cols-4 gap-4 mb-6">
                <div>
                    <label class="block text-gray-400 text-sm mb-1">"Status"</label>
                    <select
                        class="w-full px-3 py-2 bg-gray-700 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                        on:change=move |ev| {
                            let val = event_target_value(&ev);
                            set_status_filter.set(if val.is_empty() { None } else { Some(val) });
                        }
                    >
                        <option value="">"All"</option>
                        <option value="SUCCEEDED">"Succeeded"</option>
                        <option value="REVERTED">"Reverted"</option>
                    </select>
                </div>
                <div>
                    <label class="block text-gray-400 text-sm mb-1">"Block From"</label>
                    <input
                        type="number"
                        class="w-full px-3 py-2 bg-gray-700 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="0"
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_block_from.set(val.parse().ok());
                        }
                    />
                </div>
                <div>
                    <label class="block text-gray-400 text-sm mb-1">"Block To"</label>
                    <input
                        type="number"
                        class="w-full px-3 py-2 bg-gray-700 rounded text-sm focus:outline-none focus:ring-2 focus:ring-blue-500"
                        placeholder="latest"
                        on:input=move |ev| {
                            let val = event_target_value(&ev);
                            set_block_to.set(val.parse().ok());
                        }
                    />
                </div>
                <div class="flex items-end">
                    <button
                        class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm"
                        on:click=move |_| set_trigger.update(|t| *t = t.wrapping_add(1))
                    >
                        "Apply Filters"
                    </button>
                </div>
            </div>

            // Results
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading transactions..."</p> }>
                {move || {
                    let on_tx_select = on_tx_select.clone();
                    transactions.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let txs = data.transactions.clone();
                                let total = data.total;

                                if txs.is_empty() {
                                    view! {
                                        <p class="text-gray-500">"No transactions match filters"</p>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div>
                                            <p class="text-gray-400 text-sm mb-2">"Found "{total}" transactions"</p>
                                            <table class="w-full text-left">
                                                <thead class="text-gray-400 text-sm">
                                                    <tr>
                                                        <th class="px-4 py-2">"Block"</th>
                                                        <th class="px-4 py-2">"Index"</th>
                                                        <th class="px-4 py-2">"Hash"</th>
                                                        <th class="px-4 py-2">"Type"</th>
                                                        <th class="px-4 py-2">"Status"</th>
                                                        <th class="px-4 py-2">"Sender"</th>
                                                    </tr>
                                                </thead>
                                                <tbody>
                                                    {txs.into_iter().map(|tx| {
                                                        let on_tx_select = on_tx_select.clone();
                                                        view! { <IndexedTransactionRow tx=tx on_click=move |t| on_tx_select(t) /> }
                                                    }).collect::<Vec<_>>()}
                                                </tbody>
                                            </table>
                                        </div>
                                    }.into_any()
                                }
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

// Raw Data Browser Components

/// Truncate hex value for display with ellipsis
fn truncate_hex(hex: &str, max_len: usize) -> String {
    if hex.len() > max_len {
        format!(
            "{}...{}",
            &hex[..max_len / 2],
            &hex[hex.len() - max_len / 4..]
        )
    } else {
        hex.to_string()
    }
}

/// Expandable hex value component
#[component]
fn ExpandableHex(hex: String, max_len: usize) -> impl IntoView {
    let (expanded, set_expanded) = signal(false);
    let hex_clone = hex.clone();
    let hex_for_copy = hex.clone();
    let is_long = hex.len() > max_len;

    view! {
        <div class="flex items-center gap-2">
            <span
                class={move || {
                    let base = "font-mono text-sm break-all";
                    if is_long && !expanded.get() {
                        format!("{} cursor-pointer hover:text-blue-400", base)
                    } else {
                        base.to_string()
                    }
                }}
                on:click=move |_| {
                    if is_long {
                        set_expanded.update(|e| *e = !*e);
                    }
                }
                title=move || if is_long && !expanded.get() { "Click to expand" } else { "" }
            >
                {move || {
                    if expanded.get() || !is_long {
                        hex_clone.clone()
                    } else {
                        truncate_hex(&hex_clone, max_len)
                    }
                }}
            </span>
            <CopyButton text=hex_for_copy />
            {if is_long {
                view! {
                    <button
                        class="text-xs text-gray-400 hover:text-white"
                        on:click=move |_| set_expanded.update(|e| *e = !*e)
                    >
                        {move || if expanded.get() { "[-]" } else { "[+]" }}
                    </button>
                }.into_any()
            } else {
                view! { <span></span> }.into_any()
            }}
        </div>
    }
}

/// Column family row in the list
#[component]
fn ColumnFamilyRow(
    cf: ColumnFamilyInfo,
    is_expanded: bool,
    on_toggle: impl Fn() + 'static,
    on_key_click: impl Fn(String) + Clone + Send + 'static,
) -> impl IntoView {
    let cf_name = cf.name.clone();
    let cf_name_for_keys = cf.name.clone();
    let key_count = cf.key_count;

    // Signal to track prefix filter and pagination
    let (prefix_filter, set_prefix_filter) = signal(String::new());
    let (offset, set_offset) = signal(0usize);
    let limit = 50usize;

    // Fetch keys when expanded
    let keys_resource = LocalResource::new(move || {
        let cf_name = cf_name_for_keys.clone();
        let prefix = prefix_filter.get();
        let current_offset = offset.get();
        async move {
            if is_expanded {
                fetch_cf_keys(&cf_name, limit, current_offset, Some(prefix)).await
            } else {
                Err("Not expanded".to_string())
            }
        }
    });

    view! {
        <div class="border border-gray-700 rounded-lg mb-2">
            // Header row
            <div
                class="flex items-center justify-between px-4 py-3 cursor-pointer hover:bg-gray-700"
                on:click=move |_| on_toggle()
            >
                <div class="flex items-center gap-3">
                    <span class="text-gray-400">
                        {if is_expanded { "[-]" } else { "[+]" }}
                    </span>
                    <span class="font-mono text-blue-400">{cf_name.clone()}</span>
                </div>
                <span class="text-gray-400 text-sm">{key_count}" keys"</span>
            </div>

            // Expanded content
            {move || {
                if is_expanded {
                    let on_key_click = on_key_click.clone();
                    view! {
                        <div class="border-t border-gray-700 px-4 py-3">
                            // Filter input
                            <div class="flex items-center gap-2 mb-3">
                                <input
                                    type="text"
                                    placeholder="Filter by hex prefix (e.g., 0x00)"
                                    class="flex-1 px-3 py-2 bg-gray-700 rounded text-sm font-mono focus:outline-none focus:ring-2 focus:ring-blue-500"
                                    prop:value=move || prefix_filter.get()
                                    on:input=move |ev| {
                                        set_prefix_filter.set(event_target_value(&ev));
                                        set_offset.set(0);
                                    }
                                />
                                <button
                                    class="px-3 py-2 bg-gray-600 hover:bg-gray-500 rounded text-sm"
                                    on:click=move |_| {
                                        set_prefix_filter.set(String::new());
                                        set_offset.set(0);
                                    }
                                >
                                    "Clear"
                                </button>
                            </div>

                            // Keys list
                            <Suspense fallback=move || view! { <p class="text-gray-400 text-sm">"Loading keys..."</p> }>
                                {move || {
                                    let on_key_click = on_key_click.clone();
                                    keys_resource.get().map(|result| {
                                        match result.as_ref() {
                                            Ok(data) => {
                                                let keys = data.keys.clone();
                                                let total = data.total;
                                                let has_more = data.has_more;
                                                let current_offset = data.offset;

                                                if keys.is_empty() {
                                                    view! {
                                                        <p class="text-gray-500 text-sm">"No keys found"</p>
                                                    }.into_any()
                                                } else {
                                                    view! {
                                                        <div>
                                                            <div class="max-h-64 overflow-y-auto bg-gray-900 rounded p-2 mb-3">
                                                                {keys.into_iter().map(|key| {
                                                                    let key_hex = key.raw_hex.clone();
                                                                    let key_hex_for_click = key.raw_hex.clone();
                                                                    let on_key_click = on_key_click.clone();
                                                                    view! {
                                                                        <div
                                                                            class="py-1 px-2 hover:bg-gray-800 cursor-pointer rounded font-mono text-xs text-gray-300 truncate"
                                                                            on:click=move |_| on_key_click(key_hex_for_click.clone())
                                                                            title=key_hex.clone()
                                                                        >
                                                                            {truncate_hex(&key_hex, 64)}
                                                                        </div>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>

                                                            // Pagination
                                                            <div class="flex items-center justify-between text-sm">
                                                                <span class="text-gray-400">
                                                                    {current_offset + 1}"-"{current_offset + limit.min(total - current_offset)}" of "{total}
                                                                </span>
                                                                <div class="flex gap-2">
                                                                    <button
                                                                        class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50"
                                                                        disabled=move || offset.get() == 0
                                                                        on:click=move |_| set_offset.update(|o| *o = o.saturating_sub(limit))
                                                                    >
                                                                        "Prev"
                                                                    </button>
                                                                    <button
                                                                        class="px-3 py-1 bg-gray-700 hover:bg-gray-600 rounded disabled:opacity-50"
                                                                        disabled=move || !has_more
                                                                        on:click=move |_| set_offset.update(|o| *o += limit)
                                                                    >
                                                                        "Next"
                                                                    </button>
                                                                </div>
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                }
                                            },
                                            Err(_) => view! {
                                                <p class="text-gray-500 text-sm">"Expand to view keys"</p>
                                            }.into_any(),
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

/// Main Raw Data Browser view
#[component]
fn RawDataView(on_key_select: impl Fn((String, String)) + Clone + Send + 'static) -> impl IntoView {
    let (expanded_cf, set_expanded_cf) = signal::<Option<String>>(None);

    let cfs = LocalResource::new(|| fetch_column_families());

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <div class="flex items-center justify-between mb-6">
                <h2 class="text-2xl font-bold">"Raw Key-Value Browser"</h2>
                <p class="text-gray-400 text-sm">"Browse raw RocksDB column families and keys"</p>
            </div>

            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading column families..."</p> }>
                {move || {
                    let on_key_select = on_key_select.clone();
                    cfs.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let mut cf_list = data.column_families.clone();
                                // Sort by key count descending
                                cf_list.sort_by(|a, b| b.key_count.cmp(&a.key_count));

                                let total_keys: usize = cf_list.iter().map(|cf| cf.key_count).sum();
                                let cf_count = cf_list.len();

                                view! {
                                    <div>
                                        // Summary stats
                                        <div class="grid grid-cols-2 gap-4 mb-6">
                                            <div class="bg-gray-900 rounded-lg p-4">
                                                <p class="text-gray-400 text-sm">"Column Families"</p>
                                                <p class="text-2xl font-bold text-blue-400">{cf_count}</p>
                                            </div>
                                            <div class="bg-gray-900 rounded-lg p-4">
                                                <p class="text-gray-400 text-sm">"Total Keys"</p>
                                                <p class="text-2xl font-bold text-purple-400">{total_keys}</p>
                                            </div>
                                        </div>

                                        // Column families list
                                        <div>
                                            {cf_list.into_iter().map(|cf| {
                                                let cf_name = cf.name.clone();
                                                let cf_name_for_check = cf.name.clone();
                                                let cf_name_for_key = cf.name.clone();
                                                let on_key_select = on_key_select.clone();

                                                let is_expanded = move || {
                                                    expanded_cf.get().as_ref() == Some(&cf_name_for_check)
                                                };

                                                view! {
                                                    <ColumnFamilyRow
                                                        cf=cf
                                                        is_expanded=is_expanded()
                                                        on_toggle=move || {
                                                            set_expanded_cf.update(|current| {
                                                                if current.as_ref() == Some(&cf_name) {
                                                                    *current = None;
                                                                } else {
                                                                    *current = Some(cf_name.clone());
                                                                }
                                                            });
                                                        }
                                                        on_key_click=move |key| {
                                                            on_key_select((cf_name_for_key.clone(), key));
                                                        }
                                                    />
                                                }
                                            }).collect::<Vec<_>>()}
                                        </div>
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <p class="text-red-400">"Error loading column families: " {e.clone()}</p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

/// Raw Key Detail View - shows full key/value with schema info
#[component]
fn RawKeyDetailView(
    cf_name: String,
    key_hex: String,
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let cf_name_for_kv = cf_name.clone();
    let key_hex_for_kv = key_hex.clone();
    let cf_name_for_schema = cf_name.clone();

    let kv_resource = LocalResource::new(move || {
        let cf = cf_name_for_kv.clone();
        let key = key_hex_for_kv.clone();
        async move { fetch_raw_key_value(&cf, &key).await }
    });

    let schema_resource = LocalResource::new(move || {
        let cf = cf_name_for_schema.clone();
        async move { fetch_cf_schema(&cf).await }
    });

    view! {
        <div class="bg-gray-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline"
                on:click=move |_| on_back()
            >
                "< Back to Raw Data"
            </button>

            <h2 class="text-2xl font-bold mb-4">"Raw Key-Value Detail"</h2>

            // Key-Value content
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading value..."</p> }>
                {move || {
                    kv_resource.get().map(|result| {
                        match result.as_ref() {
                            Ok(resp) => {
                                if let Some(kv) = &resp.key_value {
                                    let key = kv.key_hex.clone();
                                    let value = kv.value_hex.clone();
                                    let value_size = kv.value_size;
                                    let decoded_hint = kv.decoded_hint.clone();
                                    let cf = resp.cf_name.clone();

                                    view! {
                                        <div class="space-y-6">
                                            // Column Family
                                            <div class="bg-gray-900 rounded-lg p-4">
                                                <p class="text-gray-400 text-sm mb-1">"Column Family"</p>
                                                <p class="font-mono text-blue-400">{cf}</p>
                                            </div>

                                            // Key
                                            <div class="bg-gray-900 rounded-lg p-4">
                                                <p class="text-gray-400 text-sm mb-2">"Key (hex)"</p>
                                                <ExpandableHex hex=key max_len=80 />
                                            </div>

                                            // Value
                                            <div class="bg-gray-900 rounded-lg p-4">
                                                <div class="flex items-center justify-between mb-2">
                                                    <p class="text-gray-400 text-sm">"Value (hex)"</p>
                                                    <span class="text-gray-500 text-xs">{value_size}" bytes"</span>
                                                </div>
                                                <ExpandableHex hex=value max_len=120 />
                                            </div>

                                            // Decoded hint if available
                                            {decoded_hint.map(|hint| {
                                                view! {
                                                    <div class="bg-gray-900 rounded-lg p-4 border-l-4 border-green-500">
                                                        <p class="text-gray-400 text-sm mb-2">"Decoded Hint"</p>
                                                        <pre class="font-mono text-sm text-green-400 whitespace-pre-wrap break-all">{hint}</pre>
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="bg-gray-900 rounded-lg p-4">
                                            <p class="text-yellow-400">"Key not found in column family"</p>
                                        </div>
                                    }.into_any()
                                }
                            },
                            Err(e) => view! {
                                <p class="text-red-400">"Error: " {e.clone()}</p>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>

            // Schema documentation section
            <div class="mt-6">
                <h3 class="text-lg font-semibold mb-4">"Schema Documentation"</h3>
                <Suspense fallback=move || view! { <p class="text-gray-400 text-sm">"Loading schema..."</p> }>
                    {move || {
                        schema_resource.get().map(|result| {
                            match result.as_ref() {
                                Ok(schema) => {
                                    let purpose = schema.purpose.clone();
                                    let category = schema.category.clone();
                                    let key_type = schema.key.rust_type.clone();
                                    let key_encoding = schema.key.encoding.clone();
                                    let key_desc = schema.key.description.clone();
                                    let value_type = schema.value.rust_type.clone();
                                    let value_encoding = schema.value.encoding.clone();
                                    let value_desc = schema.value.description.clone();
                                    let fields = schema.value.fields.clone();
                                    let relationships = schema.relationships.clone();

                                    view! {
                                        <div class="bg-gray-900 rounded-lg p-4 space-y-4">
                                            // Purpose
                                            <div>
                                                <p class="text-gray-400 text-sm">"Purpose"</p>
                                                <p class="text-gray-200">{purpose}</p>
                                            </div>

                                            // Category
                                            <div>
                                                <p class="text-gray-400 text-sm">"Category"</p>
                                                <span class="px-2 py-1 text-xs rounded bg-gray-700 text-purple-400">{category}</span>
                                            </div>

                                            // Key format
                                            <div class="border-t border-gray-700 pt-4">
                                                <p class="text-gray-400 text-sm mb-2">"Key Format"</p>
                                                <div class="grid grid-cols-2 gap-2 text-sm">
                                                    <div>
                                                        <span class="text-gray-500">"Type: "</span>
                                                        <span class="font-mono text-cyan-400">{key_type}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-gray-500">"Encoding: "</span>
                                                        <span class="font-mono text-yellow-400">{key_encoding}</span>
                                                    </div>
                                                </div>
                                                <p class="text-gray-300 text-sm mt-1">{key_desc}</p>
                                            </div>

                                            // Value format
                                            <div class="border-t border-gray-700 pt-4">
                                                <p class="text-gray-400 text-sm mb-2">"Value Format"</p>
                                                <div class="grid grid-cols-2 gap-2 text-sm">
                                                    <div>
                                                        <span class="text-gray-500">"Type: "</span>
                                                        <span class="font-mono text-cyan-400">{value_type}</span>
                                                    </div>
                                                    <div>
                                                        <span class="text-gray-500">"Encoding: "</span>
                                                        <span class="font-mono text-yellow-400">{value_encoding}</span>
                                                    </div>
                                                </div>
                                                <p class="text-gray-300 text-sm mt-1">{value_desc}</p>

                                                // Fields if any
                                                {if !fields.is_empty() {
                                                    view! {
                                                        <div class="mt-3">
                                                            <p class="text-gray-500 text-xs mb-1">"Fields:"</p>
                                                            <div class="space-y-1">
                                                                {fields.into_iter().map(|f| {
                                                                    view! {
                                                                        <div class="text-xs bg-gray-800 rounded px-2 py-1">
                                                                            <span class="font-mono text-blue-400">{f.name}</span>
                                                                            <span class="text-gray-500">": "</span>
                                                                            <span class="font-mono text-cyan-400">{f.rust_type}</span>
                                                                            {if !f.description.is_empty() {
                                                                                view! {
                                                                                    <span class="text-gray-400">" - "{f.description}</span>
                                                                                }.into_any()
                                                                            } else {
                                                                                view! { <span></span> }.into_any()
                                                                            }}
                                                                        </div>
                                                                    }
                                                                }).collect::<Vec<_>>()}
                                                            </div>
                                                        </div>
                                                    }.into_any()
                                                } else {
                                                    view! { <div></div> }.into_any()
                                                }}
                                            </div>

                                            // Relationships if any
                                            {if !relationships.is_empty() {
                                                view! {
                                                    <div class="border-t border-gray-700 pt-4">
                                                        <p class="text-gray-400 text-sm mb-2">"Related Column Families"</p>
                                                        <div class="space-y-2">
                                                            {relationships.into_iter().map(|r| {
                                                                view! {
                                                                    <div class="text-sm bg-gray-800 rounded px-3 py-2">
                                                                        <span class="font-mono text-blue-400">{r.target_cf}</span>
                                                                        <span class="text-gray-500">" ("</span>
                                                                        <span class="text-orange-400">{r.relationship_type}</span>
                                                                        <span class="text-gray-500">")"</span>
                                                                        <p class="text-gray-400 text-xs mt-1">{r.description}</p>
                                                                    </div>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div></div> }.into_any()
                                            }}
                                        </div>
                                    }.into_any()
                                },
                                Err(_) => view! {
                                    <div class="bg-gray-900 rounded-lg p-4">
                                        <p class="text-gray-500 text-sm">"Schema documentation not available for this column family"</p>
                                    </div>
                                }.into_any(),
                            }
                        })
                    }}
                </Suspense>
            </div>
        </div>
    }
}

#[component]
fn IndexStatusCard() -> impl IntoView {
    let status = LocalResource::new(|| fetch_index_status());

    view! {
        <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-4">
            <h2 class="text-lg font-semibold mb-3">"Index Status"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || {
                    status.get().map(|result| {
                        match result.as_ref() {
                            Ok(s) => {
                                let synced = s.is_synced;
                                let indexed = s.indexed_blocks;
                                let latest = s.latest_block;
                                let total_tx = s.total_transactions;
                                let failed_tx = s.failed_transactions;
                                let sync_class = if synced { "text-green-400" } else { "text-yellow-400" };

                                view! {
                                    <div class="space-y-2 text-sm">
                                        <p>
                                            <span class="text-gray-400">"Status: "</span>
                                            <span class=sync_class>{if synced { "Synced" } else { "Syncing..." }}</span>
                                        </p>
                                        <p>
                                            <span class="text-gray-400">"Indexed: "</span>
                                            <span class="text-blue-400">{indexed}</span>
                                            <span class="text-gray-500">" / "</span>
                                            <span>{latest}</span>
                                        </p>
                                        <p>
                                            <span class="text-gray-400">"Transactions: "</span>
                                            <span class="text-purple-400">{total_tx}</span>
                                        </p>
                                        {if failed_tx > 0 {
                                            view! {
                                                <p>
                                                    <span class="text-gray-400">"Failed: "</span>
                                                    <span class="text-red-400">{failed_tx}</span>
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            },
                            Err(_) => view! {
                                <p class="text-gray-500 text-sm">"Index unavailable"</p>
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
        <div class="bg-slate-900/50 border border-slate-800 rounded-xl p-4">
            <h2 class="text-lg font-semibold mb-3">"Database Stats"</h2>
            <Suspense fallback=move || view! { <p class="text-gray-400">"Loading..."</p> }>
                {move || {
                    stats.get().map(|result| {
                        match result.as_ref() {
                            Ok(s) => {
                                let latest = s.latest_block.unwrap_or(0);
                                let cols = s.column_count;
                                let version_text = match s.madara_db_version.version {
                                    Some(v) => format!("v{}", v),
                                    None => "Unknown".to_string(),
                                };
                                let (support_text, support_class) = match s.madara_db_version.supported {
                                    Some(true) => ("Supported", "text-green-400"),
                                    Some(false) => ("Unsupported", "text-red-400"),
                                    None => ("Unknown", "text-yellow-400"),
                                };
                                let mut supported_versions = s.madara_db_version.supported_versions.clone();
                                supported_versions.sort_unstable();
                                let supported_versions_text = supported_versions
                                    .into_iter()
                                    .map(|v| v.to_string())
                                    .collect::<Vec<_>>()
                                    .join(", ");
                                let show_supported_versions = s.madara_db_version.supported != Some(true);
                                let version_error = s.madara_db_version.error.clone();
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
                                        <p>
                                            <span class="text-gray-400">"Madara DB Version: "</span>
                                            <span class="text-blue-400">{version_text}</span>
                                            <span class="text-gray-500">" ("</span>
                                            <span class=support_class>{support_text}</span>
                                            <span class="text-gray-500">")"</span>
                                        </p>
                                        {if show_supported_versions {
                                            view! {
                                                <p class="text-xs text-gray-500">
                                                    <span class="text-gray-400">"Supported versions: "</span>
                                                    {supported_versions_text}
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
                                        {if let Some(err) = version_error {
                                            view! {
                                                <p class="text-xs text-yellow-400">
                                                    <span class="text-gray-400">"Version note: "</span>
                                                    {err}
                                                </p>
                                            }.into_any()
                                        } else {
                                            view! { <span></span> }.into_any()
                                        }}
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
fn NavItem(label: &'static str, active: bool, on_click: impl Fn() + 'static) -> impl IntoView {
    let class = if active {
        "w-full px-3 py-2 text-left text-sm rounded-lg bg-slate-800/80 text-slate-50 ring-1 ring-slate-700/80 hover:bg-slate-800 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
    } else {
        "w-full px-3 py-2 text-left text-sm rounded-lg text-slate-300 hover:bg-slate-800/60 hover:text-slate-100 transition-colors focus:outline-none focus-visible:ring-2 focus-visible:ring-sky-500 focus-visible:ring-offset-2 focus-visible:ring-offset-slate-900"
    };

    view! {
        <button
            class=class
            type="button"
            on:click=move |_| on_click()
        >
            {label}
        </button>
    }
}

#[component]
fn SearchBar(on_result: impl Fn(Page) + Clone + Send + 'static) -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (searching, set_searching) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (trigger, set_trigger) = signal(0u32);

    // Effect to handle search when triggered
    {
        let on_result = on_result.clone();
        Effect::new(move |_| {
            let _ = trigger.get(); // Subscribe to trigger
            let q = query.get();
            if q.trim().is_empty() {
                return;
            }
            let on_result = on_result.clone();
            set_searching.set(true);
            set_error.set(None);
            leptos::task::spawn_local(async move {
                match fetch_search(q).await {
                    Ok(result) => {
                        set_searching.set(false);
                        match result.result_type.as_str() {
                            "block" => {
                                if let Some(bn) = result.block_number {
                                    on_result(Page::BlockDetail { block_number: bn });
                                }
                            }
                            "transaction" => {
                                if let (Some(bn), Some(idx)) =
                                    (result.block_number, result.tx_index)
                                {
                                    on_result(Page::TransactionDetail {
                                        block_number: bn,
                                        tx_index: idx as usize,
                                    });
                                }
                            }
                            "contract" => {
                                if let Some(addr) = result.address {
                                    on_result(Page::ContractDetail { address: addr });
                                }
                            }
                            "class" => {
                                if let Some(hash) = result.class_hash {
                                    on_result(Page::ClassDetail { class_hash: hash });
                                }
                            }
                            "not_found" => {
                                set_error.set(Some("No results found".to_string()));
                            }
                            _ => {
                                set_error.set(Some("Unknown result type".to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        set_searching.set(false);
                        set_error.set(Some(e));
                    }
                }
            });
        });
    }

    view! {
        <div class="w-full sm:w-auto">
            <div class="flex flex-col sm:flex-row sm:items-center gap-2 w-full">
                <input
                    type="text"
                    placeholder="Search block, tx hash, contract..."
                    class="min-w-0 w-full sm:w-72 md:w-96 px-3 py-2 bg-slate-800/80 rounded-lg text-sm text-slate-100 placeholder:text-slate-400 ring-1 ring-slate-700/80 focus:outline-none focus:ring-2 focus:ring-sky-500"
                    prop:value=move || query.get()
                    on:input=move |ev| set_query.set(event_target_value(&ev))
                    on:keypress=move |ev| {
                        if ev.key() == "Enter" {
                            set_trigger.update(|t| *t = t.wrapping_add(1));
                        }
                    }
                />
                <button
                    class="shrink-0 px-4 py-2 bg-sky-600 hover:bg-sky-700 rounded-lg text-sm text-white disabled:opacity-50 transition-colors"
                    disabled=move || searching.get()
                    on:click=move |_| set_trigger.update(|t| *t = t.wrapping_add(1))
                >
                    {move || if searching.get() { "..." } else { "Search" }}
                </button>
            </div>
            {move || error.get().map(|e| view! {
                <p class="mt-1 text-xs text-rose-300">{e}</p>
            })}
        </div>
    }
}

// Schema Browser Components

#[component]
fn SchemaCategoryCard(
    category: SchemaCategoryInfo,
    is_expanded: bool,
    on_toggle: impl Fn() + 'static,
    on_cf_select: impl Fn(String) + Clone + Send + 'static,
) -> impl IntoView {
    let cat_name = category.name.clone();
    let cat_name_for_fetch = category.name.clone();
    let count = category.column_family_count;
    let description = category.description.clone();

    // Fetch column families for this category when expanded
    let column_families = LocalResource::new(move || {
        let cat = cat_name_for_fetch.clone();
        async move {
            if is_expanded {
                fetch_schema_column_families(Some(&cat)).await
            } else {
                Ok(SchemaColumnFamiliesResponse {
                    column_families: vec![],
                    total: 0,
                })
            }
        }
    });

    view! {
        <div class="bg-slate-800 rounded-lg overflow-hidden">
            // Category header
            <button
                class="w-full px-4 py-3 flex items-center justify-between hover:bg-slate-700 transition-colors"
                on:click=move |_| on_toggle()
            >
                <div class="flex items-center gap-3">
                    <span class="text-lg font-semibold text-white capitalize">{cat_name.clone()}</span>
                    <span class="px-2 py-0.5 text-xs bg-slate-600 rounded-full text-gray-300">
                        {count}" CFs"
                    </span>
                </div>
                <span class="text-gray-400 text-lg">
                    {if is_expanded { "▼" } else { "▶" }}
                </span>
            </button>

            // Description
            <div class="px-4 pb-2">
                <p class="text-sm text-gray-400">{description}</p>
            </div>

            // Expanded content - column families list
            {move || {
                if is_expanded {
                    let on_cf_select = on_cf_select.clone();
                    view! {
                        <div class="border-t border-slate-700">
                            <Suspense fallback=move || view! {
                                <div class="p-4 text-gray-400">"Loading column families..."</div>
                            }>
                                {move || {
                                    let on_cf_select = on_cf_select.clone();
                                    column_families.get().map(|result| {
                                        match result.as_ref() {
                                            Ok(data) => {
                                                let cfs = data.column_families.clone();
                                                view! {
                                                    <div class="divide-y divide-slate-700">
                                                        {cfs.into_iter().map(|cf| {
                                                            let cf_name = cf.name.clone();
                                                            let cf_name_click = cf.name.clone();
                                                            let purpose = cf.purpose.clone();
                                                            let on_cf_select = on_cf_select.clone();
                                                            view! {
                                                                <button
                                                                    class="w-full px-4 py-3 text-left hover:bg-slate-700 transition-colors"
                                                                    on:click=move |_| on_cf_select(cf_name_click.clone())
                                                                >
                                                                    <div class="font-mono text-sm text-blue-400 mb-1">
                                                                        {cf_name}
                                                                    </div>
                                                                    <div class="text-xs text-gray-400 line-clamp-2">
                                                                        {purpose}
                                                                    </div>
                                                                </button>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                }.into_any()
                                            },
                                            Err(e) => view! {
                                                <div class="p-4 text-red-400">
                                                    "Error: "{e.clone()}
                                                </div>
                                            }.into_any(),
                                        }
                                    })
                                }}
                            </Suspense>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

#[component]
fn SchemaView(on_cf_select: impl Fn(String) + Clone + Send + 'static) -> impl IntoView {
    let categories = LocalResource::new(|| fetch_schema_categories());
    let (expanded_category, set_expanded_category) = signal::<Option<String>>(None);

    view! {
        <div class="bg-slate-800 rounded-lg p-6">
            <div class="mb-6">
                <h2 class="text-2xl font-bold text-white mb-2">"Database Schema Browser"</h2>
                <p class="text-gray-400">
                    "Explore Madara's RocksDB column families, their key/value encoding, and relationships. "
                    "Click on a column family to see detailed schema documentation."
                </p>
            </div>

            <Suspense fallback=move || view! {
                <div class="text-gray-400">"Loading schema categories..."</div>
            }>
                {move || {
                    let on_cf_select = on_cf_select.clone();
                    categories.get().map(|result| {
                        match result.as_ref() {
                            Ok(data) => {
                                let cats = data.categories.clone();
                                view! {
                                    <div class="space-y-3">
                                        {cats.into_iter().map(|cat| {
                                            let cat_name_check = cat.name.clone();
                                            let cat_name_toggle = cat.name.clone();
                                            let on_cf_select = on_cf_select.clone();
                                            let is_expanded = move || expanded_category.get() == Some(cat_name_check.clone());
                                            view! {
                                                <SchemaCategoryCard
                                                    category=cat
                                                    is_expanded=is_expanded()
                                                    on_toggle=move || {
                                                        let current = expanded_category.get();
                                                        if current == Some(cat_name_toggle.clone()) {
                                                            set_expanded_category.set(None);
                                                        } else {
                                                            set_expanded_category.set(Some(cat_name_toggle.clone()));
                                                        }
                                                    }
                                                    on_cf_select=move |name| on_cf_select(name)
                                                />
                                            }
                                        }).collect::<Vec<_>>()}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <div class="text-red-400">"Error loading categories: "{e.clone()}</div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

#[component]
fn SchemaDetailView(cf_name: String, on_back: impl Fn() + 'static) -> impl IntoView {
    let name = cf_name.clone();
    let schema = LocalResource::new(move || {
        let n = name.clone();
        async move { fetch_cf_schema(&n).await }
    });

    view! {
        <div class="bg-slate-800 rounded-lg p-6">
            <button
                class="mb-4 text-blue-400 hover:underline flex items-center gap-1"
                on:click=move |_| on_back()
            >
                "< Back to Schema Browser"
            </button>

            <Suspense fallback=move || view! {
                <div class="text-gray-400">"Loading schema details..."</div>
            }>
                {move || {
                    schema.get().map(|result| {
                        match result.as_ref() {
                            Ok(s) => {
                                let name = s.name.clone();
                                let category = s.category.clone();
                                let purpose = s.purpose.clone();
                                let key = s.key.clone();
                                let value = s.value.clone();
                                let relationships = s.relationships.clone();

                                view! {
                                    <div class="space-y-6">
                                        // Header
                                        <div>
                                            <div class="flex items-center gap-3 mb-2">
                                                <h2 class="text-2xl font-bold font-mono text-white">{name}</h2>
                                                <span class="px-2 py-1 text-xs bg-slate-600 rounded text-gray-300 capitalize">
                                                    {category}
                                                </span>
                                            </div>
                                            <p class="text-gray-300">{purpose}</p>
                                        </div>

                                        // Key Encoding Section
                                        <div class="bg-slate-900 rounded-lg p-4">
                                            <h3 class="text-lg font-semibold text-white mb-3 flex items-center gap-2">
                                                <span class="text-blue-400">"KEY"</span>
                                                " Encoding"
                                            </h3>
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                                <div>
                                                    <p class="text-sm text-gray-400 mb-1">"Rust Type"</p>
                                                    <code class="text-sm font-mono text-green-400 bg-slate-800 px-2 py-1 rounded">
                                                        {key.rust_type.clone()}
                                                    </code>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-400 mb-1">"Encoding"</p>
                                                    <code class="text-sm font-mono text-yellow-400 bg-slate-800 px-2 py-1 rounded">
                                                        {key.encoding.clone()}
                                                    </code>
                                                </div>
                                                {key.size_bytes.map(|size| view! {
                                                    <div>
                                                        <p class="text-sm text-gray-400 mb-1">"Size"</p>
                                                        <code class="text-sm font-mono text-purple-400 bg-slate-800 px-2 py-1 rounded">
                                                            {size}" bytes"
                                                        </code>
                                                    </div>
                                                })}
                                            </div>
                                            <div class="mb-4">
                                                <p class="text-sm text-gray-400 mb-1">"Description"</p>
                                                <p class="text-sm text-gray-300">{key.description.clone()}</p>
                                            </div>
                                            <div class="bg-slate-950 rounded p-3">
                                                <p class="text-xs text-gray-400 mb-2">"Example"</p>
                                                <div class="space-y-1">
                                                    <div class="flex items-start gap-2">
                                                        <span class="text-xs text-gray-500 w-16">"Raw:"</span>
                                                        <code class="text-xs font-mono text-gray-300 break-all">
                                                            {key.example_raw.clone()}
                                                        </code>
                                                    </div>
                                                    <div class="flex items-start gap-2">
                                                        <span class="text-xs text-gray-500 w-16">"Decoded:"</span>
                                                        <code class="text-xs font-mono text-blue-300 break-all">
                                                            {key.example_decoded.clone()}
                                                        </code>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>

                                        // Value Serialization Section
                                        <div class="bg-slate-900 rounded-lg p-4">
                                            <h3 class="text-lg font-semibold text-white mb-3 flex items-center gap-2">
                                                <span class="text-purple-400">"VALUE"</span>
                                                " Serialization"
                                            </h3>
                                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4 mb-4">
                                                <div>
                                                    <p class="text-sm text-gray-400 mb-1">"Rust Type"</p>
                                                    <code class="text-sm font-mono text-green-400 bg-slate-800 px-2 py-1 rounded">
                                                        {value.rust_type.clone()}
                                                    </code>
                                                </div>
                                                <div>
                                                    <p class="text-sm text-gray-400 mb-1">"Encoding"</p>
                                                    <code class="text-sm font-mono text-yellow-400 bg-slate-800 px-2 py-1 rounded">
                                                        {value.encoding.clone()}
                                                    </code>
                                                </div>
                                            </div>
                                            <div class="mb-4">
                                                <p class="text-sm text-gray-400 mb-1">"Description"</p>
                                                <p class="text-sm text-gray-300">{value.description.clone()}</p>
                                            </div>

                                            // Fields table
                                            {if !value.fields.is_empty() {
                                                let fields = value.fields.clone();
                                                view! {
                                                    <div>
                                                        <p class="text-sm text-gray-400 mb-2">"Fields"</p>
                                                        <div class="bg-slate-950 rounded overflow-hidden">
                                                            <table class="w-full text-sm">
                                                                <thead class="bg-slate-800">
                                                                    <tr>
                                                                        <th class="px-3 py-2 text-left text-gray-400 font-medium">"Name"</th>
                                                                        <th class="px-3 py-2 text-left text-gray-400 font-medium">"Type"</th>
                                                                        <th class="px-3 py-2 text-left text-gray-400 font-medium">"Description"</th>
                                                                    </tr>
                                                                </thead>
                                                                <tbody class="divide-y divide-slate-800">
                                                                    {fields.into_iter().map(|f| {
                                                                        view! {
                                                                            <tr>
                                                                                <td class="px-3 py-2 font-mono text-blue-300">{f.name}</td>
                                                                                <td class="px-3 py-2 font-mono text-green-300">{f.rust_type}</td>
                                                                                <td class="px-3 py-2 text-gray-300">{f.description}</td>
                                                                            </tr>
                                                                        }
                                                                    }).collect::<Vec<_>>()}
                                                                </tbody>
                                                            </table>
                                                        </div>
                                                    </div>
                                                }.into_any()
                                            } else {
                                                view! { <div></div> }.into_any()
                                            }}
                                        </div>

                                        // Relationships Section
                                        {if !relationships.is_empty() {
                                            let rels = relationships.clone();
                                            view! {
                                                <div class="bg-slate-900 rounded-lg p-4">
                                                    <h3 class="text-lg font-semibold text-white mb-3 flex items-center gap-2">
                                                        <span class="text-cyan-400">"RELATIONSHIPS"</span>
                                                    </h3>
                                                    <div class="space-y-3">
                                                        {rels.into_iter().map(|rel| {
                                                            let rel_type_color = match rel.relationship_type.as_str() {
                                                                "inverse" => "text-yellow-400",
                                                                "references" => "text-blue-400",
                                                                "contains" => "text-green-400",
                                                                "indexed_by" => "text-purple-400",
                                                                _ => "text-gray-400",
                                                            };
                                                            view! {
                                                                <div class="flex items-start gap-3 bg-slate-950 rounded p-3">
                                                                    <span class={format!("text-sm font-mono {}", rel_type_color)}>
                                                                        {rel.relationship_type}
                                                                    </span>
                                                                    <span class="text-gray-500">"→"</span>
                                                                    <div>
                                                                        <code class="font-mono text-sm text-blue-300">
                                                                            {rel.target_cf}
                                                                        </code>
                                                                        <p class="text-sm text-gray-400 mt-1">
                                                                            {rel.description}
                                                                        </p>
                                                                    </div>
                                                                </div>
                                                            }
                                                        }).collect::<Vec<_>>()}
                                                    </div>
                                                </div>
                                            }.into_any()
                                        } else {
                                            view! { <div></div> }.into_any()
                                        }}
                                    </div>
                                }.into_any()
                            },
                            Err(e) => view! {
                                <div class="text-red-400">"Error loading schema: "{e.clone()}</div>
                            }.into_any(),
                        }
                    })
                }}
            </Suspense>
        </div>
    }
}

// SQL Console View Component

/// Component to display a single table's schema in the sidebar
#[component]
fn TableSchemaItem(
    table: TableInfo,
    is_expanded: bool,
    on_toggle: impl Fn() + 'static,
    on_insert_template: impl Fn(String) + Clone + Send + 'static,
) -> impl IntoView {
    let table_name = table.name.clone();
    let table_name_for_template = table.name.clone();
    let columns = table.columns.clone();
    let row_count = table.row_count;

    view! {
        <div class="border-b border-slate-700 last:border-b-0">
            <button
                class="w-full px-3 py-2 text-left hover:bg-slate-700 flex items-center justify-between text-sm"
                on:click=move |_| on_toggle()
            >
                <div class="flex items-center gap-2">
                    <span class="text-gray-400 text-xs">
                        {if is_expanded { "[-]" } else { "[+]" }}
                    </span>
                    <span class="font-mono text-blue-400">{table_name.clone()}</span>
                </div>
                <span class="text-gray-500 text-xs">{row_count}" rows"</span>
            </button>
            {move || {
                if is_expanded {
                    let cols = columns.clone();
                    let on_insert = on_insert_template.clone();
                    let tname = table_name_for_template.clone();
                    view! {
                        <div class="px-3 pb-2 space-y-1">
                            <div class="bg-slate-900 rounded p-2 text-xs space-y-1">
                                {cols.into_iter().map(|col| {
                                    view! {
                                        <div class="flex items-center justify-between">
                                            <span class="font-mono text-gray-300">{col.name}</span>
                                            <span class="text-gray-500">{col.data_type}</span>
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                            <button
                                class="w-full px-2 py-1 text-xs bg-slate-700 hover:bg-slate-600 rounded text-gray-300"
                                on:click=move |_| on_insert(tname.clone())
                            >
                                "Insert SELECT template"
                            </button>
                        </div>
                    }.into_any()
                } else {
                    view! { <div></div> }.into_any()
                }
            }}
        </div>
    }
}

/// Format a JSON value for display in the results table
fn format_json_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => "NULL".to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Array(arr) => {
            serde_json::to_string(arr).unwrap_or_else(|_| "[]".to_string())
        }
        serde_json::Value::Object(obj) => {
            serde_json::to_string(obj).unwrap_or_else(|_| "{}".to_string())
        }
    }
}

#[component]
fn SqlConsoleView() -> impl IntoView {
    // SQL query input state
    let (sql_input, set_sql_input) = signal(String::new());
    let (is_executing, set_is_executing) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);
    let (result, set_result) = signal::<Option<QueryResult>>(None);
    let (expanded_table, set_expanded_table) = signal::<Option<String>>(None);

    // Fetch tables list
    let tables = LocalResource::new(|| fetch_index_tables());

    // Execute query handler
    let execute_query = move || {
        let sql = sql_input.get();
        if sql.trim().is_empty() {
            set_error.set(Some("Please enter a SQL query".to_string()));
            return;
        }

        set_is_executing.set(true);
        set_error.set(None);
        set_result.set(None);

        leptos::task::spawn_local(async move {
            match execute_sql_query(sql, vec![]).await {
                Ok(query_result) => {
                    set_result.set(Some(query_result));
                    set_error.set(None);
                }
                Err(e) => {
                    set_error.set(Some(e));
                    set_result.set(None);
                }
            }
            set_is_executing.set(false);
        });
    };

    // Copy results as JSON
    let copy_results_json = move || {
        if let Some(res) = result.get() {
            let json_data = serde_json::json!({
                "columns": res.columns,
                "rows": res.rows,
                "row_count": res.row_count,
                "truncated": res.truncated
            });
            if let Ok(json_str) = serde_json::to_string_pretty(&json_data) {
                copy_to_clipboard(&json_str);
            }
        }
    };

    view! {
        <div class="flex gap-4 h-full">
            // Left sidebar - Tables schema
            <div class="w-64 flex-shrink-0 bg-slate-800 rounded-lg overflow-hidden flex flex-col">
                <div class="px-4 py-3 bg-slate-700 border-b border-slate-600">
                    <h3 class="font-semibold text-white">"Indexed Tables"</h3>
                </div>
                <div class="flex-1 overflow-y-auto">
                    <Suspense fallback=move || view! {
                        <div class="p-4 text-gray-400 text-sm">"Loading tables..."</div>
                    }>
                        {move || {
                            tables.get().map(|result| {
                                match result.as_ref() {
                                    Ok(data) => {
                                        let table_list = data.tables.clone();
                                        if table_list.is_empty() {
                                            view! {
                                                <div class="p-4 text-gray-500 text-sm">"No indexed tables found"</div>
                                            }.into_any()
                                        } else {
                                            view! {
                                                <div>
                                                    {table_list.into_iter().map(|table| {
                                                        let table_name = table.name.clone();
                                                        let table_name_check = table.name.clone();
                                                        let is_exp = move || expanded_table.get() == Some(table_name_check.clone());
                                                        view! {
                                                            <TableSchemaItem
                                                                table=table
                                                                is_expanded=is_exp()
                                                                on_toggle=move || {
                                                                    if expanded_table.get() == Some(table_name.clone()) {
                                                                        set_expanded_table.set(None);
                                                                    } else {
                                                                        set_expanded_table.set(Some(table_name.clone()));
                                                                    }
                                                                }
                                                                on_insert_template=move |name| {
                                                                    set_sql_input.set(format!("SELECT * FROM {} LIMIT 10", name));
                                                                }
                                                            />
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </div>
                                            }.into_any()
                                        }
                                    },
                                    Err(e) => view! {
                                        <div class="p-4 text-red-400 text-sm">"Error: "{e.clone()}</div>
                                    }.into_any(),
                                }
                            })
                        }}
                    </Suspense>
                </div>
            </div>

            // Main content area
            <div class="flex-1 flex flex-col gap-4">
                // Query input area
                <div class="bg-slate-800 rounded-lg p-4">
                    <div class="flex items-center justify-between mb-3">
                        <h2 class="text-xl font-bold text-white">"SQL Console"</h2>
                        <div class="text-gray-400 text-sm">
                            "Query the SQLite index database"
                        </div>
                    </div>

                    // SQL textarea
                    <div class="mb-3">
                        <textarea
                            class="w-full h-32 px-4 py-3 bg-slate-900 rounded-lg font-mono text-sm text-gray-200 placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 resize-y"
                            placeholder="Enter SQL query here... e.g., SELECT * FROM transactions WHERE status = 'REVERTED' LIMIT 10"
                            prop:value=move || sql_input.get()
                            on:input=move |ev| set_sql_input.set(event_target_value(&ev))
                            on:keydown=move |ev| {
                                // Ctrl+Enter or Cmd+Enter to execute
                                if (ev.ctrl_key() || ev.meta_key()) && ev.key() == "Enter" {
                                    execute_query();
                                }
                            }
                        ></textarea>
                    </div>

                    // Execute button and hints
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <button
                                class="px-4 py-2 bg-green-600 hover:bg-green-700 disabled:opacity-50 disabled:cursor-not-allowed rounded font-medium text-white transition-colors"
                                disabled=move || is_executing.get()
                                on:click=move |_| execute_query()
                            >
                                {move || if is_executing.get() { "Executing..." } else { "Execute" }}
                            </button>
                            <span class="text-gray-500 text-sm">"Ctrl+Enter to run"</span>
                        </div>
                        {move || result.get().map(|_| {
                            view! {
                                <button
                                    class="px-3 py-1 bg-slate-700 hover:bg-slate-600 rounded text-sm text-gray-300"
                                    on:click=move |_| copy_results_json()
                                >
                                    "Copy as JSON"
                                </button>
                            }
                        })}
                    </div>
                </div>

                // Error display
                {move || error.get().map(|e| {
                    view! {
                        <div class="bg-red-900/30 border border-red-700 rounded-lg p-4">
                            <div class="flex items-start gap-3">
                                <span class="text-red-400 font-bold">"Error:"</span>
                                <pre class="text-red-300 text-sm font-mono whitespace-pre-wrap break-all">{e}</pre>
                            </div>
                        </div>
                    }
                })}

                // Results area
                {move || result.get().map(|res| {
                    let columns = res.columns.clone();
                    let rows = res.rows.clone();
                    let row_count = res.row_count;
                    let truncated = res.truncated;

                    view! {
                        <div class="bg-slate-800 rounded-lg flex-1 flex flex-col overflow-hidden">
                            // Results header
                            <div class="px-4 py-3 bg-slate-700 border-b border-slate-600 flex items-center justify-between">
                                <div class="flex items-center gap-3">
                                    <span class="font-semibold text-white">"Results"</span>
                                    <span class="text-gray-400 text-sm">
                                        {row_count}" row"{if row_count == 1 { "" } else { "s" }}
                                    </span>
                                </div>
                                {if truncated {
                                    view! {
                                        <span class="px-2 py-1 bg-yellow-600/30 text-yellow-400 text-xs rounded">
                                            "Results truncated"
                                        </span>
                                    }.into_any()
                                } else {
                                    view! { <span></span> }.into_any()
                                }}
                            </div>

                            // Results table
                            {if rows.is_empty() {
                                view! {
                                    <div class="p-4 text-gray-500 text-center">"No results"</div>
                                }.into_any()
                            } else {
                                view! {
                                    <div class="flex-1 overflow-auto">
                                        <table class="w-full text-left">
                                            <thead class="bg-slate-900 sticky top-0">
                                                <tr>
                                                    {columns.clone().into_iter().map(|col| {
                                                        view! {
                                                            <th class="px-3 py-2 text-gray-400 font-medium text-sm border-b border-slate-700 whitespace-nowrap">
                                                                {col}
                                                            </th>
                                                        }
                                                    }).collect::<Vec<_>>()}
                                                </tr>
                                            </thead>
                                            <tbody class="divide-y divide-slate-700">
                                                {rows.into_iter().enumerate().map(|(idx, row)| {
                                                    let row_class = if idx % 2 == 0 {
                                                        "bg-slate-800"
                                                    } else {
                                                        "bg-slate-800/50"
                                                    };
                                                    view! {
                                                        <tr class=row_class>
                                                            {row.into_iter().map(|cell| {
                                                                let formatted = format_json_value(&cell);
                                                                let formatted_clone = formatted.clone();
                                                                let is_null = matches!(cell, serde_json::Value::Null);
                                                                let cell_class = if is_null {
                                                                    "px-3 py-2 font-mono text-sm text-gray-500 italic"
                                                                } else {
                                                                    "px-3 py-2 font-mono text-sm text-gray-300"
                                                                };
                                                                view! {
                                                                    <td class=cell_class title=formatted>
                                                                        <div class="max-w-xs truncate">{formatted_clone}</div>
                                                                    </td>
                                                                }
                                                            }).collect::<Vec<_>>()}
                                                        </tr>
                                                    }
                                                }).collect::<Vec<_>>()}
                                            </tbody>
                                        </table>
                                    </div>
                                }.into_any()
                            }}
                        </div>
                    }
                })}

                // Empty state when no query has been run
                {move || {
                    if result.get().is_none() && error.get().is_none() {
                        view! {
                            <div class="bg-slate-800 rounded-lg flex-1 flex items-center justify-center">
                                <div class="text-center text-gray-500">
                                    <p class="text-lg mb-2">"Enter a SQL query and click Execute"</p>
                                    <p class="text-sm">"Click on a table name in the sidebar to insert a SELECT template"</p>
                                </div>
                            </div>
                        }.into_any()
                    } else {
                        view! { <div></div> }.into_any()
                    }
                }}
            </div>
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (page, set_page) = signal::<Page>(Page::BlockList);

    view! {
        <div class="min-h-screen bg-slate-950 text-slate-100">
            <header class="border-b border-slate-800 bg-slate-950/90 backdrop-blur">
                <div class="mx-auto max-w-screen-2xl px-4 md:px-6 py-4 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
                    <div class="flex items-baseline gap-3">
                        <h1 class="text-xl sm:text-2xl font-semibold tracking-tight">"Madara DB Visualizer"</h1>
                        <span class="hidden sm:inline text-sm text-slate-400">"RocksDB inspector"</span>
                    </div>
                    <SearchBar on_result=move |p| set_page.set(p) />
                </div>
            </header>

            <div class="mx-auto max-w-screen-2xl grid grid-cols-1 md:grid-cols-[18rem_1fr] gap-6 px-4 md:px-6 py-6">
                // Sidebar
                <aside class="rounded-2xl border border-slate-800 bg-slate-900/30 p-4 md:sticky md:top-6 h-fit">
                    <div class="space-y-3">
                        <StatsCard />
                        <IndexStatusCard />
                    </div>

                    <div class="mt-6">
                        <h3 class="text-xs font-semibold tracking-widest text-slate-400">"NAVIGATION"</h3>
                        <div class="mt-2 space-y-1">
                            <NavItem
                                label="Blocks"
                                active=matches!(page.get(), Page::BlockList | Page::BlockDetail { .. } | Page::TransactionDetail { .. } | Page::StateDiff { .. })
                                on_click=move || set_page.set(Page::BlockList)
                            />
                            <NavItem
                                label="Contracts"
                                active=matches!(page.get(), Page::ContractList | Page::ContractDetail { .. })
                                on_click=move || set_page.set(Page::ContractList)
                            />
                            <NavItem
                                label="Classes"
                                active=matches!(page.get(), Page::ClassList | Page::ClassDetail { .. })
                                on_click=move || set_page.set(Page::ClassList)
                            />
                            <NavItem
                                label="Advanced"
                                active=matches!(page.get(), Page::AdvancedFilters)
                                on_click=move || set_page.set(Page::AdvancedFilters)
                            />
                            <NavItem
                                label="Raw Data"
                                active=matches!(page.get(), Page::RawData | Page::RawKeyDetail { .. })
                                on_click=move || set_page.set(Page::RawData)
                            />
                            <NavItem
                                label="Schema"
                                active=matches!(page.get(), Page::Schema | Page::SchemaDetail { .. })
                                on_click=move || set_page.set(Page::Schema)
                            />
                            <NavItem
                                label="SQL Console"
                                active=matches!(page.get(), Page::SqlConsole)
                                on_click=move || set_page.set(Page::SqlConsole)
                            />
                        </div>
                    </div>
                </aside>

                // Main content
                <main class="min-w-0 rounded-2xl border border-slate-800 bg-slate-900/10 p-4 md:p-6">
                    {move || {
                        match page.get() {
                            Page::BlockList => view! {
                                <BlockList on_select=move |n| set_page.set(Page::BlockDetail { block_number: n }) />
                            }.into_any(),
                            Page::BlockDetail { block_number } => view! {
                                <BlockDetailView
                                    block_number=block_number
                                    on_back=move || set_page.set(Page::BlockList)
                                    on_tx_select=move |(bn, idx)| set_page.set(Page::TransactionDetail { block_number: bn, tx_index: idx })
                                    on_state_diff=move |bn| set_page.set(Page::StateDiff { block_number: bn })
                                />
                            }.into_any(),
                            Page::StateDiff { block_number } => view! {
                                <StateDiffView
                                    block_number=block_number
                                    on_back=move || set_page.set(Page::BlockDetail { block_number })
                                />
                            }.into_any(),
                            Page::TransactionDetail { block_number, tx_index } => view! {
                                <TransactionDetailView
                                    block_number=block_number
                                    tx_index=tx_index
                                    on_back=move || set_page.set(Page::BlockDetail { block_number })
                                />
                            }.into_any(),
                            Page::ContractList => view! {
                                <ContractList on_select=move |addr| set_page.set(Page::ContractDetail { address: addr }) />
                            }.into_any(),
                            Page::ContractDetail { address } => view! {
                                <ContractDetailView
                                    address=address.clone()
                                    on_back=move || set_page.set(Page::ContractList)
                                />
                            }.into_any(),
                            Page::ClassList => view! {
                                <ClassList on_select=move |hash| set_page.set(Page::ClassDetail { class_hash: hash }) />
                            }.into_any(),
                            Page::ClassDetail { class_hash } => view! {
                                <ClassDetailView
                                    class_hash=class_hash.clone()
                                    on_back=move || set_page.set(Page::ClassList)
                                />
                            }.into_any(),
                            Page::AdvancedFilters => view! {
                                <AdvancedFiltersView
                                    on_tx_select=move |(bn, idx)| set_page.set(Page::TransactionDetail { block_number: bn, tx_index: idx as usize })
                                />
                            }.into_any(),
                            Page::RawData => view! {
                                <RawDataView
                                    on_key_select=move |(cf, key)| set_page.set(Page::RawKeyDetail { cf_name: cf, key_hex: key })
                                />
                            }.into_any(),
                            Page::RawKeyDetail { cf_name, key_hex } => view! {
                                <RawKeyDetailView
                                    cf_name=cf_name.clone()
                                    key_hex=key_hex.clone()
                                    on_back=move || set_page.set(Page::RawData)
                                />
                            }.into_any(),
                            Page::Schema => view! {
                                <SchemaView
                                    on_cf_select=move |name| set_page.set(Page::SchemaDetail { cf_name: name })
                                />
                            }.into_any(),
                            Page::SchemaDetail { cf_name } => view! {
                                <SchemaDetailView
                                    cf_name=cf_name.clone()
                                    on_back=move || set_page.set(Page::Schema)
                                />
                            }.into_any(),
                            Page::SqlConsole => view! {
                                <SqlConsoleView />
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
