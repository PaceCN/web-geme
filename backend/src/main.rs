use axum::{
    extract::{Path, State},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<BackendState>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct BackendState {
    players: HashMap<String, PlayerAdminRecord>,
    ad_policy: AdRewardPolicy,
    audit_logs: Vec<AuditLog>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PlayerAdminRecord {
    user_id: String,
    nickname: String,
    level: i32,
    current_stamina: i32,
    max_stamina: i32,
    ad_claim_date: String,
    ad_claims_today: i32,
    is_banned: bool,
    last_active_at: u64,
}

#[derive(Clone, Serialize, Deserialize)]
struct AdRewardPolicy {
    placement_id: String,
    enabled: bool,
    stamina_reward: i32,
    daily_limit: i32,
    cooldown_seconds: i32,
}

#[derive(Clone, Serialize, Deserialize)]
struct AuditLog {
    id: u64,
    at: u64,
    actor: String,
    event: String,
    detail: String,
}

#[derive(Clone, Serialize)]
struct AdminDashboard {
    players_total: usize,
    ad_reward_enabled: bool,
    stamina_reward: i32,
    daily_limit: i32,
    ad_claims_today: i32,
    latest_logs: Vec<AuditLog>,
}

#[derive(Deserialize)]
struct AdRewardRequest {
    user_id: String,
    placement_id: String,
    ad_network: String,
    proof_token: String,
}

#[derive(Serialize)]
struct AdRewardResponse {
    accepted: bool,
    message: String,
    current_stamina: i32,
    max_stamina: i32,
    claims_left_today: i32,
}

#[derive(Deserialize)]
struct StaminaGrantRequest {
    amount: i32,
    reason: String,
    operator: String,
}

#[tokio::main]
async fn main() {
    let state = AppState {
        inner: Arc::new(RwLock::new(seed_backend_state())),
    };

    let app = Router::new()
        .route("/", get(admin_page))
        .route("/admin", get(admin_page))
        .route("/health", get(health))
        .route("/api/admin/dashboard", get(admin_dashboard))
        .route("/api/admin/ad-policy", get(get_ad_policy).put(update_ad_policy))
        .route("/api/admin/players", get(list_players))
        .route("/api/admin/players/:user_id/stamina", post(grant_stamina))
        .route("/api/v1/ads/reward", post(claim_ad_reward))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], 8787));
    let listener = tokio::net::TcpListener::bind(addr).await.expect("bind backend");
    println!("huayuan backend listening on http://{addr}");
    axum::serve(listener, app).await.expect("serve backend");
}

async fn admin_page() -> Html<&'static str> {
    Html(ADMIN_HTML)
}

async fn health() -> &'static str {
    "ok"
}

async fn admin_dashboard(State(state): State<AppState>) -> Json<AdminDashboard> {
    let store = state.inner.read().expect("state read");
    let today = today_key();
    let ad_claims_today = store
        .players
        .values()
        .filter(|player| player.ad_claim_date == today)
        .map(|player| player.ad_claims_today)
        .sum();
    Json(AdminDashboard {
        players_total: store.players.len(),
        ad_reward_enabled: store.ad_policy.enabled,
        stamina_reward: store.ad_policy.stamina_reward,
        daily_limit: store.ad_policy.daily_limit,
        ad_claims_today,
        latest_logs: store.audit_logs.iter().rev().take(20).cloned().collect(),
    })
}

async fn get_ad_policy(State(state): State<AppState>) -> Json<AdRewardPolicy> {
    Json(state.inner.read().expect("state read").ad_policy.clone())
}

async fn update_ad_policy(
    State(state): State<AppState>,
    Json(policy): Json<AdRewardPolicy>,
) -> Json<AdRewardPolicy> {
    let mut store = state.inner.write().expect("state write");
    store.ad_policy = policy.clone();
    push_log(&mut store, "admin", "ad_policy.update", "更新广告恢复体力配置");
    Json(policy)
}

async fn list_players(State(state): State<AppState>) -> Json<Vec<PlayerAdminRecord>> {
    Json(state.inner.read().expect("state read").players.values().cloned().collect())
}

async fn grant_stamina(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<StaminaGrantRequest>,
) -> Json<AdRewardResponse> {
    let mut store = state.inner.write().expect("state write");
    let daily_limit = store.ad_policy.daily_limit;
    let Some(player) = store.players.get_mut(&user_id) else {
        return Json(AdRewardResponse::not_found());
    };
    let amount = request.amount.max(0);
    player.current_stamina = (player.current_stamina + amount).min(player.max_stamina);
    let response = AdRewardResponse {
        accepted: true,
        message: format!("后台补发体力 {} 点：{}", amount, request.reason),
        current_stamina: player.current_stamina,
        max_stamina: player.max_stamina,
        claims_left_today: (daily_limit - player.ad_claims_today).max(0),
    };
    push_log(
        &mut store,
        &request.operator,
        "player.stamina.grant",
        &format!("{} 补发体力 {}", user_id, amount),
    );
    Json(response)
}

async fn claim_ad_reward(
    State(state): State<AppState>,
    Json(request): Json<AdRewardRequest>,
) -> Json<AdRewardResponse> {
    let mut store = state.inner.write().expect("state write");
    if !store.ad_policy.enabled {
        return Json(AdRewardResponse::rejected("广告恢复体力功能已关闭"));
    }
    if request.placement_id != store.ad_policy.placement_id {
        return Json(AdRewardResponse::rejected("广告位不匹配"));
    }
    if request.proof_token.trim().is_empty() {
        return Json(AdRewardResponse::rejected("缺少广告平台回调凭证"));
    }

    let policy = store.ad_policy.clone();
    let today = today_key();
    let Some(player) = store.players.get_mut(&request.user_id) else {
        return Json(AdRewardResponse::not_found());
    };
    if player.is_banned {
        return Json(AdRewardResponse::rejected("账号已被后台限制"));
    }
    if player.ad_claim_date != today {
        player.ad_claim_date = today;
        player.ad_claims_today = 0;
    }
    if player.current_stamina >= player.max_stamina {
        return Json(AdRewardResponse::rejected("体力已满"));
    }
    if player.ad_claims_today >= policy.daily_limit {
        return Json(AdRewardResponse::rejected("今日广告奖励次数已用完"));
    }

    player.current_stamina = (player.current_stamina + policy.stamina_reward).min(player.max_stamina);
    player.ad_claims_today += 1;
    player.last_active_at = unix_now();
    let response = AdRewardResponse {
        accepted: true,
        message: format!("{} 广告奖励已发放", request.ad_network),
        current_stamina: player.current_stamina,
        max_stamina: player.max_stamina,
        claims_left_today: (policy.daily_limit - player.ad_claims_today).max(0),
    };
    push_log(
        &mut store,
        "ad-network",
        "ad.reward.claim",
        &format!("{} 领取广告体力奖励", request.user_id),
    );
    Json(response)
}

impl AdRewardResponse {
    fn rejected(message: &str) -> Self {
        Self {
            accepted: false,
            message: message.into(),
            current_stamina: 0,
            max_stamina: 0,
            claims_left_today: 0,
        }
    }

    fn not_found() -> Self {
        Self::rejected("玩家不存在")
    }
}

fn seed_backend_state() -> BackendState {
    let mut players = HashMap::new();
    players.insert(
        "u_10001".into(),
        PlayerAdminRecord {
            user_id: "u_10001".into(),
            nickname: "庄园见习主理人".into(),
            level: 1,
            current_stamina: 12,
            max_stamina: 30,
            ad_claim_date: today_key(),
            ad_claims_today: 0,
            is_banned: false,
            last_active_at: unix_now(),
        },
    );
    BackendState {
        players,
        ad_policy: AdRewardPolicy {
            placement_id: "reward_stamina".into(),
            enabled: true,
            stamina_reward: 8,
            daily_limit: 6,
            cooldown_seconds: 45,
        },
        audit_logs: Vec::new(),
    }
}

fn push_log(store: &mut BackendState, actor: &str, event: &str, detail: &str) {
    let id = store.audit_logs.last().map(|log| log.id + 1).unwrap_or(1);
    store.audit_logs.push(AuditLog {
        id,
        at: unix_now(),
        actor: actor.into(),
        event: event.into(),
        detail: detail.into(),
    });
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn today_key() -> String {
    format!("day-{}", unix_now() / 86_400)
}

const ADMIN_HTML: &str = r#"<!doctype html>
<html lang="zh-CN">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width,initial-scale=1">
  <title>庄园物语运营后台</title>
  <style>
    body{margin:0;background:#101820;color:#f5f7ed;font-family:system-ui,"Microsoft YaHei",sans-serif}
    main{max-width:960px;margin:0 auto;padding:24px;display:grid;gap:16px}
    section{border:1px solid rgba(195,227,174,.18);border-radius:8px;background:#17242d;padding:16px}
    h1,h2{margin:0 0 12px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px}
    input,button{height:38px;border-radius:8px;border:1px solid rgba(255,255,255,.12);padding:0 10px}
    input{background:#0d161d;color:#fff}button{background:#65b86a;color:#07100a;font-weight:800;cursor:pointer}
    pre{white-space:pre-wrap;color:#c8d7c3}
  </style>
</head>
<body>
<main>
  <h1>庄园物语运营后台</h1>
  <section><h2>数据看板</h2><div id="dashboard" class="grid"></div></section>
  <section>
    <h2>广告恢复体力配置</h2>
    <div class="grid">
      <input id="reward" placeholder="单次恢复体力">
      <input id="limit" placeholder="每日次数">
      <input id="cooldown" placeholder="冷却秒数">
      <button onclick="savePolicy()">保存配置</button>
    </div>
  </section>
  <section>
    <h2>玩家体力补发</h2>
    <div class="grid">
      <input id="uid" value="u_10001">
      <input id="amount" value="8">
      <input id="reason" value="运营补偿">
      <button onclick="grant()">补发体力</button>
    </div>
  </section>
  <section><h2>日志</h2><pre id="logs"></pre></section>
</main>
<script>
async function load(){
  const dash=await fetch('/api/admin/dashboard').then(r=>r.json());
  document.getElementById('dashboard').innerHTML=[
    ['玩家数',dash.players_total],['广告开关',dash.ad_reward_enabled?'开启':'关闭'],
    ['单次体力',dash.stamina_reward],['每日次数',dash.daily_limit],['今日广告领取',dash.ad_claims_today]
  ].map(([k,v])=>`<section><b>${k}</b><p>${v}</p></section>`).join('');
  document.getElementById('logs').textContent=JSON.stringify(dash.latest_logs,null,2);
  const policy=await fetch('/api/admin/ad-policy').then(r=>r.json());
  reward.value=policy.stamina_reward; limit.value=policy.daily_limit; cooldown.value=policy.cooldown_seconds;
}
async function savePolicy(){
  await fetch('/api/admin/ad-policy',{method:'PUT',headers:{'content-type':'application/json'},body:JSON.stringify({
    placement_id:'reward_stamina',enabled:true,stamina_reward:+reward.value,daily_limit:+limit.value,cooldown_seconds:+cooldown.value
  })}); load();
}
async function grant(){
  await fetch(`/api/admin/players/${uid.value}/stamina`,{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({
    amount:+amount.value,reason:reason.value,operator:'admin'
  })}); load();
}
load();
</script>
</body>
</html>"#;
