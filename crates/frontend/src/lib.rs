use leptos::prelude::*;
use visualizer_types::{
    BlockDetail, BlockListResponse, BlockSummary, ClassListResponse, ClassResponse,
    ContractListResponse, ContractResponse, ContractStorageResponse, HealthResponse,
    SearchResponse, StateDiffResponse, StatsResponse, TransactionDetail,
    TransactionListResponse, TransactionSummary,
};

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

async fn fetch_block_transactions(block_number: u64) -> Result<TransactionListResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/blocks/{block_number}/transactions"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_transaction(block_number: u64, tx_index: usize) -> Result<TransactionDetail, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/blocks/{block_number}/transactions/{tx_index}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_contracts(limit: usize) -> Result<ContractListResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/contracts?limit={limit}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_contract(address: String) -> Result<ContractResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/contracts/{address}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_contract_storage(address: String, limit: usize) -> Result<ContractStorageResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/contracts/{address}/storage?limit={limit}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_classes(limit: usize) -> Result<ClassListResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/classes?limit={limit}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_class(class_hash: String) -> Result<ClassResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/classes/{class_hash}"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_state_diff(block_number: u64) -> Result<StateDiffResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/blocks/{block_number}/state-diff"))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

async fn fetch_search(query: String) -> Result<SearchResponse, String> {
    gloo_net::http::Request::get(&format!("{API_BASE}/api/search?q={}", urlencoding::encode(&query)))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())
}

fn truncate_hash(hash: &str) -> String {
    if hash.len() > 16 {
        format!("{}...{}", &hash[..10], &hash[hash.len()-6..])
    } else {
        hash.to_string()
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
    let transactions = LocalResource::new(move || async move { fetch_block_transactions(block_number).await });
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
                                let parent_hash = b.parent_hash.clone();
                                let state_root = b.state_root.clone();
                                let sequencer = b.sequencer_address.clone();
                                let tx_count = b.transaction_count;
                                let event_count = b.event_count;
                                let gas_used = b.l2_gas_used;

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
    let tx = LocalResource::new(move || async move { fetch_transaction(block_number, tx_index).await });

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
                                let tx_type = t.tx_type.clone();
                                let status = t.status.clone();
                                let revert_reason = t.revert_reason.clone();
                                let block_num = t.block_number;
                                let idx = t.tx_index;
                                let actual_fee = t.actual_fee.clone();
                                let fee_unit = t.fee_unit.clone();
                                let sender = t.sender_address.clone();
                                let nonce = t.nonce.clone();
                                let version = t.version.clone();
                                let calldata = t.calldata.clone();
                                let signature = t.signature.clone();
                                let events = t.events.clone();

                                let status_class = if status == "SUCCEEDED" {
                                    "text-green-400"
                                } else {
                                    "text-red-400"
                                };

                                view! {
                                    <div>
                                        <h2 class="text-2xl font-bold mb-4">"Transaction"</h2>
                                        <div class="grid grid-cols-2 gap-4 mb-6">
                                            <div class="col-span-2">
                                                <p class="text-gray-400">"Transaction Hash"</p>
                                                <p class="font-mono text-sm break-all text-blue-400">{tx_hash}</p>
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
                                            {sender.map(|s| view! {
                                                <div class="col-span-2">
                                                    <p class="text-gray-400">"Sender Address"</p>
                                                    <p class="font-mono text-sm break-all">{s}</p>
                                                </div>
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
fn ContractRow(
    contract: ContractResponse,
    on_click: impl Fn(String) + 'static,
) -> impl IntoView {
    let address = contract.address.clone();
    let address_for_click = address.clone();
    let class_hash = contract.class_hash.clone().unwrap_or_else(|| "None".to_string());
    let nonce = contract.nonce.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string());

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
fn ContractDetailView(
    address: String,
    on_back: impl Fn() + 'static,
) -> impl IntoView {
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
fn ClassRow(
    class: ClassResponse,
    on_click: impl Fn(String) + 'static,
) -> impl IntoView {
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
fn ClassDetailView(
    class_hash: String,
    on_back: impl Fn() + 'static,
) -> impl IntoView {
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
fn StateDiffView(
    block_number: u64,
    on_back: impl Fn() + 'static,
) -> impl IntoView {
    let state_diff = LocalResource::new(move || async move { fetch_state_diff(block_number).await });

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
fn NavItem(
    label: &'static str,
    active: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let class = if active {
        "px-4 py-2 text-left hover:bg-gray-700 rounded text-blue-400 bg-gray-700"
    } else {
        "px-4 py-2 text-left hover:bg-gray-700 rounded text-gray-300"
    };

    view! {
        <button
            class=class
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
                                if let (Some(bn), Some(idx)) = (result.block_number, result.tx_index) {
                                    on_result(Page::TransactionDetail { block_number: bn, tx_index: idx as usize });
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
        <div class="flex items-center gap-2">
            <input
                type="text"
                placeholder="Search block, tx hash, contract..."
                class="px-3 py-2 bg-gray-700 rounded text-sm w-80 focus:outline-none focus:ring-2 focus:ring-blue-500"
                prop:value=move || query.get()
                on:input=move |ev| set_query.set(event_target_value(&ev))
                on:keypress=move |ev| {
                    if ev.key() == "Enter" {
                        set_trigger.update(|t| *t = t.wrapping_add(1));
                    }
                }
            />
            <button
                class="px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded text-sm disabled:opacity-50"
                disabled=move || searching.get()
                on:click=move |_| set_trigger.update(|t| *t = t.wrapping_add(1))
            >
                {move || if searching.get() { "..." } else { "Search" }}
            </button>
            {move || error.get().map(|e| view! {
                <span class="text-red-400 text-sm">{e}</span>
            })}
        </div>
    }
}

#[component]
fn App() -> impl IntoView {
    let (page, set_page) = signal::<Page>(Page::BlockList);

    view! {
        <div class="min-h-screen bg-gray-900 text-white">
            <header class="bg-gray-800 border-b border-gray-700 px-6 py-4 flex justify-between items-center">
                <h1 class="text-2xl font-bold">"Madara DB Visualizer"</h1>
                <SearchBar on_result=move |p| set_page.set(p) />
            </header>

            <div class="flex">
                // Sidebar
                <aside class="w-64 bg-gray-800 border-r border-gray-700 p-4 min-h-screen">
                    <StatsCard />

                    <div class="mt-6">
                        <h3 class="text-sm font-semibold text-gray-400 mb-2">"NAVIGATION"</h3>
                        <div class="flex flex-col space-y-1">
                            <NavItem
                                label="Blocks"
                                active=matches!(page.get(), Page::BlockList | Page::BlockDetail { .. } | Page::TransactionDetail { .. })
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
                        </div>
                    </div>
                </aside>

                // Main content
                <main class="flex-1 p-6">
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
