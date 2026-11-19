use std::fs;
use std::io;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use primitive_app::{App, Change};
use primitive_ui::{
    back_button, header, screen_rect, Canvas, Input, Rect, TextStyle, BLACK, BODY, CAPTION, PAPER,
};
use ships_core::{Difficulty, Game, Mark, Mode, Orientation, Stage, Strike, FLEET, SIZE};

const MODES: [(Mode, &str); 4] = [
    (Mode::Computer(Difficulty::Easy), "Easy solo game"),
    (Mode::Computer(Difficulty::Standard), "Standard solo game"),
    (Mode::Computer(Difficulty::Hard), "Hard solo game"),
    (Mode::TwoPlayers, "Two players"),
];

pub struct View {
    pub game: Option<Game>,
    path: Option<PathBuf>,
    seed: u64,
    direction: Orientation,
    own: bool,
    confirm: bool,
    notice: Option<String>,
}

impl View {
    pub fn open(path: PathBuf) -> Self {
        let game = fs::read(&path)
            .ok()
            .and_then(|data| serde_json::from_slice::<Option<Game>>(&data).ok().flatten())
            .and_then(Game::restored);
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos() as u64;
        Self::new(seed, game, Some(path))
    }

    pub fn preview(seed: u64) -> Self {
        Self::new(seed, None, None)
    }

    fn new(seed: u64, game: Option<Game>, path: Option<PathBuf>) -> Self {
        Self {
            game,
            path,
            seed,
            direction: Orientation::Across,
            own: false,
            confirm: false,
            notice: None,
        }
    }

    fn draw_modes(&self, canvas: &mut Canvas, width: i32) {
        canvas.text(
            rect(width, 40, 285, 1184, 100),
            "Place your fleet and find the other ships.",
            TextStyle::new(48).center(),
        );
        for (i, (_, label)) in MODES.into_iter().enumerate() {
            canvas.button(mode_button(width, i), label, i == 1, false);
        }
    }

    fn draw_board(&self, game: &Game, canvas: &mut Canvas, width: i32) {
        let board = board_rect(width);
        for index in 0..SIZE {
            let x = board.x + board.w * index as i32 / SIZE as i32;
            let y = board.y + board.h * index as i32 / SIZE as i32;
            let cell = board.w / SIZE as i32;
            let letter = char::from(b'A' + index as u8).to_string();
            canvas.text(
                Rect::new(x, board.y - canvas.px(57), cell, canvas.px(54)),
                &letter,
                CAPTION.center(),
            );
            canvas.text(
                Rect::new(
                    board.x - canvas.px(60),
                    y + canvas.px(23),
                    canvas.px(54),
                    cell,
                ),
                &(index + 1).to_string(),
                CAPTION.center(),
            );
        }
        let can_place = !self.confirm
            && matches!(game.stage, Stage::Setup(_))
            && game.setup_count() < FLEET.len();
        let can_fire = !self.confirm && matches!(game.stage, Stage::Battle(_)) && !self.own;
        for y in 0..SIZE {
            for x in 0..SIZE {
                let w = board.w / SIZE as i32;
                let h = board.h / SIZE as i32;
                let bounds = Rect::new(board.x + x as i32 * w, board.y + y as i32 * h, w, h);
                let mark = game.visible(x, y, self.own);
                let pressed = (can_place || can_fire && matches!(mark, Mark::Unknown))
                    && canvas.control_pressed(bounds, false);
                canvas.fill(bounds.inset(1), if pressed { BLACK } else { 242 });
                match mark {
                    Mark::Unknown => {}
                    Mark::Ship => canvas.panel(bounds.inset(canvas.px(9)), 14, 2, 85),
                    Mark::Miss => canvas.circle(bounds.inset(canvas.px(41)), 60),
                    Mark::Hit => draw_hit(canvas, bounds, false),
                    Mark::Sunk => draw_hit(canvas, bounds, true),
                }
            }
        }
        for index in 0..=SIZE {
            let exterior = matches!(index, 0 | SIZE);
            let line = canvas.px(if exterior { 5 } else { 3 });
            let ink = if exterior { BLACK } else { 85 };
            let x = board.x + board.w * index as i32 / SIZE as i32;
            let y = board.y + board.h * index as i32 / SIZE as i32;
            canvas.fill(Rect::new(x, board.y, line, board.h + line), ink);
            canvas.fill(Rect::new(board.x, y, board.w + line, line), ink);
        }
    }

    fn draw_setup(&self, game: &Game, canvas: &mut Canvas, width: i32, player: usize) {
        let title = if game.mode == Mode::TwoPlayers {
            format!("Player {}: place your fleet", player + 1)
        } else {
            "Place your fleet".into()
        };
        canvas.text(
            rect(width, 40, 170, 1184, 70),
            &title,
            TextStyle::new(48).bold().center(),
        );
        let count = game.setup_count();
        let text = match FLEET.get(count) {
            Some(length) => format!("Tap the first square for a ship of {length} spaces."),
            None => "Fleet ready. Start when you're ready.".into(),
        };
        canvas.text(rect(width, 40, 236, 1184, 58), &text, CAPTION.center());
        self.draw_board(game, canvas, width);
        if count < FLEET.len() {
            let direction = match self.direction {
                Orientation::Across => "Across",
                Orientation::Down => "Down",
            };
            canvas.button(setup_button(width, 0), direction, false, false);
        }
        canvas.button(setup_button(width, 1), "Randomize", false, false);
        if count > 0 {
            canvas.button(setup_button(width, 2), "Clear fleet", false, false);
        }
        if count == FLEET.len() {
            canvas.button(wide_button(width, 1400), "Start", true, false);
        }
    }

    fn draw_battle(
        &self,
        game: &Game,
        canvas: &mut Canvas,
        width: i32,
        active: usize,
        finished: bool,
    ) {
        let label = if finished {
            match (game.mode, active) {
                (Mode::Computer(_), 0) => "You won".into(),
                (Mode::Computer(_), _) => "The Kindle won".into(),
                (Mode::TwoPlayers, _) => format!("Player {} won", active + 1),
            }
        } else if game.mode == Mode::TwoPlayers {
            format!(
                "Player {}: {}",
                active + 1,
                if self.own { "your fleet" } else { "target map" }
            )
        } else if self.own {
            "Your fleet".into()
        } else {
            "Target map".into()
        };
        canvas.text(
            rect(width, 40, 169, 1184, 76),
            &label,
            TextStyle::new(48).bold().center(),
        );
        let help = if finished {
            "Both fleets are visible."
        } else if self.own {
            "Check where the other fleet fired."
        } else {
            "Tap an unmarked square to fire."
        };
        canvas.text(rect(width, 40, 236, 1184, 58), help, CAPTION.center());
        self.draw_board(game, canvas, width);
        canvas.text(
            rect(width, 40, 1218, 1184, 63),
            "Dot: miss   ·   X: hit   ·   Dark X: sunk",
            CAPTION.center(),
        );
        canvas.button(
            rect(width, 180, 1330, 436, 100),
            "Target map",
            !self.own,
            false,
        );
        canvas.button(
            rect(width, 650, 1330, 436, 100),
            "Your fleet",
            self.own,
            false,
        );
    }

    fn draw_pass(&self, canvas: &mut Canvas, width: i32, player: usize) {
        canvas.text(
            rect(width, 40, 510, 1184, 110),
            &format!("Pass to player {}", player + 1),
            TextStyle::new(70).bold().center(),
        );
        canvas.text(
            rect(width, 40, 665, 1184, 80),
            "Open the map when you have the Kindle.",
            BODY.center(),
        );
        canvas.button(wide_button(width, 819), "Show map", true, false);
    }

    fn tap(&mut self, x: f32, y: f32) -> Change {
        let width = primitive_app::WIDTH as i32;
        if self.confirm {
            self.confirm = false;
            if rect(width, 666, 808, 360, 100).contains(x, y) {
                self.game = None;
                self.notice = None;
                self.persist();
            }
            return Change::Full;
        }
        if back_button(width).contains(x, y) {
            return Change::Exit;
        }
        let Some(game) = self.game.as_mut() else {
            if let Some((index, (mode, _))) = MODES
                .into_iter()
                .enumerate()
                .find(|(i, _)| mode_button(width, *i).contains(x, y))
            {
                self.seed = self.seed.wrapping_add(index as u64 + 1);
                self.game = Some(Game::new(mode, self.seed));
                self.notice = None;
                self.direction = Orientation::Across;
                self.own = false;
                self.persist();
                return Change::Full;
            }
            return Change::None;
        };
        if new_game_button(width).contains(x, y) {
            self.confirm = true;
            return Change::Full;
        }
        let mut redraw = Change::Partial;
        match game.stage {
            Stage::Setup(_) => {
                if setup_button(width, 0).contains(x, y) && game.setup_count() < FLEET.len() {
                    self.direction = match self.direction {
                        Orientation::Across => Orientation::Down,
                        Orientation::Down => Orientation::Across,
                    };
                    return Change::Partial;
                }
                if setup_button(width, 1).contains(x, y) {
                    game.randomize();
                    self.notice = Some("Fleet randomized.".into());
                } else if setup_button(width, 2).contains(x, y) && game.setup_count() > 0 {
                    game.clear_fleet();
                    self.notice = Some("Fleet cleared.".into());
                } else if wide_button(width, 1400).contains(x, y) && game.confirm_fleet() {
                    self.own = false;
                    self.notice = None;
                    redraw = Change::Full;
                } else if let Some((col, row)) = board_point(width, x, y) {
                    if !game.place(col, row, self.direction) {
                        self.notice = Some("That ship does not fit there.".into());
                        return Change::Partial;
                    }
                    self.notice = None;
                } else {
                    return Change::None;
                }
            }
            Stage::PassSetup | Stage::PassBattle(_) => {
                if !wide_button(width, 819).contains(x, y) {
                    return Change::None;
                }
                game.pass_ready();
                self.own = false;
                self.notice = None;
                redraw = Change::Full;
            }
            Stage::Battle(_) | Stage::Finished(_) => {
                if rect(width, 180, 1330, 436, 100).contains(x, y) {
                    self.own = false;
                    return Change::Partial;
                }
                if rect(width, 650, 1330, 436, 100).contains(x, y) {
                    self.own = true;
                    return Change::Partial;
                }
                let Some((col, row)) = board_point(width, x, y) else {
                    return Change::None;
                };
                if self.own || matches!(game.stage, Stage::Finished(_)) {
                    return Change::None;
                }
                let stage = game.stage;
                if let Some(round) = game.fire(col, row) {
                    let hit = result(round.strike);
                    let mut notice = format!("{}{}: {hit}.", char::from(b'A' + col as u8), row + 1);
                    if let Some(((x, y), reply)) = round.reply {
                        notice.push_str(&format!(
                            " Kindle at {}{}: {}.",
                            char::from(b'A' + x as u8),
                            y + 1,
                            result(reply)
                        ));
                    }
                    self.notice = Some(notice);
                    if game.stage != stage {
                        redraw = Change::Full;
                    }
                } else {
                    self.notice = Some("You already fired there.".into());
                    return Change::Partial;
                }
            }
        }
        self.persist();
        redraw
    }

    fn persist(&mut self) {
        if self.save().is_err() {
            self.notice = Some("Could not save the game.".into());
        }
    }
}

impl App for View {
    fn draw(&self, canvas: &mut Canvas) {
        let width = canvas.bounds().w;
        header(canvas, "Ships", "Done");
        match &self.game {
            None => self.draw_modes(canvas, width),
            Some(game) => {
                match game.stage {
                    Stage::Setup(player) => self.draw_setup(game, canvas, width, player),
                    Stage::PassSetup => self.draw_pass(canvas, width, 1),
                    Stage::PassBattle(player) => self.draw_pass(canvas, width, player),
                    Stage::Battle(player) => self.draw_battle(game, canvas, width, player, false),
                    Stage::Finished(winner) => self.draw_battle(game, canvas, width, winner, true),
                }
                canvas.button(new_game_button(width), "New game", false, false);
                if let Some(message) = self
                    .notice
                    .as_ref()
                    .filter(|_| !matches!(game.stage, Stage::PassSetup | Stage::PassBattle(_)))
                {
                    canvas.text(rect(width, 40, 1535, 730, 62), message, CAPTION.lines(2));
                }
            }
        }
        if self.confirm {
            canvas.popup(rect(width, 190, 580, 884, 375));
            canvas.text(
                rect(width, 238, 632, 788, 70),
                "Start a new game?",
                TextStyle::new(50).bold(),
            );
            canvas.text(
                rect(width, 238, 720, 788, 65),
                "This replaces the game in progress.",
                CAPTION,
            );
            canvas.button(rect(width, 238, 808, 360, 100), "Cancel", false, false);
            canvas.button(rect(width, 666, 808, 360, 100), "New game", true, false);
        }
    }

    fn input(&mut self, input: Input) -> Change {
        match input {
            Input::Tap(x, y) => self.tap(x, y),
            Input::PageBack if self.confirm => {
                self.confirm = false;
                Change::Full
            }
            Input::PageBack => Change::Exit,
            _ => Change::None,
        }
    }

    fn save(&mut self) -> io::Result<()> {
        if let Some(path) = &self.path {
            primitive_app::save(path, &serde_json::to_vec(&self.game)?)?;
        }
        Ok(())
    }
}

fn draw_hit(canvas: &mut Canvas, bounds: Rect, sunk: bool) {
    let tile = bounds.inset(canvas.px(12));
    canvas.panel(tile, 13, 3, if sunk { BLACK } else { PAPER });
    canvas.text(
        Rect::new(tile.x, tile.y + canvas.px(10), tile.w, tile.h),
        "X",
        TextStyle::new(41)
            .bold()
            .center()
            .ink(if sunk { PAPER } else { BLACK }),
    );
}

fn result(strike: Strike) -> &'static str {
    match strike {
        Strike::Miss => "miss",
        Strike::Hit => "hit",
        Strike::Sunk => "sunk",
    }
}

fn rect(width: i32, x: i32, y: i32, w: i32, h: i32) -> Rect {
    screen_rect(width, x, y, w, h)
}

fn board_rect(width: i32) -> Rect {
    rect(width, 212, 342, 840, 840)
}

fn board_point(width: i32, x: f32, y: f32) -> Option<(usize, usize)> {
    let area = board_rect(width);
    area.contains(x, y).then(|| {
        (
            ((x - area.x as f32) * SIZE as f32 / area.w as f32) as usize,
            ((y - area.y as f32) * SIZE as f32 / area.h as f32) as usize,
        )
    })
}

fn mode_button(width: i32, index: usize) -> Rect {
    rect(width, 200, 460 + index as i32 * 150, 864, 110)
}

fn setup_button(width: i32, index: usize) -> Rect {
    rect(width, 40 + index as i32 * 400, 1225, 384, 100)
}

fn wide_button(width: i32, y: i32) -> Rect {
    rect(width, 275, y, 714, 100)
}

fn new_game_button(width: i32) -> Rect {
    rect(width, 845, 1510, 379, 100)
}
