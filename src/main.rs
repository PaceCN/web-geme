use leptos::*;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, PointerEvent, Storage};

const STORAGE_KEY: &str = "huayuan_rust_web_demo_v1";

#[derive(Clone, Serialize, Deserialize)]
struct Player {
    user_id: String,
    nickname: String,
    level: i32,
    exp: i32,
    coins: i32,
    gems: i32,
    current_stamina: i32,
    max_stamina: i32,
    score_2048: i32,
    score_match3: i32,
    score_make10: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct InventoryItem {
    item_id: String,
    count: i32,
    name: String,
    item_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct Requirement {
    item_id: String,
    count: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Order {
    order_id: String,
    status: String,
    title: String,
    requirements: Vec<Requirement>,
    coins_reward: i32,
    exp_reward: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct Task {
    task_id: String,
    title: String,
    description: String,
    target_progress: i32,
    current_progress: i32,
    claimed: bool,
    task_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct GameState {
    player: Player,
    inventory: Vec<InventoryItem>,
    orders: Vec<Order>,
    tasks: Vec<Task>,
    transactions: Vec<String>,
}

#[derive(Clone)]
struct ShopItem {
    item_id: &'static str,
    name: &'static str,
    price: i32,
    item_type: &'static str,
    description: &'static str,
}

#[derive(Clone)]
struct BazaarListing {
    item_id: &'static str,
    qty: i32,
    price: i32,
    seller: &'static str,
}

#[derive(Clone)]
struct Hotspot {
    id: &'static str,
    name: &'static str,
    kind: &'static str,
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct Game2048 {
    board: Vec<i32>,
    score: i32,
    moves: i32,
    over: bool,
}

#[derive(Clone)]
enum Action {
    Purchase(String),
    Drink(String),
    SubmitOrder(String),
    ExecuteTask(String),
    ClaimTask(String),
    BuyBazaar(String, i32, i32),
    SellBazaar(String, i32, i32),
    FinishGame(String, i32),
}

#[derive(Clone, PartialEq)]
enum Tab {
    Orders,
    Chores,
    Backpack,
    Shop,
    Profile,
}

impl Tab {
    fn title(&self) -> &'static str {
        match self {
            Tab::Orders => "居民契约",
            Tab::Chores => "庄园内务",
            Tab::Backpack => "物资背包",
            Tab::Shop => "集市商店",
            Tab::Profile => "个人信息",
        }
    }

    fn icon(&self) -> &'static str {
        match self {
            Tab::Orders => "契",
            Tab::Chores => "务",
            Tab::Backpack => "仓",
            Tab::Shop => "市",
            Tab::Profile => "人",
        }
    }
}

fn main() {
    console_error_panic_hook::set_once();
    if let Some(root) = web_sys::window()
        .and_then(|window| window.document())
        .and_then(|document| document.get_element_by_id("app"))
    {
        root.set_inner_html("");
        mount_to(root, || view! { <App /> });
    } else {
        mount_to_body(|| view! { <App /> });
    }
}

#[component]
fn App() -> impl IntoView {
    let state = create_rw_signal(load_state().unwrap_or_else(seed_state));
    let active_tab = create_rw_signal(Tab::Orders);
    let chores_subtab = create_rw_signal(0_i32);
    let shop_subtab = create_rw_signal(0_i32);
    let selected_hotspot = create_rw_signal(String::new());
    let toast = create_rw_signal(String::new());
    let open_game = create_rw_signal(false);
    let game = create_rw_signal(Game2048::new());
    let drag_start = create_rw_signal::<Option<(f64, f64)>>(None);

    create_effect(move |_| {
        state.with(save_state);
    });

    let content = move || match active_tab.get() {
        Tab::Orders => orders_tab(state, toast),
        Tab::Chores => chores_tab(state, toast, chores_subtab, open_game),
        Tab::Backpack => backpack_tab(state, toast, active_tab),
        Tab::Shop => shop_tab(state, toast, shop_subtab),
        Tab::Profile => profile_tab(state, toast),
    };

    view! {
        <main class="app-shell">
            <TopStatus state=state />
            <section class="content-shell">
                <div class="tab-page">
                    {move || if active_tab.get() == Tab::Chores {
                        view! {
                            <div class="scene-card card">
                                <GardenCanvas state=state selected_hotspot=selected_hotspot active_tab=active_tab />
                            </div>
                        }.into_view()
                    } else {
                        view! {}.into_view()
                    }}
                    {content}
                </div>
            </section>
            <BottomTabs active_tab=active_tab />
        </main>

        {move || {
            let message = toast.get();
            if message.is_empty() {
                view! {}.into_view()
            } else {
                view! { <div class="toast">{message}</div> }.into_view()
            }
        }}

        {move || if open_game.get() {
            view! {
                <Game2048Overlay
                    state=state
                    toast=toast
                    open_game=open_game
                    game=game
                    drag_start=drag_start
                />
            }.into_view()
        } else {
            view! {}.into_view()
        }}
    }
}

#[component]
fn TopStatus(state: RwSignal<GameState>) -> impl IntoView {
    view! {
        <header class="top-status">
            <div class="top-main">
                <div class="avatar">"庄"</div>
                <div>
                    <h1>{move || state.with(|s| s.player.nickname.clone())}</h1>
                    <div class="status-sub">
                        {move || state.with(|s| format!("阅历: {} / {}", s.player.exp, s.player.level * 100))}
                    </div>
                    <span class="level-pill">{move || state.with(|s| format!("庄园等阶 {}", s.player.level))}</span>
                </div>
                <div class="coins">{move || state.with(|s| s.player.coins.to_string())}</div>
            </div>
            <div class="meter-grid">
                <div>
                    <div class="meter-title">
                        <span>"精力体力"</span>
                        <b>{move || state.with(|s| format!("{}/{}", s.player.current_stamina, s.player.max_stamina))}</b>
                    </div>
                    <div class="meter energy">
                        <i style=move || state.with(|s| format!("width:{}%", percent(s.player.current_stamina, s.player.max_stamina)))></i>
                    </div>
                </div>
                <div>
                    <div class="meter-title">
                        <span>"晋升经验"</span>
                        <b>{move || state.with(|s| format!("{}/{}", s.player.exp, s.player.level * 100))}</b>
                    </div>
                    <div class="meter">
                        <i style=move || state.with(|s| format!("width:{}%", percent(s.player.exp, s.player.level * 100)))></i>
                    </div>
                </div>
            </div>
        </header>
    }
}

#[component]
fn BottomTabs(active_tab: RwSignal<Tab>) -> impl IntoView {
    let tabs = vec![Tab::Orders, Tab::Chores, Tab::Backpack, Tab::Shop, Tab::Profile];
    view! {
        <nav class="bottom-tabs" aria-label="主导航">
            {tabs.into_iter().map(|tab| {
                let tab_for_click = tab.clone();
                let tab_for_class = tab.clone();
                view! {
                    <button
                        class=move || if active_tab.get() == tab_for_class { "nav-btn active" } else { "nav-btn" }
                        on:click=move |_| active_tab.set(tab_for_click.clone())
                        type="button"
                    >
                        <span>{tab.icon()}</span>
                        {tab.title()}
                    </button>
                }
            }).collect_view()}
        </nav>
    }
}

#[component]
fn GardenCanvas(
    state: RwSignal<GameState>,
    selected_hotspot: RwSignal<String>,
    active_tab: RwSignal<Tab>,
) -> impl IntoView {
    create_effect(move |_| {
        let selected = selected_hotspot.get();
        state.with(|s| draw_scene(s, &selected));
    });

    view! {
        <canvas
            id="garden-canvas"
            class="garden-canvas"
            on:pointerup=move |event: PointerEvent| {
                if let Some(canvas) = canvas_element() {
                    let rect = canvas.get_bounding_client_rect();
                    let x = (event.client_x() as f64 - rect.left()) / rect.width() * 390.0;
                    let y = (event.client_y() as f64 - rect.top()) / rect.height() * 520.0;
                    if let Some(hotspot) = hotspots().into_iter().find(|h| x >= h.x && x <= h.x + h.w && y >= h.y && y <= h.y + h.h) {
                        selected_hotspot.set(hotspot.id.to_string());
                        match hotspot.kind {
                            "order_board" => active_tab.set(Tab::Orders),
                            "inventory" => active_tab.set(Tab::Backpack),
                            "well" => active_tab.set(Tab::Shop),
                            _ => {}
                        }
                    }
                }
            }
        ></canvas>
    }
}

fn orders_tab(state: RwSignal<GameState>, toast: RwSignal<String>) -> View {
    view! {
        <div class="hero-card gold">
            <div class="hero-row">
                <div class="hero-icon">"契"</div>
                <div>
                    <h2 class="section-title">"「庄园公共契约告示栏」"</h2>
                    <p class="muted">"庄园周边集市货商马车已经备好。清扫并交付各种原材料，赚取佣金和阅历。"</p>
                </div>
            </div>
        </div>
        <div class="section-title">"正在派发中的契约货单"</div>
        <div class="card-list">
            {move || state.with(|s| {
                let mut orders = s.orders.clone();
                orders.sort_by_key(|o| o.status == "completed");
                orders.into_iter().map(|order| order_card(state, toast, order)).collect_view()
            })}
        </div>
    }.into_view()
}

fn order_card(state: RwSignal<GameState>, toast: RwSignal<String>, order: Order) -> View {
    let can_fulfill = state.with(|s| {
        order.requirements.iter().all(|req| inventory_count(s, &req.item_id) >= req.count)
    });
    let is_done = order.status == "completed";
    let order_id = order.order_id.clone();
    view! {
        <article class="order-card">
            <div class="order-head">
                <div class="round-icon">{if is_done { "✓" } else { "货" }}</div>
                <div>
                    <div class="name">{order.title.clone()}</div>
                    <p class="muted">{order_lore(&order.order_id)}</p>
                </div>
                <span class=if is_done { "tag gray" } else { "tag warn" }>{if is_done { "已交付" } else { "待交付" }}</span>
            </div>
            <div class="requirements">
                {order.requirements.clone().into_iter().map(move |req| {
                    let current = state.with(|s| inventory_count(s, &req.item_id));
                    let ok = current >= req.count;
                    view! {
                        <div class=if ok { "req-row ok" } else { "req-row miss" }>
                            <span>{format!("{} x {}", item_name(&req.item_id), req.count)}</span>
                            <b>{format!("现有: {}", current)}</b>
                        </div>
                    }
                }).collect_view()}
            </div>
            <div class="actions">
                <span class="tag warn">{format!("+{} 金币", order.coins_reward)}</span>
                <span class="tag">{format!("+{} EXP", order.exp_reward)}</span>
                {if !is_done {
                    let order_id_for_click = order_id.clone();
                    view! {
                        <button
                            class="btn"
                            disabled=!can_fulfill
                            on:click=move |_| run_action(state, toast, Action::SubmitOrder(order_id_for_click.clone()))
                            type="button"
                        >
                            {if can_fulfill { "交付货物" } else { "物料不足" }}
                        </button>
                    }.into_view()
                } else {
                    view! {}.into_view()
                }}
            </div>
        </article>
    }.into_view()
}

fn chores_tab(
    state: RwSignal<GameState>,
    toast: RwSignal<String>,
    subtab: RwSignal<i32>,
    open_game: RwSignal<bool>,
) -> View {
    view! {
        <div class="subtabs">
            <button class=move || if subtab.get() == 0 { "subtab active" } else { "subtab" } on:click=move |_| subtab.set(0) type="button">"打扫庄园"</button>
            <button class=move || if subtab.get() == 1 { "subtab active" } else { "subtab" } on:click=move |_| subtab.set(1) type="button">"玩法说明"</button>
        </div>
        {move || if subtab.get() == 0 {
            chores_work_tab(state, toast, open_game)
        } else {
            chores_help_tab()
        }}
    }.into_view()
}

fn chores_work_tab(state: RwSignal<GameState>, toast: RwSignal<String>, open_game: RwSignal<bool>) -> View {
    view! {
        <div class="hero-card green">
            <h2 class="section-title">"互动深度清洁秘境（趣味小游戏）"</h2>
            <p class="muted">"觉得精力值不够用了？挑战免费益智除扫小游戏，免消耗精力，大量掉落指定原材料。"</p>
        </div>
        <div class="card-list">
            <article class="card row-card">
                <div class="tile-icon">"2048"</div>
                <div>
                    <div class="name">"2048 扫除拼图"</div>
                    <p class="muted">"对应原材料：干枯杂草 / 金色落叶"</p>
                    <span class="tag warn">{move || state.with(|s| format!("历史最高分: {}", s.player.score_2048))}</span>
                </div>
                <button class="btn" on:click=move |_| open_game.set(true) type="button">"立即扫除"</button>
            </article>
            <article class="card row-card">
                <div class="tile-icon">"落"</div>
                <div>
                    <div class="name">"开心消消乐（扫叶扫蛛网）"</div>
                    <p class="muted">"对应原材料：金色落叶 / 坚韧蛛丝"</p>
                    <span class="tag warn">{move || state.with(|s| format!("历史最高分: {}", s.player.score_match3))}</span>
                </div>
                <button class="btn blue" on:click=move |_| run_action(state, toast, Action::FinishGame("match3".into(), 60)) type="button">"立即滑消"</button>
            </article>
            <article class="card row-card">
                <div class="tile-icon">"10"</div>
                <div>
                    <div class="name">"一不小心就到十（木柴合数）"</div>
                    <p class="muted">"对应原材料：修剪碎木 / 坚韧蛛丝"</p>
                    <span class="tag warn">{move || state.with(|s| format!("历史最高分: {}", s.player.score_make10))}</span>
                </div>
                <button class="btn orange" on:click=move |_| run_action(state, toast, Action::FinishGame("make10".into(), 50)) type="button">"立即清点"</button>
            </article>
        </div>
        <div class="section-title">"常规基础清扫委托"</div>
        <div class="card-list">
            {move || state.with(|s| {
                s.tasks.clone().into_iter().map(|task| task_card(state, toast, task)).collect_view()
            })}
        </div>
    }.into_view()
}

fn task_card(state: RwSignal<GameState>, toast: RwSignal<String>, task: Task) -> View {
    let task_id = task.task_id.clone();
    let stamina_cost = task_stamina(&task.task_type);
    let full = task.current_progress >= task.target_progress;
    let runnable = state.with(|s| s.player.current_stamina >= stamina_cost);
    view! {
        <article class="card">
            <div class="order-head">
                <div class="round-icon">{task_icon(&task.task_type)}</div>
                <div>
                    <div class="name">{task.title.clone()}</div>
                    <p class="muted">{task.description.clone()}</p>
                </div>
                <span class=if task.claimed { "tag gray" } else if full { "tag warn" } else { "tag" }>
                    {if task.claimed { "事务全成".to_string() } else if full { "可结业受赏".to_string() } else if stamina_cost == 0 { "免消耗".to_string() } else { format!("体力 -{}", stamina_cost) }}
                </span>
            </div>
            <div class="tag-row">
                <div class="progress" style="flex:1">
                    <i style=format!("width:{}%", percent(task.current_progress, task.target_progress))></i>
                </div>
                <b>{format!("{}/{}", task.current_progress, task.target_progress)}</b>
            </div>
            <div class="actions">
                {if task.claimed {
                    view! { <span class="muted">"契约圆满终结"</span> }.into_view()
                } else if full {
                    let task_for_click = task_id.clone();
                    view! { <button class="btn gold" on:click=move |_| run_action(state, toast, Action::ClaimTask(task_for_click.clone())) type="button">"领赏"</button> }.into_view()
                } else {
                    let task_for_click = task_id.clone();
                    view! {
                        <button class="btn" disabled=!runnable on:click=move |_| run_action(state, toast, Action::ExecuteTask(task_for_click.clone())) type="button">
                            {if task.task_type == "sign" { "签到" } else if runnable { "打扫" } else { "体力不足" }}
                        </button>
                    }.into_view()
                }}
            </div>
        </article>
    }.into_view()
}

fn chores_help_tab() -> View {
    view! {
        <article class="card">
            <h2 class="section-title">"见习主理人城堡打扫指引说明"</h2>
            <p class="muted">"1. 在居民契约界面，集齐原材料清单并交付，即可获得金币与 EXP。"</p>
            <p class="muted">"2. 基础清扫会扣减体力并稳定产出原材料；趣味小游戏用于免体力获得材料。"</p>
            <p class="muted">"3. 黄金神水和庄园能量饮料可在背包中服用，快速恢复体力。"</p>
            <p class="muted">"4. 玩家市集可低价买入散户材料，也可把背包原料挂牌，成交扣 10% 手续费。"</p>
        </article>
    }.into_view()
}

fn backpack_tab(state: RwSignal<GameState>, toast: RwSignal<String>, active_tab: RwSignal<Tab>) -> View {
    view! {
        <div class="hero-card gold">
            <div class="hero-row">
                <div class="hero-icon">"仓"</div>
                <div>
                    <h2 class="section-title">"庄园私人仓库储藏室"</h2>
                    <p class="muted">{move || state.with(|s| format!("已存入物料与契约卷：{} 份", s.inventory.iter().map(|i| i.count).sum::<i32>()))}</p>
                </div>
            </div>
        </div>
        {move || state.with(|s| {
            let items: Vec<_> = s.inventory.iter().filter(|item| item.count > 0).cloned().collect();
            if items.is_empty() {
                view! {
                    <article class="card">
                        <h2 class="section-title">"您的储藏室还空空如也"</h2>
                        <p class="muted">"背囊中没有原料，无法换取亮闪闪的金币。现在前去城堡清扫。"</p>
                        <div class="actions"><button class="btn" on:click=move |_| active_tab.set(Tab::Chores) type="button">"立刻干活清扫"</button></div>
                    </article>
                }.into_view()
            } else {
                view! {
                    <div class="inventory-grid">
                        {items.into_iter().map(|item| inventory_card(state, toast, item)).collect_view()}
                    </div>
                }.into_view()
            }
        })}
    }.into_view()
}

fn inventory_card(state: RwSignal<GameState>, toast: RwSignal<String>, item: InventoryItem) -> View {
    let item_id = item.item_id.clone();
    let usable = item.item_type == "elixir";
    view! {
        <article class="item-card">
            <div class="tile-icon" style="margin:0 auto">{item_icon(&item.item_id)}</div>
            <div class="name">{item.name.clone()}</div>
            <div class="count">{format!("储存: {} 份", item.count)}</div>
            <span class="tag">{item_kind_label(&item.item_type)}</span>
            <p class="muted">{item_definition(&item.item_id)}</p>
            {if usable {
                let item_for_click = item_id.clone();
                view! { <button class="btn blue" on:click=move |_| run_action(state, toast, Action::Drink(item_for_click.clone())) type="button">"一键温服"</button> }.into_view()
            } else {
                view! {}.into_view()
            }}
        </article>
    }.into_view()
}

fn shop_tab(state: RwSignal<GameState>, toast: RwSignal<String>, subtab: RwSignal<i32>) -> View {
    view! {
        <div class="shop-tabs">
            <button class=move || if subtab.get() == 0 { "subtab active" } else { "subtab" } on:click=move |_| subtab.set(0) type="button">"小镇商城"</button>
            <button class=move || if subtab.get() == 1 { "subtab active" } else { "subtab" } on:click=move |_| subtab.set(1) type="button">"玩家市集"</button>
        </div>
        {move || if subtab.get() == 0 { shop_official_tab(state, toast) } else { bazaar_tab(state, toast) }}
    }.into_view()
}

fn shop_official_tab(state: RwSignal<GameState>, toast: RwSignal<String>) -> View {
    view! {
        <div class="hero-card orange">
            <div class="hero-row">
                <div class="hero-icon">"店"</div>
                <div>
                    <h2 class="section-title">"「老皮特塞外杂货铺」"</h2>
                    <p class="muted">"打扫城堡极耗力气。金币充足的话，买几壶黄金圣水和快充药剂带上吧。"</p>
                </div>
            </div>
        </div>
        <div class="card-list">
            {shop_items().into_iter().map(|item| {
                let enough = state.with(|s| s.player.coins >= item.price);
                let item_id = item.item_id.to_string();
                view! {
                    <article class="card row-card">
                        <div class="tile-icon">{item_icon(item.item_id)}</div>
                        <div>
                            <div class="name">{item.name}</div>
                            <p class="muted">{item.description}</p>
                            <span class="tag warn">{promo_tag(item.item_id)}</span>
                        </div>
                        <button class="btn orange" disabled=!enough on:click=move |_| run_action(state, toast, Action::Purchase(item_id.clone())) type="button">
                            {if enough { format!("{} 买下", item.price) } else { "钱不够".to_string() }}
                        </button>
                    </article>
                }
            }).collect_view()}
        </div>
    }.into_view()
}

fn bazaar_tab(state: RwSignal<GameState>, toast: RwSignal<String>) -> View {
    view! {
        <div class="hero-card">
            <h2 class="section-title">"庄园玩家市集自由交易规则"</h2>
            <p class="muted">"散户寄售闲置原料。每笔卖单成交时，工会扣缴 10% 工本服务费。"</p>
        </div>
        <div class="section-title">"市集在售特价原材料"</div>
        <div class="card-list">
            {bazaar_listings().into_iter().map(|listing| {
                let affordable = state.with(|s| s.player.coins >= listing.price);
                let item_id = listing.item_id.to_string();
                view! {
                    <article class="card row-card">
                        <div class="tile-icon">{item_icon(listing.item_id)}</div>
                        <div>
                            <div class="name">{format!("{} x{}", item_name(listing.item_id), listing.qty)}</div>
                            <p class="muted">{format!("寄售人: {}", listing.seller)}</p>
                        </div>
                        <button class="btn blue" disabled=!affordable on:click=move |_| run_action(state, toast, Action::BuyBazaar(item_id.clone(), listing.qty, listing.price)) type="button">
                            {format!("{} 吃下", listing.price)}
                        </button>
                    </article>
                }
            }).collect_view()}
        </div>
        <div class="section-title">"我要上架出售背囊物品"</div>
        <div class="card-list">
            {move || state.with(|s| {
                let items: Vec<_> = s.inventory.iter().filter(|item| is_debris(&item.item_id) && item.count > 0).cloned().collect();
                if items.is_empty() {
                    view! { <article class="card"><p class="muted">"您的背包中暂时没有可供交易的庄园原材料。请先去庄园内务完成清扫打杂。"</p></article> }.into_view()
                } else {
                    view! {
                        <>
                            {items.into_iter().map(|item| {
                                let item_id = item.item_id.clone();
                                view! {
                                    <article class="card row-card">
                                        <div class="tile-icon">{item_icon(&item.item_id)}</div>
                                        <div>
                                            <div class="name">{item.name.clone()}</div>
                                            <p class="muted">{format!("背囊当前持有: {} 份", item.count)}</p>
                                            <span class="tag warn">"默认挂牌 1 份 / 15 金币，成交扣 10%"</span>
                                        </div>
                                        <button class="btn" on:click=move |_| run_action(state, toast, Action::SellBazaar(item_id.clone(), 1, 15)) type="button">"挂牌"</button>
                                    </article>
                                }
                            }).collect_view()}
                        </>
                    }.into_view()
                }
            })}
        </div>
    }.into_view()
}

fn profile_tab(state: RwSignal<GameState>, _toast: RwSignal<String>) -> View {
    view! {
        <article class="profile-card">
            <div class="hero-card green" style="box-shadow:none;border:0">
                <div class="hero-row">
                    <div class="hero-icon">"人"</div>
                    <div>
                        <h2 class="section-title">{move || state.with(|s| s.player.nickname.clone())}</h2>
                        <p class="muted">{move || state.with(|s| format!("庄园护照账号 ID: #{}", s.player.user_id.replace("u_", "")))}</p>
                        <span class="tag warn">{move || state.with(|s| format!("等级 {}", s.player.level))}</span>
                    </div>
                </div>
            </div>
        </article>
        <article class="profile-card card">
            <h2 class="section-title">"游戏模块生涯历史最高积分"</h2>
            <div class="card-list">
                <div class="row-card">
                    <div class="tile-icon">"2048"</div>
                    <div><div class="name">"2048 扫除拼图"</div><p class="muted">"对应: 干枯杂草 / 金色落叶"</p></div>
                    <b>{move || state.with(|s| s.player.score_2048)}</b>
                </div>
                <div class="row-card">
                    <div class="tile-icon">"落"</div>
                    <div><div class="name">"开心消消乐"</div><p class="muted">"对应: 金色落叶 / 坚韧蛛丝"</p></div>
                    <b>{move || state.with(|s| s.player.score_match3)}</b>
                </div>
                <div class="row-card">
                    <div class="tile-icon">"10"</div>
                    <div><div class="name">"一不小心就到十"</div><p class="muted">"对应: 修剪碎木 / 坚韧蛛丝"</p></div>
                    <b>{move || state.with(|s| s.player.score_make10)}</b>
                </div>
            </div>
        </article>
        <article class="profile-card card">
            <h2 class="section-title">"资产与进度总览"</h2>
            <div class="profile-stat-grid">
                <div class="profile-stat"><small>"当前硬币持有"</small><strong>{move || state.with(|s| format!("{} 金币", s.player.coins))}</strong></div>
                <div class="profile-stat"><small>"角色剩余精力"</small><strong>{move || state.with(|s| format!("{}/{}", s.player.current_stamina, s.player.max_stamina))}</strong></div>
            </div>
        </article>
    }.into_view()
}

#[component]
fn Game2048Overlay(
    state: RwSignal<GameState>,
    toast: RwSignal<String>,
    open_game: RwSignal<bool>,
    game: RwSignal<Game2048>,
    drag_start: RwSignal<Option<(f64, f64)>>,
) -> impl IntoView {
    view! {
        <div class="game-modal">
            <section class="game-panel">
                <div class="game-head">
                    <div>
                        <h2 class="section-title">"2048 扫除拼图"</h2>
                        <p class="muted">"合并方块，结算枯草和落叶"</p>
                    </div>
                    <button class="close" on:click=move |_| open_game.set(false) type="button">"×"</button>
                </div>
                <div class="score-row">
                    <div class="score-chip"><small>"本局"</small><strong>{move || game.with(|g| g.score)}</strong></div>
                    <div class="score-chip"><small>"最高"</small><strong>{move || state.with(|s| s.player.score_2048)}</strong></div>
                    <div class="score-chip"><small>"步数"</small><strong>{move || game.with(|g| g.moves)}</strong></div>
                </div>
                <div
                    class="board"
                    on:pointerdown=move |event: PointerEvent| drag_start.set(Some((event.client_x() as f64, event.client_y() as f64)))
                    on:pointerup=move |event: PointerEvent| {
                        if let Some((x, y)) = drag_start.get() {
                            let dx = event.client_x() as f64 - x;
                            let dy = event.client_y() as f64 - y;
                            drag_start.set(None);
                            if dx.abs().max(dy.abs()) > 26.0 {
                                let direction = if dx.abs() > dy.abs() {
                                    if dx > 0.0 { "right" } else { "left" }
                                } else if dy > 0.0 { "down" } else { "up" };
                                game.update(|g| g.move_dir(direction));
                            }
                        }
                    }
                >
                    {move || game.with(|g| {
                        g.board.iter().map(|value| {
                            let class = if *value == 2 || *value == 4 { "tile low" } else { "tile" };
                            view! {
                                <div class=class style=format!("background:{}", tile_color(*value))>
                                    {if *value == 0 { "".to_string() } else { value.to_string() }}
                                </div>
                            }
                        }).collect_view()
                    })}
                </div>
                <div class="actions">
                    <button class="btn secondary" on:click=move |_| game.set(Game2048::new()) type="button">"重开"</button>
                    <button
                        class="btn"
                        on:click=move |_| {
                            let score = game.with(|g| g.score);
                            open_game.set(false);
                            run_action(state, toast, Action::FinishGame("2048".into(), score));
                        }
                        type="button"
                    >
                        "结算奖励"
                    </button>
                </div>
            </section>
        </div>
    }
}

fn run_action(state: RwSignal<GameState>, toast: RwSignal<String>, action: Action) {
    let mut message = String::new();
    state.update(|s| {
        message = apply_action(s, action);
    });
    toast.set(message);
}

fn apply_action(state: &mut GameState, action: Action) -> String {
    match action {
        Action::Purchase(item_id) => {
            let Some(item) = shop_items().into_iter().find(|item| item.item_id == item_id) else {
                return "集市中暂时没有这件道具".into();
            };
            if state.player.coins < item.price {
                return format!("金币财富不足，购买需要 {} 金币。", item.price);
            }
            state.player.coins -= item.price;
            add_inventory(state, item.item_id, 1, item.name, item.item_type);
            progress_task(state, "purchase", 1);
            format!("购买成功，{} 已存入仓库。", item.name)
        }
        Action::Drink(item_id) => {
            let restore = match item_id.as_str() {
                "elixir_water" => 10,
                "potion_energy" => 20,
                _ => 0,
            };
            if restore == 0 {
                return "该物品无法直接消耗饮用".into();
            }
            if !remove_inventory(state, &item_id, 1) {
                return "仓库中缺少该道具".into();
            }
            state.player.current_stamina = (state.player.current_stamina + restore).min(state.player.max_stamina);
            format!("体力提升 {} 点。", restore)
        }
        Action::SubmitOrder(order_id) => {
            let Some(index) = state.orders.iter().position(|order| order.order_id == order_id) else {
                return "指定订单不存在".into();
            };
            if state.orders[index].status == "completed" {
                return "该派单契约已经结算。".into();
            }
            let reqs = state.orders[index].requirements.clone();
            for req in &reqs {
                if inventory_count(state, &req.item_id) < req.count {
                    return format!("缺少 {}。", item_name(&req.item_id));
                }
            }
            for req in &reqs {
                remove_inventory(state, &req.item_id, req.count);
            }
            let coins = state.orders[index].coins_reward;
            let exp = state.orders[index].exp_reward;
            state.orders[index].status = "completed".into();
            state.player.coins += coins;
            grant_exp(state, exp);
            progress_task(state, "order", 1);
            format!("交货结账成功，金币 +{}，经验 +{}。", coins, exp)
        }
        Action::ExecuteTask(task_id) => {
            let Some(index) = state.tasks.iter().position(|task| task.task_id == task_id) else {
                return "相应副业任务不存在".into();
            };
            if state.tasks[index].claimed {
                return "该杂役奖励已领取完了".into();
            }
            if state.tasks[index].current_progress >= state.tasks[index].target_progress {
                return "该杂役本轮额度已满，请领取奖励。".into();
            }
            let task_type = state.tasks[index].task_type.clone();
            match task_type.as_str() {
                "sign" => {
                    state.tasks[index].current_progress = 1;
                    state.player.coins += 50;
                    grant_exp(state, 15);
                    "每日签到成功，金币 +50，经验 +15。".into()
                }
                "weed" => execute_chore(state, index, 3, "debris_weed", "干枯杂草", "除草完成，干枯杂草 x2。"),
                "prune" => execute_chore(state, index, 4, "estate_wood", "修剪碎木", "修剪完成，修剪碎木 x2。"),
                "clean" => execute_chore(state, index, 3, "debris_leaves", "金色落叶", "清扫完成，金色落叶 x2。"),
                "web" => execute_chore(state, index, 4, "spider_silk", "坚韧蛛丝", "蛛网清理完成，坚韧蛛丝 x2。"),
                _ => "该任务暂时没有对应行动。".into(),
            }
        }
        Action::ClaimTask(task_id) => {
            let Some(index) = state.tasks.iter().position(|task| task.task_id == task_id) else {
                return "任务不存在".into();
            };
            if state.tasks[index].claimed {
                return "奖励已经领取过了".into();
            }
            if state.tasks[index].current_progress < state.tasks[index].target_progress {
                return "副务考核尚未通过。".into();
            }
            state.tasks[index].claimed = true;
            state.player.coins += 100;
            grant_exp(state, 25);
            "领取结业津贴，金币 +100，经验 +25。".into()
        }
        Action::BuyBazaar(item_id, count, price) => {
            if state.player.coins < price {
                return format!("金币不足，需要 {}。", price);
            }
            state.player.coins -= price;
            add_inventory(state, &item_id, count, item_name(&item_id), "debris");
            format!("交易达成，{} x{} 已收入背囊。", item_name(&item_id), count)
        }
        Action::SellBazaar(item_id, count, price) => {
            if !remove_inventory(state, &item_id, count) {
                return format!("{} 数量不足。", item_name(&item_id));
            }
            let fee = (price / 10).max(1);
            let gain = price - fee;
            state.player.coins += gain;
            format!("挂牌成交，扣除手续费 {}，净收 {} 金币。", fee, gain)
        }
        Action::FinishGame(game_type, score) => {
            match game_type.as_str() {
                "2048" => {
                    state.player.score_2048 = state.player.score_2048.max(score);
                    if score > 0 {
                        add_inventory(state, "debris_weed", 2, "干枯杂草", "debris");
                        add_inventory(state, "debris_leaves", 1, "金色落叶", "debris");
                    }
                    format!("2048 扫除完毕，获得 {} 分。", score)
                }
                "match3" => {
                    state.player.score_match3 = state.player.score_match3.max(score);
                    add_inventory(state, "debris_leaves", 2, "金色落叶", "debris");
                    add_inventory(state, "spider_silk", 1, "坚韧蛛丝", "debris");
                    format!("开心消消乐完成，获得 {} 分。", score)
                }
                "make10" => {
                    state.player.score_make10 = state.player.score_make10.max(score);
                    add_inventory(state, "estate_wood", 2, "修剪碎木", "debris");
                    add_inventory(state, "spider_silk", 1, "坚韧蛛丝", "debris");
                    format!("一不小心就到十完成，获得 {} 分。", score)
                }
                _ => "小游戏已结算。".into(),
            }
        }
    }
}

fn execute_chore(
    state: &mut GameState,
    task_index: usize,
    stamina: i32,
    item_id: &str,
    name: &str,
    message: &str,
) -> String {
    if state.player.current_stamina < stamina {
        return format!("体力不足，需要 {} 点。", stamina);
    }
    state.player.current_stamina -= stamina;
    state.tasks[task_index].current_progress += 1;
    add_inventory(state, item_id, 2, name, "debris");
    message.into()
}

fn seed_state() -> GameState {
    GameState {
        player: Player {
            user_id: "u_10001".into(),
            nickname: "庄园见习主理人".into(),
            level: 1,
            exp: 0,
            coins: 300,
            gems: 10,
            current_stamina: 30,
            max_stamina: 30,
            score_2048: 0,
            score_match3: 0,
            score_make10: 0,
        },
        inventory: vec![
            InventoryItem { item_id: "elixir_water".into(), count: 2, name: "黄金神水".into(), item_type: "elixir".into() },
            InventoryItem { item_id: "potion_energy".into(), count: 1, name: "庄园能量饮料".into(), item_type: "elixir".into() },
        ],
        orders: vec![
            order("order_001", "张爷爷的过冬干草委托", vec![req("debris_weed", 3), req("estate_wood", 1)], 120, 20),
            order("order_002", "苏菲博士的落叶堆肥", vec![req("debris_leaves", 3), req("debris_weed", 2)], 160, 30),
            order("order_003", "围栏工人的修枝木材需求", vec![req("estate_wood", 4)], 220, 40),
            order("order_004", "纺织坊老板的蛛丝契约", vec![req("spider_silk", 3)], 190, 35),
            order("order_005", "庄园伟业大建设计划", vec![req("estate_badge", 1), req("estate_wood", 5)], 450, 75),
        ],
        tasks: vec![
            task("task_001", "每日签到", "点击右侧签到按钮，获得 15 EXP 与 50 金币", 1, "sign"),
            task("task_002", "庄园除草", "清除庄园周边的野草，收集【干枯杂草】原料（消耗 3 体力）", 5, "weed"),
            task("task_003", "修剪杂枝", "修剪庄园里的杂乱枯枝，收集【修剪碎木】原料（消耗 4 体力）", 5, "prune"),
            task("task_004", "清扫落叶", "扫清庭院中飘落的枫叶，收集【金色落叶】原料（消耗 3 体力）", 5, "clean"),
            task("task_005", "清理蛛网", "清除门窗和回廊积累的厚重蛛网，收集【坚韧蛛丝】原料（消耗 4 体力）", 3, "web"),
        ],
        transactions: vec!["欢迎入驻本庄园！日常订单、建设打理副业玩法正式启动！".into()],
    }
}

fn order(id: &str, title: &str, requirements: Vec<Requirement>, coins: i32, exp: i32) -> Order {
    Order {
        order_id: id.into(),
        status: "available".into(),
        title: title.into(),
        requirements,
        coins_reward: coins,
        exp_reward: exp,
    }
}

fn req(item_id: &str, count: i32) -> Requirement {
    Requirement { item_id: item_id.into(), count }
}

fn task(id: &str, title: &str, description: &str, target: i32, task_type: &str) -> Task {
    Task {
        task_id: id.into(),
        title: title.into(),
        description: description.into(),
        target_progress: target,
        current_progress: 0,
        claimed: false,
        task_type: task_type.into(),
    }
}

fn shop_items() -> Vec<ShopItem> {
    vec![
        ShopItem { item_id: "elixir_water", name: "黄金神水", price: 50, item_type: "elixir", description: "服用后立即恢复 10 点体力" },
        ShopItem { item_id: "potion_energy", name: "庄园能量饮料", price: 90, item_type: "elixir", description: "服用后立即恢复 20 点体力" },
        ShopItem { item_id: "estate_badge", name: "庄园荣誉徽章", price: 150, item_type: "badge", description: "庄园建设的荣誉结晶，纪念收藏" },
    ]
}

fn bazaar_listings() -> Vec<BazaarListing> {
    vec![
        BazaarListing { item_id: "debris_weed", qty: 5, price: 45, seller: "庄园清荒客" },
        BazaarListing { item_id: "debris_leaves", qty: 4, price: 50, seller: "落叶收藏家" },
        BazaarListing { item_id: "estate_wood", qty: 3, price: 60, seller: "砍柴老王" },
        BazaarListing { item_id: "spider_silk", qty: 2, price: 60, seller: "荒宅扫除人" },
    ]
}

fn hotspots() -> Vec<Hotspot> {
    vec![
        Hotspot { id: "decor_shop", kind: "inventory", name: "主屋仓库", x: 10.0, y: 40.0, w: 120.0, h: 110.0 },
        Hotspot { id: "customer_001", kind: "customer", name: "神秘访客", x: 150.0, y: 120.0, w: 90.0, h: 110.0 },
        Hotspot { id: "order_board", kind: "order_board", name: "订单公告栏", x: 280.0, y: 140.0, w: 110.0, h: 110.0 },
        Hotspot { id: "well_001", kind: "well", name: "神奇泉水", x: 40.0, y: 160.0, w: 90.0, h: 95.0 },
        Hotspot { id: "plot_001", kind: "plot", name: "地块 1", x: 120.0, y: 280.0, w: 100.0, h: 100.0 },
        Hotspot { id: "plot_002", kind: "plot", name: "地块 2", x: 240.0, y: 280.0, w: 100.0, h: 100.0 },
        Hotspot { id: "plot_003", kind: "plot", name: "地块 3", x: 120.0, y: 400.0, w: 100.0, h: 100.0 },
        Hotspot { id: "plot_004", kind: "plot", name: "地块 4", x: 240.0, y: 400.0, w: 100.0, h: 100.0 },
    ]
}

impl Game2048 {
    fn new() -> Self {
        let mut game = Self { board: vec![0; 16], score: 0, moves: 0, over: false };
        game.add_random_tile();
        game.add_random_tile();
        game
    }

    fn add_random_tile(&mut self) {
        let empties: Vec<usize> = self
            .board
            .iter()
            .enumerate()
            .filter_map(|(index, value)| if *value == 0 { Some(index) } else { None })
            .collect();
        if empties.is_empty() {
            return;
        }
        let pick = (js_sys::Math::random() * empties.len() as f64).floor() as usize;
        self.board[empties[pick]] = if js_sys::Math::random() < 0.9 { 2 } else { 4 };
    }

    fn move_dir(&mut self, direction: &str) {
        if self.over {
            return;
        }
        let (next, gained) = match direction {
            "left" => move_left(&self.board),
            "right" => move_right(&self.board),
            "up" => {
                let transposed = transpose(&self.board);
                let (moved, score) = move_left(&transposed);
                (transpose(&moved), score)
            }
            "down" => {
                let transposed = transpose(&self.board);
                let (moved, score) = move_right(&transposed);
                (transpose(&moved), score)
            }
            _ => (self.board.clone(), 0),
        };
        if next != self.board {
            self.board = next;
            self.score += gained;
            self.moves += 1;
            self.add_random_tile();
            self.over = is_game_over(&self.board);
        }
    }
}

fn move_left(board: &[i32]) -> (Vec<i32>, i32) {
    let mut next = Vec::with_capacity(16);
    let mut score = 0;
    for row in 0..4 {
        let (line, gained) = slide_left(&board[row * 4..row * 4 + 4]);
        next.extend(line);
        score += gained;
    }
    (next, score)
}

fn move_right(board: &[i32]) -> (Vec<i32>, i32) {
    let mut next = Vec::with_capacity(16);
    let mut score = 0;
    for row in 0..4 {
        let mut line = board[row * 4..row * 4 + 4].to_vec();
        line.reverse();
        let (mut moved, gained) = slide_left(&line);
        moved.reverse();
        next.extend(moved);
        score += gained;
    }
    (next, score)
}

fn slide_left(row: &[i32]) -> (Vec<i32>, i32) {
    let non_zero: Vec<i32> = row.iter().copied().filter(|value| *value != 0).collect();
    let mut result = Vec::new();
    let mut score = 0;
    let mut i = 0;
    while i < non_zero.len() {
        if i + 1 < non_zero.len() && non_zero[i] == non_zero[i + 1] {
            let merged = non_zero[i] * 2;
            result.push(merged);
            score += merged;
            i += 2;
        } else {
            result.push(non_zero[i]);
            i += 1;
        }
    }
    while result.len() < 4 {
        result.push(0);
    }
    (result, score)
}

fn transpose(board: &[i32]) -> Vec<i32> {
    let mut next = vec![0; 16];
    for r in 0..4 {
        for c in 0..4 {
            next[c * 4 + r] = board[r * 4 + c];
        }
    }
    next
}

fn is_game_over(board: &[i32]) -> bool {
    if board.iter().any(|value| *value == 0) {
        return false;
    }
    for r in 0..4 {
        for c in 0..3 {
            if board[r * 4 + c] == board[r * 4 + c + 1] {
                return false;
            }
        }
    }
    for c in 0..4 {
        for r in 0..3 {
            if board[r * 4 + c] == board[(r + 1) * 4 + c] {
                return false;
            }
        }
    }
    true
}

fn draw_scene(_state: &GameState, selected: &str) {
    let Some(canvas) = canvas_element() else { return; };
    let width = canvas.client_width().max(1) as f64;
    let height = canvas.client_height().max(1) as f64;
    let dpr = web_sys::window().map(|w| w.device_pixel_ratio()).unwrap_or(1.0).min(2.0);
    canvas.set_width((width * dpr) as u32);
    canvas.set_height((height * dpr) as u32);
    let Some(context) = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .and_then(|value| value.dyn_into::<CanvasRenderingContext2d>().ok())
    else {
        return;
    };
    let _ = context.set_transform(dpr, 0.0, 0.0, dpr, 0.0, 0.0);
    context.set_fill_style(&JsValue::from_str("#b9d8ac"));
    context.fill_rect(0.0, 0.0, width, height);
    context.set_fill_style(&JsValue::from_str("#d5c099"));
    context.begin_path();
    context.move_to(width * 0.45, 0.0);
    context.quadratic_curve_to(width * 0.58, height * 0.45, width * 0.5, height);
    context.line_to(width * 0.66, height);
    context.quadratic_curve_to(width * 0.68, height * 0.45, width * 0.56, 0.0);
    context.close_path();
    context.fill();

    for hotspot in hotspots() {
        let x = hotspot.x / 390.0 * width;
        let y = hotspot.y / 520.0 * height;
        let w = hotspot.w / 390.0 * width;
        let h = hotspot.h / 520.0 * height;
        let color = if selected == hotspot.id { "#f4c66d" } else { match hotspot.kind {
            "plot" => "#75543d",
            "well" => "#526f84",
            "order_board" => "#8a5a32",
            "inventory" => "#6d8b55",
            "customer" => "#51455d",
            _ => "#777777",
        }};
        round_rect(&context, x, y, w, h, 8.0, color);
        context.set_fill_style(&JsValue::from_str("#fff8df"));
        let _ = context.fill_text(hotspot.name, x, y + h + 12.0);
    }
}

fn canvas_element() -> Option<HtmlCanvasElement> {
    web_sys::window()?
        .document()?
        .get_element_by_id("garden-canvas")?
        .dyn_into::<HtmlCanvasElement>()
        .ok()
}

fn round_rect(context: &CanvasRenderingContext2d, x: f64, y: f64, w: f64, h: f64, r: f64, color: &str) {
    context.set_fill_style(&JsValue::from_str(color));
    context.begin_path();
    context.move_to(x + r, y);
    context.line_to(x + w - r, y);
    context.quadratic_curve_to(x + w, y, x + w, y + r);
    context.line_to(x + w, y + h - r);
    context.quadratic_curve_to(x + w, y + h, x + w - r, y + h);
    context.line_to(x + r, y + h);
    context.quadratic_curve_to(x, y + h, x, y + h - r);
    context.line_to(x, y + r);
    context.quadratic_curve_to(x, y, x + r, y);
    context.fill();
}

fn storage() -> Option<Storage> {
    web_sys::window()?.local_storage().ok().flatten()
}

fn load_state() -> Option<GameState> {
    let value = storage()?.get_item(STORAGE_KEY).ok().flatten()?;
    serde_json::from_str(&value).ok()
}

fn save_state(state: &GameState) {
    if let Some(storage) = storage() {
        if let Ok(value) = serde_json::to_string(state) {
            let _ = storage.set_item(STORAGE_KEY, &value);
        }
    }
}

fn add_inventory(state: &mut GameState, item_id: &str, count: i32, name: &str, item_type: &str) {
    if let Some(item) = state.inventory.iter_mut().find(|item| item.item_id == item_id) {
        item.count += count;
    } else {
        state.inventory.push(InventoryItem {
            item_id: item_id.into(),
            count,
            name: name.into(),
            item_type: item_type.into(),
        });
    }
}

fn remove_inventory(state: &mut GameState, item_id: &str, count: i32) -> bool {
    if let Some(item) = state.inventory.iter_mut().find(|item| item.item_id == item_id) {
        if item.count >= count {
            item.count -= count;
            return true;
        }
    }
    false
}

fn inventory_count(state: &GameState, item_id: &str) -> i32 {
    state.inventory.iter().find(|item| item.item_id == item_id).map(|item| item.count).unwrap_or(0)
}

fn progress_task(state: &mut GameState, task_type: &str, amount: i32) {
    for task in &mut state.tasks {
        if task.task_type == task_type && !task.claimed {
            task.current_progress = (task.current_progress + amount).min(task.target_progress);
        }
    }
}

fn grant_exp(state: &mut GameState, amount: i32) {
    state.player.exp += amount;
    let required = state.player.level * 100;
    if state.player.exp >= required {
        state.player.exp -= required;
        state.player.level += 1;
        state.player.max_stamina = (state.player.level * 5 + 30).min(100);
        state.player.current_stamina = state.player.max_stamina;
    }
}

fn percent(value: i32, max: i32) -> i32 {
    if max <= 0 { 0 } else { ((value * 100) / max).clamp(0, 100) }
}

fn item_name(item_id: &str) -> &'static str {
    match item_id {
        "elixir_water" => "黄金神水",
        "potion_energy" => "庄园能量饮料",
        "estate_badge" => "庄园荣誉徽章",
        "debris_weed" => "干枯杂草",
        "debris_leaves" => "金色落叶",
        "estate_wood" => "修剪碎木",
        "spider_silk" => "坚韧蛛丝",
        _ => "未知物品",
    }
}

fn item_icon(item_id: &str) -> &'static str {
    match item_id {
        "elixir_water" => "水",
        "potion_energy" => "能",
        "estate_badge" => "章",
        "debris_weed" => "草",
        "debris_leaves" => "叶",
        "estate_wood" => "木",
        "spider_silk" => "丝",
        _ => "物",
    }
}

fn item_kind_label(kind: &str) -> &'static str {
    match kind {
        "elixir" => "城堡药剂",
        "badge" => "名誉勋章",
        "debris" => "仓储废料",
        _ => "庄园原料",
    }
}

fn item_definition(item_id: &str) -> &'static str {
    match item_id {
        "elixir_water" => "皇家黄金泉水，服用后精力提升 10 点。",
        "potion_energy" => "圣树嫩叶秘制能量魔药，服用后体力提升 20 点。",
        "estate_badge" => "表彰庄园打扫工作的名誉大勋章。",
        "debris_weed" => "勤恳除草所得的干枯杂草。",
        "debris_leaves" => "枫树下清扫得到的高营养金色落叶。",
        "estate_wood" => "修剪粗壮枝桠所得的干柴木段。",
        "spider_silk" => "老蛛网里清扫而得的银白蛛丝。",
        _ => "庄园原料。",
    }
}

fn order_lore(id: &str) -> &'static str {
    match id {
        "order_001" => "张爷爷急需干枯野草填充旧瓦盆垫。",
        "order_002" => "苏菲博士正在试验古代土壤复活发酵大法。",
        "order_003" => "外墙围栏垮塌，急需高韧性的修剪木桩。",
        "order_004" => "纺织坊老板需要高强度蜘蛛丝编织绸伞。",
        "order_005" => "镇长皮特签署了庄园荣誉功勋大建设案。",
        _ => "庄园领主日常建设清单中的民情大单。",
    }
}

fn task_icon(kind: &str) -> &'static str {
    match kind {
        "sign" => "签",
        "weed" => "草",
        "prune" => "木",
        "clean" => "叶",
        "web" => "丝",
        _ => "务",
    }
}

fn task_stamina(kind: &str) -> i32 {
    match kind {
        "weed" | "clean" => 3,
        "prune" | "web" => 4,
        _ => 0,
    }
}

fn promo_tag(item_id: &str) -> &'static str {
    match item_id {
        "elixir_water" => "皇家极品",
        "potion_energy" => "瞬间满充",
        "estate_badge" => "建功大勋章",
        _ => "精选新品",
    }
}

fn is_debris(item_id: &str) -> bool {
    matches!(item_id, "debris_weed" | "debris_leaves" | "estate_wood" | "spider_silk")
}

fn tile_color(value: i32) -> &'static str {
    match value {
        0 => "#cdc4ba",
        2 => "#f7f0e7",
        4 => "#ede0c8",
        8 => "#f2b179",
        16 => "#f59563",
        32 => "#f67c5f",
        64 => "#f65e3b",
        128 => "#edcf72",
        256 => "#edcc61",
        512 => "#4f8f76",
        1024 => "#4b73a8",
        2048 => "#3f3a54",
        _ => "#2f2a3d",
    }
}
