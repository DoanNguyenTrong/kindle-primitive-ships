use serde::{Deserialize, Serialize};

pub const SIZE: usize = 8;
pub const FLEET: [usize; 4] = [4, 3, 3, 2];

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Difficulty {
    Easy,
    Standard,
    Hard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Mode {
    Computer(Difficulty),
    TwoPlayers,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Orientation {
    Across,
    Down,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mark {
    Unknown,
    Ship,
    Miss,
    Hit,
    Sunk,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Strike {
    Miss,
    Hit,
    Sunk,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Stage {
    Setup(usize),
    PassSetup,
    Battle(usize),
    PassBattle(usize),
    Finished(usize),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
struct Placement {
    x: usize,
    y: usize,
    direction: Orientation,
}

impl Placement {
    fn fits_board(self, length: usize) -> bool {
        if self.x >= SIZE || self.y >= SIZE {
            return false;
        }
        match self.direction {
            Orientation::Across => self.x + length <= SIZE,
            Orientation::Down => self.y + length <= SIZE,
        }
    }

    fn cells(self, length: usize) -> impl Iterator<Item = (usize, usize)> {
        (0..length).map(move |offset| match self.direction {
            Orientation::Across => (self.x + offset, self.y),
            Orientation::Down => (self.x, self.y + offset),
        })
    }
}

#[derive(Clone, Default, Serialize, Deserialize)]
struct Board {
    ships: [Option<Placement>; FLEET.len()],
    shots: [[bool; SIZE]; SIZE],
}

impl Board {
    fn count(&self) -> usize {
        self.ships.iter().flatten().count()
    }

    fn ship_at(&self, x: usize, y: usize) -> Option<usize> {
        self.ships.iter().enumerate().position(|(i, &ship)| {
            ship.is_some_and(|ship| ship.cells(FLEET[i]).any(|cell| cell == (x, y)))
        })
    }

    fn can_place(&self, place: Placement, length: usize) -> bool {
        place.fits_board(length)
            && place
                .cells(length)
                .all(|(x, y)| self.ship_at(x, y).is_none())
    }

    fn place(&mut self, place: Placement) -> bool {
        let index = self.count();
        if index == FLEET.len() || !self.can_place(place, FLEET[index]) {
            return false;
        }
        self.ships[index] = Some(place);
        true
    }

    fn valid(&self) -> bool {
        let mut expected = Board::default();
        for (i, &ship) in self.ships.iter().enumerate() {
            if let Some(ship) = ship {
                if i != expected.count() || !expected.place(ship) {
                    return false;
                }
            }
        }
        true
    }

    fn sunk(&self, index: usize) -> bool {
        self.ships[index]
            .is_some_and(|ship| ship.cells(FLEET[index]).all(|(x, y)| self.shots[y][x]))
    }

    fn lost(&self) -> bool {
        (0..FLEET.len()).all(|i| self.sunk(i))
    }

    fn mark(&self, x: usize, y: usize, reveal: bool) -> Mark {
        let ship = self.ship_at(x, y);
        match (self.shots[y][x], ship) {
            (true, Some(at)) if self.sunk(at) => Mark::Sunk,
            (true, Some(_)) => Mark::Hit,
            (true, None) => Mark::Miss,
            (false, Some(_)) if reveal => Mark::Ship,
            (false, _) => Mark::Unknown,
        }
    }

    fn fire(&mut self, x: usize, y: usize) -> Option<Strike> {
        if x >= SIZE || y >= SIZE || self.shots[y][x] {
            return None;
        }
        self.shots[y][x] = true;
        match self.mark(x, y, false) {
            Mark::Miss => Some(Strike::Miss),
            Mark::Hit => Some(Strike::Hit),
            Mark::Sunk => Some(Strike::Sunk),
            _ => unreachable!("a fired-on cell must have a result"),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Round {
    pub strike: Strike,
    pub reply: Option<((usize, usize), Strike)>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Game {
    pub mode: Mode,
    pub stage: Stage,
    boards: [Board; 2],
    seed: u64,
}

impl Game {
    pub fn new(mode: Mode, seed: u64) -> Self {
        let mut game = Self {
            mode,
            stage: Stage::Setup(0),
            boards: std::array::from_fn(|_| Board::default()),
            seed: seed.max(1),
        };
        if matches!(mode, Mode::Computer(_)) {
            game.boards[1] = game.random_board();
        }
        game
    }

    pub fn restored(self) -> Option<Self> {
        if !self.boards.iter().all(Board::valid) {
            return None;
        }
        let [first, second] = self.boards.each_ref().map(Board::count);
        let full = FLEET.len();
        let valid_stage = match (self.mode, self.stage) {
            (Mode::Computer(_), Stage::Setup(0)) => second == full,
            (Mode::Computer(_), Stage::Battle(0) | Stage::Finished(0 | 1)) => {
                first == full && second == full
            }
            (Mode::TwoPlayers, Stage::Setup(0)) => second == 0,
            (Mode::TwoPlayers, Stage::Setup(1) | Stage::PassSetup) => first == full,
            (
                Mode::TwoPlayers,
                Stage::Battle(0 | 1) | Stage::PassBattle(0 | 1) | Stage::Finished(0 | 1),
            ) => first == full && second == full,
            _ => false,
        };
        valid_stage.then_some(self)
    }

    pub fn setup_count(&self) -> usize {
        match self.stage {
            Stage::Setup(player) => self.boards[player].count(),
            _ => 0,
        }
    }

    pub fn place(&mut self, x: usize, y: usize, direction: Orientation) -> bool {
        let Stage::Setup(player) = self.stage else {
            return false;
        };
        self.boards[player].place(Placement { x, y, direction })
    }

    pub fn randomize(&mut self) {
        if let Stage::Setup(player) = self.stage {
            self.boards[player] = self.random_board();
        }
    }

    pub fn clear_fleet(&mut self) {
        if let Stage::Setup(player) = self.stage {
            self.boards[player] = Board::default();
        }
    }

    pub fn confirm_fleet(&mut self) -> bool {
        let Stage::Setup(player) = self.stage else {
            return false;
        };
        if self.boards[player].count() != FLEET.len() {
            return false;
        }
        self.stage = match (self.mode, player) {
            (Mode::TwoPlayers, 0) => Stage::PassSetup,
            (Mode::TwoPlayers, _) => Stage::PassBattle(0),
            (Mode::Computer(_), _) => Stage::Battle(0),
        };
        true
    }

    pub fn pass_ready(&mut self) -> bool {
        self.stage = match self.stage {
            Stage::PassSetup => Stage::Setup(1),
            Stage::PassBattle(player) => Stage::Battle(player),
            _ => return false,
        };
        true
    }

    pub fn visible(&self, x: usize, y: usize, own_board: bool) -> Mark {
        if x >= SIZE || y >= SIZE {
            return Mark::Unknown;
        }
        let (player, reveal) = match self.stage {
            Stage::Setup(player) => (player, true),
            Stage::Battle(player) if own_board => (player, true),
            Stage::Battle(player) => (1 - player, false),
            Stage::Finished(winner) => {
                let viewer = if self.mode == Mode::TwoPlayers {
                    winner
                } else {
                    0
                };
                (if own_board { viewer } else { 1 - viewer }, true)
            }
            Stage::PassSetup | Stage::PassBattle(_) => return Mark::Unknown,
        };
        self.boards[player].mark(x, y, reveal)
    }

    pub fn fire(&mut self, x: usize, y: usize) -> Option<Round> {
        let Stage::Battle(player) = self.stage else {
            return None;
        };
        let opponent = 1 - player;
        let strike = self.boards[opponent].fire(x, y)?;
        let mut round = Round {
            strike,
            reply: None,
        };
        if self.boards[opponent].lost() {
            self.stage = Stage::Finished(player);
            return Some(round);
        }
        match self.mode {
            Mode::TwoPlayers => self.stage = Stage::PassBattle(opponent),
            Mode::Computer(level) => {
                let targets = targets(&self.boards[player], level);
                let pick = targets[self.random_index(targets.len())];
                let reply = self.boards[player].fire(pick.0, pick.1)?;
                round.reply = Some((pick, reply));
                if self.boards[player].lost() {
                    self.stage = Stage::Finished(opponent);
                }
            }
        }
        Some(round)
    }

    fn random_board(&mut self) -> Board {
        let mut board = Board::default();
        for length in FLEET {
            let candidates: Vec<_> = placements()
                .filter(|place| board.can_place(*place, length))
                .collect();
            let index = self.random_index(candidates.len());
            board.place(candidates[index]);
        }
        board
    }

    fn random_index(&mut self, count: usize) -> usize {
        let mut seed = self.seed.max(1);
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        self.seed = seed;
        (seed % count as u64) as usize
    }
}

fn placements() -> impl Iterator<Item = Placement> {
    (0..SIZE).flat_map(|y| {
        (0..SIZE).flat_map(move |x| {
            [Orientation::Across, Orientation::Down]
                .into_iter()
                .map(move |direction| Placement { x, y, direction })
        })
    })
}

fn targets(board: &Board, level: Difficulty) -> Vec<(usize, usize)> {
    let fields: [[Mark; SIZE]; SIZE] =
        std::array::from_fn(|y| std::array::from_fn(|x| board.mark(x, y, false)));
    let mut score = [[0; SIZE]; SIZE];
    if level == Difficulty::Hard {
        for (i, length) in FLEET.into_iter().enumerate() {
            if board.sunk(i) {
                continue;
            }
            for place in placements().filter(|place| place.fits_board(length)) {
                if place
                    .cells(length)
                    .any(|(x, y)| matches!(fields[y][x], Mark::Miss | Mark::Sunk))
                {
                    continue;
                }
                let hits = place
                    .cells(length)
                    .filter(|&(x, y)| fields[y][x] == Mark::Hit)
                    .count();
                for (x, y) in place.cells(length) {
                    score[y][x] += 1 + hits * hits * 100;
                }
            }
        }
    } else if level == Difficulty::Standard {
        for (y, row) in fields.iter().enumerate() {
            for (x, mark) in row.iter().enumerate() {
                if *mark == Mark::Hit {
                    for (dx, dy) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
                        if let (Some(nx), Some(ny)) =
                            (x.checked_add_signed(dx), y.checked_add_signed(dy))
                        {
                            if nx < SIZE && ny < SIZE {
                                score[ny][nx] += 1;
                            }
                        }
                    }
                }
            }
        }
    }
    let mut best = 0;
    let mut cells = Vec::new();
    for y in 0..SIZE {
        for x in 0..SIZE {
            if fields[y][x] != Mark::Unknown || score[y][x] < best {
                continue;
            }
            if score[y][x] > best {
                best = score[y][x];
                cells.clear();
            }
            cells.push((x, y));
        }
    }
    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manual_fleet_refuses_overlap_and_edges() {
        let mut game = Game::new(Mode::Computer(Difficulty::Easy), 234);
        assert!(!game.place(7, 0, Orientation::Across));
        assert!(game.place(0, 0, Orientation::Across));
        assert!(!game.place(1, 0, Orientation::Down));
        assert!(!game.confirm_fleet());
        assert_eq!(game.visible(0, 0, true), Mark::Ship);
        game.randomize();
        assert_eq!(game.setup_count(), 4);
        assert!(game.confirm_fleet());
        assert_eq!(game.visible(0, 0, false), Mark::Unknown);
        assert!(game.fire(0, 0).unwrap().reply.is_some());
        assert!(game.fire(0, 0).is_none());
    }

    #[test]
    fn handoffs_hide_the_board_and_each_player_gets_a_turn() {
        let mut game = Game::new(Mode::TwoPlayers, 123);
        game.randomize();
        assert!(game.confirm_fleet());
        assert_eq!(game.stage, Stage::PassSetup);
        assert_eq!(game.visible(0, 0, true), Mark::Unknown);
        assert!(game.pass_ready());
        assert_eq!(game.stage, Stage::Setup(1));
        game.randomize();
        assert!(game.confirm_fleet());
        assert_eq!(game.stage, Stage::PassBattle(0));
        assert!(game.pass_ready());
        assert_eq!(game.fire(0, 0).unwrap().reply, None);
        assert_eq!(game.stage, Stage::PassBattle(1));
        assert_eq!(game.visible(0, 0, true), Mark::Unknown);
        assert!(game.pass_ready());
        assert!(game.fire(0, 0).is_some());
        assert_eq!(game.stage, Stage::PassBattle(0));
        assert!(game.restored().is_some());
    }

    #[test]
    fn hard_level_targets_a_line_with_two_hits() {
        let mut board = Board::default();
        assert!(board.place(Placement {
            x: 2,
            y: 2,
            direction: Orientation::Across
        }));
        assert_eq!(board.fire(2, 2), Some(Strike::Hit));
        assert_eq!(board.fire(3, 2), Some(Strike::Hit));
        let hard = targets(&board, Difficulty::Hard);
        assert!(!hard.is_empty());
        assert!(hard.iter().all(|&(x, y)| y == 2 && (x == 1 || x == 4)));
        let easy = targets(&board, Difficulty::Easy);
        assert_eq!(easy.len(), SIZE * SIZE - 2);
        let standard = targets(&board, Difficulty::Standard);
        assert!(standard.contains(&(2, 1)));
        assert!(standard.len() < easy.len());
    }
}
