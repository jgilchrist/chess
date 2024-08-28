use crate::chess::moves::Move;
use std::time::Duration;

use crate::engine::options::EngineOptions;
use crate::engine::search::time_control::TimeStrategy;

use crate::chess::game::Game;
use crate::engine::search::move_picker::MovePicker;
use crate::engine::search::principal_variation::PrincipalVariation;
use crate::engine::search::tables::{HistoryTable, KillersTable};
use crate::engine::search::transposition::SearchTranspositionTable;

mod aspiration;
mod iterative_deepening;
mod move_ordering;
pub mod move_picker;
mod negamax;
mod principal_variation;
mod quiescence;
mod tables;
pub mod time_control;
pub mod transposition;

const MAX_SEARCH_DEPTH: u8 = u8::MAX;
const MAX_SEARCH_DEPTH_SIZE: usize = MAX_SEARCH_DEPTH as usize;

mod params {
    use crate::engine::eval::Eval;

    pub const CHECK_TERMINATION_NODE_FREQUENCY: u64 = 10000;

    pub const ASPIRATION_MIN_DEPTH: u8 = 5;
    pub const ASPIRATION_WINDOW_SIZE: Eval = Eval::new(25);

    pub const NULL_MOVE_PRUNING_DEPTH_LIMIT: u8 = 3;
    pub const NULL_MOVE_PRUNING_DEPTH_REDUCTION: u8 = 2;

    pub const REVERSE_FUTILITY_PRUNE_DEPTH: u8 = 4;
    pub const REVERSE_FUTILITY_PRUNE_MARGIN_PER_PLY: Eval = Eval::new(150);

    pub const HISTORY_DECAY_FACTOR: i32 = 8;

    pub const MAX_TIME_PER_MOVE: f32 = 0.5;
    pub const INCREMENT_TO_USE: f32 = 0.5;
    pub const BASE_TIME_PER_MOVE: f32 = 0.033;

    pub const SOFT_TIME_MULTIPLIER: f32 = 0.75;
    pub const HARD_TIME_MULTIPLIER: f32 = 3.00;
}

pub struct PersistentState {
    pub tt: SearchTranspositionTable,
    pub history_table: HistoryTable,
}

impl PersistentState {
    pub fn new(tt_size_mb: usize) -> Self {
        Self {
            tt: SearchTranspositionTable::new(tt_size_mb),
            history_table: HistoryTable::new(),
        }
    }
}

pub struct SearchState {
    pub killer_moves: KillersTable,

    nodes_visited: u64,
    max_depth_reached: u8,
}

impl SearchState {
    pub const fn new() -> Self {
        Self {
            killer_moves: KillersTable::new(),

            max_depth_reached: 0,
            nodes_visited: 0,
        }
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum SearchScore {
    Centipawns(i16),
    Mate(i16),
}

#[derive(Default)]
pub struct SearchRestrictions {
    pub depth: Option<u8>,
}

#[derive(Debug, Clone)]
pub enum TimeControl {
    Clocks(Clocks),
    ExactTime(Duration),
    Infinite,
}

#[derive(Debug, Clone)]
pub struct Clocks {
    pub white_clock: Option<Duration>,
    pub black_clock: Option<Duration>,
    pub white_increment: Option<Duration>,
    pub black_increment: Option<Duration>,
    pub moves_to_go: Option<u32>,
}

pub struct SearchInfo {
    pub depth: u8,
    pub seldepth: u8,
    pub score: SearchScore,
    pub stats: SearchStats,
    pub pv: PrincipalVariation,
    pub hashfull: usize,
}

pub struct SearchStats {
    pub time: Duration,
    pub nodes: u64,
    pub nodes_per_second: u64,
}

pub trait Reporter {
    #[allow(unused)]
    fn generic_report(&self, s: &str);

    fn report_search_progress(&mut self, game: &Game, progress: SearchInfo);

    fn best_move(&self, game: &Game, mv: Move);
}

pub struct NullReporter;

impl Reporter for NullReporter {
    fn generic_report(&self, _: &str) {}

    fn report_search_progress(&mut self, _: &Game, _: SearchInfo) {}

    fn best_move(&self, _: &Game, _: Move) {}
}

pub struct CapturingReporter {
    pub score: Option<SearchScore>,
    pub nodes: u64,
    pub nps: u64,
}

impl CapturingReporter {
    pub fn new() -> Self {
        Self {
            score: None,
            nodes: 0,
            nps: 0,
        }
    }
}

impl Reporter for CapturingReporter {
    fn generic_report(&self, _: &str) {}

    fn report_search_progress(&mut self, _: &Game, stats: SearchInfo) {
        self.score = Some(stats.score);
        self.nodes = stats.stats.nodes;
        self.nps = stats.stats.nodes_per_second;
    }

    fn best_move(&self, _: &Game, _: Move) {}
}

pub fn search(
    game: &Game,
    persistent_state: &mut PersistentState,
    time_strategy: &mut TimeStrategy,
    search_restrictions: &SearchRestrictions,
    options: &EngineOptions,
    reporter: &mut impl Reporter,
) -> Move {
    let mut state = SearchState::new();

    persistent_state.tt.new_generation();
    persistent_state
        .history_table
        .decay(params::HISTORY_DECAY_FACTOR);

    let mut pv = PrincipalVariation::new();

    // The game is modified as moves are played during search. When the search terminates,
    // the game will be left in a dirty state since we will not undo the moves played to
    // reach the terminating node in the search tree. To keep our original 'game' copy clean
    // we perform the search on a copy of the game.
    let mut search_game = game.clone();

    iterative_deepening::search(
        &mut search_game,
        persistent_state,
        search_restrictions,
        options,
        &mut state,
        &mut pv,
        time_strategy,
        reporter,
    );

    let best_move = pv.first();

    best_move.unwrap_or_else(|| panic_move(game, persistent_state, &state))
}

// If we have so little time to search that we couldn't determine a best move, we'll need to spend
// a bit of extra time so that we still make a move.
// Rather than returning a random move, we return the first move that is returned after move ordering
fn panic_move(game: &Game, persistent_state: &PersistentState, search_state: &SearchState) -> Move {
    let mut move_picker = MovePicker::new(None);
    move_picker
        .next(game, persistent_state, search_state, 0)
        .unwrap()
}
