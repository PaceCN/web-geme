use leptos::*;
use serde::{Deserialize, Serialize};

const BOARD_SIZE: usize = 4;
const CELL_COUNT: usize = BOARD_SIZE * BOARD_SIZE;

#[derive(Clone, Serialize, Deserialize)]
pub struct Make10Game {
    pub board: Vec<i32>,
    pub score: i32,
    pub moves_left: i32,
    pub combos: i32,
    pub selected: Vec<usize>,
    pub message: String,
}

#[component]
pub fn Make10Overlay(
    open_game: RwSignal<bool>,
    game: RwSignal<Make10Game>,
    best_score: i32,
    finish_score: RwSignal<Option<i32>>,
) -> impl IntoView {
    view! {
        <div class="game-modal">
            <section class="game-panel make10-panel">
                <div class="game-head">
                    <div>
                        <h2 class="section-title">"一不小心就到十"</h2>
                        <p class="muted">"连选相邻数字，凑满 10 就能清点木柴"</p>
                    </div>
                    <button class="close" on:click=move |_| open_game.set(false) type="button">"×"</button>
                </div>
                <div class="score-row">
                    <div class="score-chip"><small>"本局"</small><strong>{move || game.with(|g| g.score)}</strong></div>
                    <div class="score-chip"><small>"最高"</small><strong>{best_score}</strong></div>
                    <div class="score-chip"><small>"剩余"</small><strong>{move || game.with(|g| g.moves_left)}</strong></div>
                </div>
                <div class="make10-sum">
                    <span>"当前合计"</span>
                    <strong>{move || game.with(|g| g.selected_sum())}</strong>
                </div>
                <div class="make10-board">
                    {move || game.with(|g| {
                        g.board.iter().enumerate().map(|(index, value)| {
                            let class = if g.selected.contains(&index) { "make10-cell selected" } else { "make10-cell" };
                            view! {
                                <button
                                    class=class
                                    on:click=move |_| game.update(|state| state.tap(index))
                                    type="button"
                                >
                                    {*value}
                                </button>
                            }
                        }).collect_view()
                    })}
                </div>
                <p class="game-hint">{move || game.with(|g| g.message.clone())}</p>
                <div class="actions">
                    <button class="btn secondary" on:click=move |_| game.set(Make10Game::new()) type="button">"重开"</button>
                    <button
                        class="btn orange"
                        on:click=move |_| {
                            let score = game.with(|g| g.score);
                            finish_score.set(Some(score));
                            open_game.set(false);
                            game.set(Make10Game::new());
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

impl Make10Game {
    pub fn new() -> Self {
        Self {
            board: (0..CELL_COUNT).map(|_| random_number()).collect(),
            score: 0,
            moves_left: 16,
            combos: 0,
            selected: Vec::new(),
            message: "点选相邻数字，凑到 10 自动清除。".into(),
        }
    }

    pub fn tap(&mut self, index: usize) {
        if index >= CELL_COUNT || self.moves_left <= 0 {
            return;
        }
        if self.selected.contains(&index) {
            self.selected.retain(|value| *value != index);
            self.message = "已移除一个数字。".into();
            return;
        }
        if let Some(last) = self.selected.last() {
            if !is_adjacent(*last, index) {
                self.selected.clear();
                self.selected.push(index);
                self.message = "路线断开，已重新开始选择。".into();
                return;
            }
        }

        self.selected.push(index);
        let sum = self.selected_sum();
        if sum == 10 {
            let count = self.selected.len() as i32;
            self.score += 20 + count * 12 + self.combos * 5;
            self.combos += 1;
            self.moves_left -= 1;
            for index in self.selected.drain(..) {
                self.board[index] = random_number();
            }
            self.message = format!("正好到十，连击 {}。", self.combos);
        } else if sum > 10 {
            self.selected.clear();
            self.selected.push(index);
            self.message = "超过 10 了，从当前数字重新开始。".into();
        } else {
            self.message = format!("当前合计 {}，继续找相邻数字。", sum);
        }
    }

    pub fn selected_sum(&self) -> i32 {
        self.selected.iter().map(|index| self.board[*index]).sum()
    }
}

fn is_adjacent(a: usize, b: usize) -> bool {
    let ar = a / BOARD_SIZE;
    let ac = a % BOARD_SIZE;
    let br = b / BOARD_SIZE;
    let bc = b % BOARD_SIZE;
    ar.abs_diff(br) + ac.abs_diff(bc) == 1
}

fn random_number() -> i32 {
    (js_sys::Math::random() * 9.0).floor() as i32 + 1
}
