use leptos::*;
use serde::{Deserialize, Serialize};
use web_sys::PointerEvent;

#[derive(Clone, Serialize, Deserialize)]
pub struct Game2048 {
    pub board: Vec<i32>,
    pub score: i32,
    pub moves: i32,
    pub over: bool,
}

#[component]
pub fn Game2048Overlay(
    open_game: RwSignal<bool>,
    game: RwSignal<Game2048>,
    best_score: i32,
    finish_score: RwSignal<Option<i32>>,
) -> impl IntoView {
    let drag_start = create_rw_signal::<Option<(f64, f64)>>(None);

    view! {
        <div class="game-modal">
            <section class="game-panel game2048-panel">
                <div class="game-head">
                    <div>
                        <h2 class="section-title">"2048 扫除拼图"</h2>
                        <p class="muted">"滑动合并方块，结算枯草和落叶"</p>
                    </div>
                    <button class="close" on:click=move |_| open_game.set(false) type="button">"×"</button>
                </div>
                <div class="score-row">
                    <div class="score-chip"><small>"本局"</small><strong>{move || game.with(|g| g.score)}</strong></div>
                    <div class="score-chip"><small>"最高"</small><strong>{best_score}</strong></div>
                    <div class="score-chip"><small>"步数"</small><strong>{move || game.with(|g| g.moves)}</strong></div>
                </div>
                <div
                    class="game2048-board"
                    on:pointerdown=move |event: PointerEvent| {
                        drag_start.set(Some((event.client_x() as f64, event.client_y() as f64)));
                    }
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
                            view! {
                                <div class=tile_class(*value) style=format!("background:{}", tile_color(*value))>
                                    <span>{if *value == 0 { "".to_string() } else { value.to_string() }}</span>
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
                            finish_score.set(Some(score));
                            open_game.set(false);
                            game.set(Game2048::new());
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

impl Game2048 {
    pub fn new() -> Self {
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

    pub fn move_dir(&mut self, direction: &str) {
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

fn tile_class(value: i32) -> &'static str {
    match value {
        0 => "game2048-cell empty",
        2 | 4 => "game2048-cell low",
        8..=64 => "game2048-cell",
        128..=512 => "game2048-cell large",
        _ => "game2048-cell huge",
    }
}

fn tile_color(value: i32) -> &'static str {
    match value {
        0 => "#263247",
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
