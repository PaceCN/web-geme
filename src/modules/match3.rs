use leptos::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

const BOARD_SIZE: usize = 6;
const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE;
const GEM_TYPES: i32 = 5;

#[derive(Clone, Serialize, Deserialize)]
pub struct Match3Game {
    pub board: Vec<i32>,
    pub score: i32,
    pub moves_left: i32,
    pub selected: Option<usize>,
    pub message: String,
}

#[component]
pub fn Match3Overlay(
    open_game: RwSignal<bool>,
    game: RwSignal<Match3Game>,
    best_score: i32,
    finish_score: RwSignal<Option<i32>>,
) -> impl IntoView {
    view! {
        <div class="game-modal">
            <section class="game-panel match3-panel">
                <div class="game-head">
                    <div>
                        <h2 class="section-title">"开心消消乐"</h2>
                        <p class="muted">"交换相邻图块，凑出三连清扫落叶和蛛网"</p>
                    </div>
                    <button class="close" on:click=move |_| open_game.set(false) type="button">"×"</button>
                </div>
                <div class="score-row">
                    <div class="score-chip"><small>"本局"</small><strong>{move || game.with(|g| g.score)}</strong></div>
                    <div class="score-chip"><small>"最高"</small><strong>{best_score}</strong></div>
                    <div class="score-chip"><small>"步数"</small><strong>{move || game.with(|g| g.moves_left)}</strong></div>
                </div>
                <div class="match3-board">
                    {move || game.with(|g| {
                        g.board.iter().enumerate().map(|(index, value)| {
                            let class = if g.selected == Some(index) { "match3-cell selected" } else { "match3-cell" };
                            view! {
                                <button
                                    class=class
                                    style=format!("--gem-color:{}", gem_color(*value))
                                    on:click=move |_| game.update(|state| state.tap(index))
                                    type="button"
                                >
                                    {gem_label(*value)}
                                </button>
                            }
                        }).collect_view()
                    })}
                </div>
                <p class="game-hint">{move || game.with(|g| g.message.clone())}</p>
                <div class="actions">
                    <button class="btn secondary" on:click=move |_| game.set(Match3Game::new()) type="button">"重开"</button>
                    <button
                        class="btn blue"
                        on:click=move |_| {
                            let score = game.with(|g| g.score);
                            finish_score.set(Some(score));
                            open_game.set(false);
                            game.set(Match3Game::new());
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

impl Match3Game {
    pub fn new() -> Self {
        let mut game = Self {
            board: (0..CELL_COUNT).map(|_| random_gem()).collect(),
            score: 0,
            moves_left: 18,
            selected: None,
            message: "点选一个图块，再点相邻图块交换。".into(),
        };
        game.remove_starting_matches();
        game
    }

    pub fn tap(&mut self, index: usize) {
        if index >= CELL_COUNT || self.moves_left <= 0 {
            return;
        }
        let Some(first) = self.selected else {
            self.selected = Some(index);
            self.message = "选择相邻图块进行交换。".into();
            return;
        };
        if first == index {
            self.selected = None;
            self.message = "已取消选择。".into();
            return;
        }
        if !is_adjacent(first, index) {
            self.selected = Some(index);
            self.message = "只能交换上下左右相邻图块。".into();
            return;
        }

        self.board.swap(first, index);
        let matches = collect_matches(&self.board);
        if matches.is_empty() {
            self.board.swap(first, index);
            self.selected = None;
            self.message = "这次交换没有形成三连。".into();
            return;
        }

        self.moves_left -= 1;
        let cleared = self.resolve_matches(matches);
        self.selected = None;
        self.message = format!("清扫成功，消除 {} 个图块。", cleared);
    }

    fn resolve_matches(&mut self, mut matches: BTreeSet<usize>) -> usize {
        let mut total = 0;
        let mut chain = 0;
        while !matches.is_empty() && chain < 8 {
            let cleared = matches.len();
            total += cleared;
            self.score += (cleared as i32) * 10 + chain * 8;
            for index in matches {
                self.board[index] = random_gem();
            }
            matches = collect_matches(&self.board);
            chain += 1;
        }
        total
    }

    fn remove_starting_matches(&mut self) {
        for _ in 0..8 {
            let matches = collect_matches(&self.board);
            if matches.is_empty() {
                return;
            }
            for index in matches {
                self.board[index] = random_gem();
            }
        }
    }
}

fn collect_matches(board: &[i32]) -> BTreeSet<usize> {
    let mut matches = BTreeSet::new();

    for row in 0..BOARD_SIZE {
        let mut start = 0;
        while start < BOARD_SIZE {
            let value = board[row * BOARD_SIZE + start];
            let mut end = start + 1;
            while end < BOARD_SIZE && board[row * BOARD_SIZE + end] == value {
                end += 1;
            }
            if end - start >= 3 {
                for col in start..end {
                    matches.insert(row * BOARD_SIZE + col);
                }
            }
            start = end;
        }
    }

    for col in 0..BOARD_SIZE {
        let mut start = 0;
        while start < BOARD_SIZE {
            let value = board[start * BOARD_SIZE + col];
            let mut end = start + 1;
            while end < BOARD_SIZE && board[end * BOARD_SIZE + col] == value {
                end += 1;
            }
            if end - start >= 3 {
                for row in start..end {
                    matches.insert(row * BOARD_SIZE + col);
                }
            }
            start = end;
        }
    }

    matches
}

fn is_adjacent(a: usize, b: usize) -> bool {
    let ar = a / BOARD_SIZE;
    let ac = a % BOARD_SIZE;
    let br = b / BOARD_SIZE;
    let bc = b % BOARD_SIZE;
    ar.abs_diff(br) + ac.abs_diff(bc) == 1
}

fn random_gem() -> i32 {
    (js_sys::Math::random() * GEM_TYPES as f64).floor() as i32
}

fn gem_label(value: i32) -> &'static str {
    match value {
        0 => "叶",
        1 => "花",
        2 => "晶",
        3 => "丝",
        _ => "星",
    }
}

fn gem_color(value: i32) -> &'static str {
    match value {
        0 => "#f6c453",
        1 => "#ef6f8f",
        2 => "#57c7d4",
        3 => "#9b83f0",
        _ => "#77c96f",
    }
}
