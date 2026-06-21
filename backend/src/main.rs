use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::Html,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::HashMap,
    env,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, RwLock},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::CorsLayer;

const SESSION_TTL_SECONDS: u64 = 12 * 60 * 60;

#[derive(Clone)]
struct AppState {
    inner: Arc<RwLock<BackendState>>,
}

#[derive(Clone)]
struct BackendState {
    players: HashMap<String, PlayerAdminRecord>,
    ad_policy: AdRewardPolicy,
    game_switches: HashMap<String, GameSwitch>,
    admin_accounts: HashMap<String, AdminAccount>,
    sessions: HashMap<String, AdminSession>,
    audit_logs: Vec<AuditLog>,
}

#[derive(Clone, Serialize, Deserialize)]
struct PlayerAdminRecord {
    user_id: String,
    nickname: String,
    level: i32,
    coins: i32,
    current_stamina: i32,
    max_stamina: i32,
    score_2048: i32,
    score_match3: i32,
    score_make10: i32,
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
struct GameSwitch {
    game_id: String,
    title: String,
    enabled: bool,
    maintenance_message: String,
}

#[derive(Clone)]
struct AdminAccount {
    username: String,
    password_hash: String,
    role: String,
    enabled: bool,
    created_at: u64,
}

#[derive(Clone)]
struct AdminSession {
    username: String,
    expires_at: u64,
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
    accounts_total: usize,
    ad_reward_enabled: bool,
    stamina_reward: i32,
    daily_limit: i32,
    ad_claims_today: i32,
    game_switches: Vec<GameSwitch>,
    latest_logs: Vec<AuditLog>,
}

#[derive(Serialize)]
struct AdminAccountView {
    username: String,
    role: String,
    enabled: bool,
    created_at: u64,
}

#[derive(Serialize)]
struct BootstrapResponse {
    player: PlayerAdminRecord,
    ad_policy: AdRewardPolicy,
    game_switches: Vec<GameSwitch>,
}

#[derive(Deserialize)]
struct LoginRequest {
    username: String,
    password: String,
}

#[derive(Serialize)]
struct LoginResponse {
    success: bool,
    token: String,
    message: String,
    expires_at: u64,
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
}

#[derive(Deserialize)]
struct GameActionRequest {
    user_id: String,
    action_id: String,
    target_id: String,
    score: Option<i32>,
}

#[derive(Serialize)]
struct GameActionResponse {
    success: bool,
    message: String,
    player: Option<PlayerAdminRecord>,
}

#[derive(Deserialize)]
struct UpdateGameSwitchRequest {
    enabled: bool,
    maintenance_message: String,
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
        .route("/api/admin/login", post(admin_login))
        .route("/api/admin/dashboard", get(admin_dashboard))
        .route("/api/admin/accounts", get(list_admin_accounts))
        .route("/api/admin/ad-policy", get(get_ad_policy).put(update_ad_policy))
        .route("/api/admin/game-switches", get(list_game_switches))
        .route("/api/admin/game-switches/:game_id", post(update_game_switch))
        .route("/api/admin/players", get(list_players))
        .route("/api/admin/players/:user_id/stamina", post(grant_stamina))
        .route("/api/v1/bootstrap/:user_id", get(player_bootstrap))
        .route("/api/v1/game-switches", get(public_game_switches))
        .route("/api/v1/garden/action", post(garden_action))
        .route("/api/v1/ads/reward", post(claim_ad_reward))
        .layer(CorsLayer::permissive())
        .with_state(state);

    let bind_host = env::var("HUAYUAN_BACKEND_HOST")
        .ok()
        .and_then(|value| value.parse::<IpAddr>().ok())
        .unwrap_or(IpAddr::V4(Ipv4Addr::LOCALHOST));
    let bind_port = env::var("HUAYUAN_BACKEND_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8787);
    let addr = SocketAddr::from((bind_host, bind_port));
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

async fn admin_login(
    State(state): State<AppState>,
    Json(request): Json<LoginRequest>,
) -> Json<LoginResponse> {
    let mut store = state.inner.write().expect("state write");
    let password_hash = hash_password(&request.password);
    let Some(account) = store.admin_accounts.get(&request.username) else {
        push_log(&mut store, "anonymous", "admin.login.failed", "未知后台账户尝试登录");
        return Json(LoginResponse::failed("账户或密码错误"));
    };
    let account_username = account.username.clone();
    let account_password_hash = account.password_hash.clone();
    let account_enabled = account.enabled;
    if !account_enabled || account_password_hash != password_hash {
        push_log(&mut store, &request.username, "admin.login.failed", "后台密码错误或账户禁用");
        return Json(LoginResponse::failed("账户或密码错误"));
    }

    let token = issue_session_token(&account_username, &account_password_hash);
    let expires_at = unix_now() + SESSION_TTL_SECONDS;
    store.sessions.insert(token.clone(), AdminSession { username: account_username, expires_at });
    push_log(&mut store, &request.username, "admin.login.success", "后台登录成功");
    Json(LoginResponse {
        success: true,
        token,
        message: "登录成功".into(),
        expires_at,
    })
}

async fn admin_dashboard(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<AdminDashboard>, StatusCode> {
    let store = state.inner.read().expect("state read");
    require_admin(&headers, &store)?;
    let today = today_key();
    let ad_claims_today = store
        .players
        .values()
        .filter(|player| player.ad_claim_date == today)
        .map(|player| player.ad_claims_today)
        .sum();
    Ok(Json(AdminDashboard {
        players_total: store.players.len(),
        accounts_total: store.admin_accounts.len(),
        ad_reward_enabled: store.ad_policy.enabled,
        stamina_reward: store.ad_policy.stamina_reward,
        daily_limit: store.ad_policy.daily_limit,
        ad_claims_today,
        game_switches: sorted_game_switches(&store),
        latest_logs: store.audit_logs.iter().rev().take(20).cloned().collect(),
    }))
}

async fn list_admin_accounts(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<AdminAccountView>>, StatusCode> {
    let store = state.inner.read().expect("state read");
    require_admin(&headers, &store)?;
    let mut accounts: Vec<_> = store
        .admin_accounts
        .values()
        .map(|account| AdminAccountView {
            username: account.username.clone(),
            role: account.role.clone(),
            enabled: account.enabled,
            created_at: account.created_at,
        })
        .collect();
    accounts.sort_by(|a, b| a.username.cmp(&b.username));
    Ok(Json(accounts))
}

async fn get_ad_policy(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<AdRewardPolicy>, StatusCode> {
    let store = state.inner.read().expect("state read");
    require_admin(&headers, &store)?;
    Ok(Json(store.ad_policy.clone()))
}

async fn update_ad_policy(
    headers: HeaderMap,
    State(state): State<AppState>,
    Json(policy): Json<AdRewardPolicy>,
) -> Result<Json<AdRewardPolicy>, StatusCode> {
    let mut store = state.inner.write().expect("state write");
    let username = require_admin(&headers, &store)?;
    store.ad_policy = policy.clone();
    push_log(&mut store, &username, "ad_policy.update", "更新广告恢复体力配置");
    Ok(Json(policy))
}

async fn list_game_switches(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<GameSwitch>>, StatusCode> {
    let store = state.inner.read().expect("state read");
    require_admin(&headers, &store)?;
    Ok(Json(sorted_game_switches(&store)))
}

async fn update_game_switch(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(game_id): Path<String>,
    Json(request): Json<UpdateGameSwitchRequest>,
) -> Result<Json<GameSwitch>, StatusCode> {
    let mut store = state.inner.write().expect("state write");
    let username = require_admin(&headers, &store)?;
    let updated = {
        let game = store
            .game_switches
            .entry(game_id.clone())
            .or_insert_with(|| GameSwitch {
                game_id: game_id.clone(),
                title: game_id.clone(),
                enabled: true,
                maintenance_message: String::new(),
            });
        game.enabled = request.enabled;
        game.maintenance_message = request.maintenance_message;
        game.clone()
    };
    push_log(&mut store, &username, "game_switch.update", &format!("{} 开关更新", game_id));
    Ok(Json(updated))
}

async fn list_players(
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Json<Vec<PlayerAdminRecord>>, StatusCode> {
    let store = state.inner.read().expect("state read");
    require_admin(&headers, &store)?;
    let mut players: Vec<_> = store.players.values().cloned().collect();
    players.sort_by(|a, b| a.user_id.cmp(&b.user_id));
    Ok(Json(players))
}

async fn grant_stamina(
    headers: HeaderMap,
    State(state): State<AppState>,
    Path(user_id): Path<String>,
    Json(request): Json<StaminaGrantRequest>,
) -> Result<Json<AdRewardResponse>, StatusCode> {
    let mut store = state.inner.write().expect("state write");
    let username = require_admin(&headers, &store)?;
    let daily_limit = store.ad_policy.daily_limit;
    let response = {
        let Some(player) = store.players.get_mut(&user_id) else {
            return Ok(Json(AdRewardResponse::not_found()));
        };
        let amount = request.amount.clamp(0, 999);
        player.current_stamina = (player.current_stamina + amount).min(player.max_stamina);
        AdRewardResponse {
            accepted: true,
            message: format!("后台补发体力 {} 点：{}", amount, request.reason),
            current_stamina: player.current_stamina,
            max_stamina: player.max_stamina,
            claims_left_today: (daily_limit - player.ad_claims_today).max(0),
        }
    };
    push_log(
        &mut store,
        &username,
        "player.stamina.grant",
        &format!("{} 补发体力 {}", user_id, request.amount),
    );
    Ok(Json(response))
}

async fn player_bootstrap(
    State(state): State<AppState>,
    Path(user_id): Path<String>,
) -> Result<Json<BootstrapResponse>, StatusCode> {
    let store = state.inner.read().expect("state read");
    let Some(player) = store.players.get(&user_id) else {
        return Err(StatusCode::NOT_FOUND);
    };
    Ok(Json(BootstrapResponse {
        player: player.clone(),
        ad_policy: store.ad_policy.clone(),
        game_switches: sorted_game_switches(&store),
    }))
}

async fn public_game_switches(State(state): State<AppState>) -> Json<Vec<GameSwitch>> {
    Json(sorted_game_switches(&state.inner.read().expect("state read")))
}

async fn garden_action(
    State(state): State<AppState>,
    Json(request): Json<GameActionRequest>,
) -> Json<GameActionResponse> {
    let mut store = state.inner.write().expect("state write");
    let response = match request.action_id.as_str() {
        "finish_minigame" => finish_minigame_action(&mut store, &request),
        _ => GameActionResponse {
            success: false,
            message: "暂不支持该游戏动作。".into(),
            player: None,
        },
    };
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
    let response = {
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
        AdRewardResponse {
            accepted: true,
            message: format!("{} 广告奖励已发放", request.ad_network),
            current_stamina: player.current_stamina,
            max_stamina: player.max_stamina,
            claims_left_today: (policy.daily_limit - player.ad_claims_today).max(0),
        }
    };
    push_log(
        &mut store,
        "ad-network",
        "ad.reward.claim",
        &format!("{} 领取广告体力奖励", request.user_id),
    );
    Json(response)
}

impl LoginResponse {
    fn failed(message: &str) -> Self {
        Self {
            success: false,
            token: String::new(),
            message: message.into(),
            expires_at: 0,
        }
    }
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

fn finish_minigame_action(store: &mut BackendState, request: &GameActionRequest) -> GameActionResponse {
    let game_id = request.target_id.as_str();
    let Some(game_switch) = store.game_switches.get(game_id) else {
        return GameActionResponse {
            success: false,
            message: "小游戏不存在。".into(),
            player: None,
        };
    };
    let game_title = game_switch.title.clone();
    let game_enabled = game_switch.enabled;
    let maintenance_message = game_switch.maintenance_message.clone();
    if !game_enabled {
        return GameActionResponse {
            success: false,
            message: if maintenance_message.is_empty() {
                "小游戏维护中，暂时不能结算。".into()
            } else {
                maintenance_message
            },
            player: None,
        };
    }

    let score = request.score.unwrap_or(0).clamp(0, 200_000);
    let updated = {
        let Some(player) = store.players.get_mut(&request.user_id) else {
            return GameActionResponse {
                success: false,
                message: "玩家不存在。".into(),
                player: None,
            };
        };
        if player.is_banned {
            return GameActionResponse {
                success: false,
                message: "账号已被后台限制。".into(),
                player: None,
            };
        }
        match game_id {
            "2048" => player.score_2048 = player.score_2048.max(score),
            "match3" => player.score_match3 = player.score_match3.max(score),
            "make10" => player.score_make10 = player.score_make10.max(score),
            _ => {}
        }
        let coin_reward = minigame_coin_reward(game_id, score);
        player.coins += coin_reward;
        player.last_active_at = unix_now();
        (player.clone(), coin_reward)
    };

    push_log(
        store,
        "game-api",
        "garden.finish_minigame",
        &format!("{} 完成 {} 分数 {}", request.user_id, game_id, score),
    );
    GameActionResponse {
        success: true,
        message: format!("{} 结算成功，金币 +{}。", game_title, updated.1),
        player: Some(updated.0),
    }
}

fn minigame_coin_reward(game_id: &str, score: i32) -> i32 {
    let base = match game_id {
        "2048" => score / 64,
        "match3" => score / 30,
        "make10" => score / 25,
        _ => score / 50,
    };
    base.clamp(0, 120)
}

fn seed_backend_state() -> BackendState {
    let admin_user = env::var("HUAYUAN_ADMIN_USER").unwrap_or_else(|_| "admin".into());
    let admin_password = env::var("HUAYUAN_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123!".into());
    let mut admin_accounts = HashMap::new();
    admin_accounts.insert(
        admin_user.clone(),
        AdminAccount {
            username: admin_user,
            password_hash: hash_password(&admin_password),
            role: "owner".into(),
            enabled: true,
            created_at: unix_now(),
        },
    );

    let mut players = HashMap::new();
    players.insert(
        "u_10001".into(),
        PlayerAdminRecord {
            user_id: "u_10001".into(),
            nickname: "庄园见习主理人".into(),
            level: 1,
            coins: 300,
            current_stamina: 12,
            max_stamina: 30,
            score_2048: 0,
            score_match3: 0,
            score_make10: 0,
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
        game_switches: seed_game_switches(),
        admin_accounts,
        sessions: HashMap::new(),
        audit_logs: Vec::new(),
    }
}

fn seed_game_switches() -> HashMap<String, GameSwitch> {
    let games = [
        ("2048", "2048 扫除拼图"),
        ("match3", "开心消消乐"),
        ("make10", "一不小心就到十"),
    ];
    games
        .into_iter()
        .map(|(game_id, title)| {
            (
                game_id.into(),
                GameSwitch {
                    game_id: game_id.into(),
                    title: title.into(),
                    enabled: true,
                    maintenance_message: String::new(),
                },
            )
        })
        .collect()
}

fn sorted_game_switches(store: &BackendState) -> Vec<GameSwitch> {
    let mut games: Vec<_> = store.game_switches.values().cloned().collect();
    games.sort_by(|a, b| a.game_id.cmp(&b.game_id));
    games
}

fn require_admin(headers: &HeaderMap, store: &BackendState) -> Result<String, StatusCode> {
    let token = headers
        .get("x-admin-token")
        .and_then(|value| value.to_str().ok())
        .unwrap_or("");
    let Some(session) = store.sessions.get(token) else {
        return Err(StatusCode::UNAUTHORIZED);
    };
    if session.expires_at <= unix_now() {
        return Err(StatusCode::UNAUTHORIZED);
    }
    if !store.admin_accounts.get(&session.username).map(|account| account.enabled).unwrap_or(false) {
        return Err(StatusCode::UNAUTHORIZED);
    }
    Ok(session.username.clone())
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

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(b"huayuan-admin-v1:");
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn issue_session_token(username: &str, password_hash: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(username.as_bytes());
    hasher.update(b":");
    hasher.update(password_hash.as_bytes());
    hasher.update(b":");
    hasher.update(unix_nonce().to_string().as_bytes());
    format!("{:x}", hasher.finalize())
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn unix_nonce() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
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
    main{max-width:1080px;margin:0 auto;padding:24px;display:grid;gap:16px}
    section{border:1px solid rgba(195,227,174,.18);border-radius:8px;background:#17242d;padding:16px}
    h1,h2{margin:0 0 12px}.grid{display:grid;grid-template-columns:repeat(auto-fit,minmax(180px,1fr));gap:12px}
    input,button{height:38px;border-radius:8px;border:1px solid rgba(255,255,255,.12);padding:0 10px}
    input{background:#0d161d;color:#fff}button{background:#65b86a;color:#07100a;font-weight:800;cursor:pointer}
    button.danger{background:#f06f5c}.hidden{display:none}.muted{color:#9fb19b;font-size:13px}
    pre{white-space:pre-wrap;color:#c8d7c3}.pill{display:inline-block;margin:4px 6px 4px 0;padding:6px 9px;border-radius:8px;background:#0d161d}
  </style>
</head>
<body>
<main>
  <h1>庄园物语运营后台</h1>
  <section id="loginBox">
    <h2>管理员登录</h2>
    <p class="muted">默认本地账号 admin / admin123!，正式部署必须用环境变量覆盖。</p>
    <div class="grid">
      <input id="loginUser" placeholder="账户" value="admin">
      <input id="loginPass" placeholder="密码" type="password">
      <button onclick="login()">登录</button>
    </div>
    <p id="loginMsg" class="muted"></p>
  </section>
  <section id="appBox" class="hidden">
    <div class="grid">
      <button onclick="load()">刷新后台</button>
      <button class="danger" onclick="logout()">退出登录</button>
    </div>
  </section>
  <section class="authed hidden"><h2>数据看板</h2><div id="dashboard" class="grid"></div></section>
  <section class="authed hidden">
    <h2>广告恢复体力配置</h2>
    <div class="grid">
      <input id="reward" placeholder="单次恢复体力">
      <input id="limit" placeholder="每日次数">
      <input id="cooldown" placeholder="冷却秒数">
      <button onclick="savePolicy()">保存配置</button>
    </div>
  </section>
  <section class="authed hidden">
    <h2>小游戏开关</h2>
    <div id="games"></div>
  </section>
  <section class="authed hidden">
    <h2>玩家体力补发</h2>
    <div class="grid">
      <input id="uid" value="u_10001">
      <input id="amount" value="8">
      <input id="reason" value="运营补偿">
      <button onclick="grant()">补发体力</button>
    </div>
  </section>
  <section class="authed hidden"><h2>后台账户</h2><pre id="accounts"></pre></section>
  <section class="authed hidden"><h2>玩家账户</h2><pre id="players"></pre></section>
  <section class="authed hidden"><h2>日志</h2><pre id="logs"></pre></section>
</main>
<script>
let token=sessionStorage.getItem('huayuan_admin_token')||'';
function showAuthed(ok){
  loginBox.classList.toggle('hidden',ok); appBox.classList.toggle('hidden',!ok);
  document.querySelectorAll('.authed').forEach(el=>el.classList.toggle('hidden',!ok));
}
async function api(url,options={}){
  options.headers=Object.assign({'content-type':'application/json','x-admin-token':token},options.headers||{});
  const res=await fetch(url,options);
  if(res.status===401){ logout(); throw new Error('登录已失效'); }
  return res.json();
}
async function login(){
  const data=await fetch('/api/admin/login',{method:'POST',headers:{'content-type':'application/json'},body:JSON.stringify({
    username:loginUser.value,password:loginPass.value
  })}).then(r=>r.json());
  loginMsg.textContent=data.message;
  if(data.success){ token=data.token; sessionStorage.setItem('huayuan_admin_token',token); showAuthed(true); load(); }
}
function logout(){ token=''; sessionStorage.removeItem('huayuan_admin_token'); showAuthed(false); }
async function load(){
  const dash=await api('/api/admin/dashboard');
  dashboard.innerHTML=[
    ['玩家数',dash.players_total],['后台账户',dash.accounts_total],['广告开关',dash.ad_reward_enabled?'开启':'关闭'],
    ['单次体力',dash.stamina_reward],['每日次数',dash.daily_limit],['今日广告领取',dash.ad_claims_today]
  ].map(([k,v])=>`<section><b>${k}</b><p>${v}</p></section>`).join('');
  games.innerHTML=dash.game_switches.map(g=>`<div class="pill"><b>${g.title}</b> ${g.enabled?'已开启':'已关闭'} <button onclick="toggleGame('${g.game_id}',${!g.enabled})">${g.enabled?'关闭':'开启'}</button></div>`).join('');
  logs.textContent=JSON.stringify(dash.latest_logs,null,2);
  const policy=await api('/api/admin/ad-policy');
  reward.value=policy.stamina_reward; limit.value=policy.daily_limit; cooldown.value=policy.cooldown_seconds;
  accounts.textContent=JSON.stringify(await api('/api/admin/accounts'),null,2);
  players.textContent=JSON.stringify(await api('/api/admin/players'),null,2);
}
async function savePolicy(){
  await api('/api/admin/ad-policy',{method:'PUT',body:JSON.stringify({
    placement_id:'reward_stamina',enabled:true,stamina_reward:+reward.value,daily_limit:+limit.value,cooldown_seconds:+cooldown.value
  })}); load();
}
async function toggleGame(gameId,enabled){
  await api(`/api/admin/game-switches/${gameId}`,{method:'POST',body:JSON.stringify({enabled,maintenance_message:enabled?'':'后台维护中'})});
  load();
}
async function grant(){
  await api(`/api/admin/players/${uid.value}/stamina`,{method:'POST',body:JSON.stringify({
    amount:+amount.value,reason:reason.value
  })}); load();
}
showAuthed(!!token); if(token) load();
</script>
</body>
</html>"#;
