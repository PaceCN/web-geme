use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Player {
    pub user_id: String,
    pub nickname: String,
    pub level: i32,
    pub exp: i32,
    pub coins: i32,
    pub gems: i32,
    pub current_stamina: i32,
    pub max_stamina: i32,
    pub score_2048: i32,
    pub score_match3: i32,
    pub score_make10: i32,
    #[serde(default)]
    pub ad_claim_date: String,
    #[serde(default)]
    pub ad_stamina_claims_today: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub item_id: String,
    pub count: i32,
    pub name: String,
    pub item_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Requirement {
    pub item_id: String,
    pub count: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Order {
    pub order_id: String,
    pub status: String,
    pub title: String,
    pub requirements: Vec<Requirement>,
    pub coins_reward: i32,
    pub exp_reward: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    pub task_id: String,
    pub title: String,
    pub description: String,
    pub target_progress: i32,
    pub current_progress: i32,
    pub claimed: bool,
    pub task_type: String,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: Player,
    pub inventory: Vec<InventoryItem>,
    pub orders: Vec<Order>,
    pub tasks: Vec<Task>,
    pub transactions: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameReward {
    pub item_id: String,
    pub count: i32,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct GameRunResult {
    pub game_id: String,
    pub score: i32,
    pub rewards: Vec<GameReward>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct AdRewardPolicy {
    pub placement_id: String,
    pub enabled: bool,
    pub stamina_reward: i32,
    pub daily_limit: i32,
    pub cooldown_seconds: i32,
}

impl Default for AdRewardPolicy {
    fn default() -> Self {
        Self {
            placement_id: "reward_stamina".into(),
            enabled: true,
            stamina_reward: 8,
            daily_limit: 6,
            cooldown_seconds: 45,
        }
    }
}
