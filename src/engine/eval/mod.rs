mod material;
mod mobility_and_king_safety;
mod params;
pub mod pawn_structure;
mod phased_eval;
pub mod piece_square_tables;
mod player_eval;
mod white_eval;

use crate::chess::board::Board;
pub use player_eval::Eval;
pub use white_eval::WhiteEval;

use crate::chess::game::Game;
use crate::chess::piece::Piece;
use crate::chess::player::ByPlayer;
use crate::chess::square::Square;
pub use crate::engine::eval::phased_eval::PhasedEval;
use crate::engine::transposition_table::TranspositionTable;

pub fn init() {
    piece_square_tables::init();
    pawn_structure::init();
}

#[derive(Debug, Clone)]
pub struct IncrementalEvalFields {
    pub phase_value: i16,

    pub piece_square_tables: PhasedEval,
}

impl IncrementalEvalFields {
    pub fn set_at(&mut self, sq: Square, piece: Piece) {
        self.phase_value += phased_eval::piece_phase_value_contribution(piece.kind);
        self.piece_square_tables += piece_square_tables::piece_contributions(sq, piece);
    }

    pub fn remove_at(&mut self, sq: Square, piece: Piece) {
        self.phase_value -= phased_eval::piece_phase_value_contribution(piece.kind);
        self.piece_square_tables -= piece_square_tables::piece_contributions(sq, piece);
    }
}

impl IncrementalEvalFields {
    pub fn init(board: &Board) -> Self {
        let phase_value = phased_eval::phase_value(board);
        let piece_square_tables = piece_square_tables::eval(board);

        Self {
            phase_value,

            piece_square_tables,
        }
    }
}

pub fn eval(game: &Game, pawn_structure_tt: &mut TranspositionTable<PhasedEval>) -> Eval {
    let absolute_eval = absolute_eval(game, pawn_structure_tt);
    Eval::from_white_eval(absolute_eval, game.player)
}

pub fn absolute_eval(
    game: &Game,
    pawn_structure_tt: &mut TranspositionTable<PhasedEval>,
) -> WhiteEval {
    let eval = game.incremental_eval.piece_square_tables
        + material::eval(game)
        + mobility_and_king_safety::eval(game)
        + pawn_structure::eval(game, pawn_structure_tt);

    eval.for_phase(game.incremental_eval.phase_value)
}

#[derive(Debug)]
pub struct EvalComponent {
    pub eval: WhiteEval,
    pub player_eval: ByPlayer<WhiteEval>,
    pub phased_player_eval: ByPlayer<PhasedEval>,
}

impl EvalComponent {
    pub fn from_phased_eval(phased_player_eval: ByPlayer<PhasedEval>, game_phase: i16) -> Self {
        let white_player_phased_eval = phased_player_eval.white();
        let black_player_phased_eval = phased_player_eval.black();

        let white_player_eval = white_player_phased_eval.for_phase(game_phase);
        let black_player_eval = black_player_phased_eval.for_phase(game_phase);

        let eval = white_player_eval + black_player_eval;

        Self {
            eval,
            player_eval: ByPlayer::new(white_player_eval, black_player_eval),
            phased_player_eval,
        }
    }
}

#[derive(Debug)]
pub struct EvalComponents {
    pub eval: WhiteEval,
    pub phase_value: i16,

    pub piece_square: EvalComponent,
    pub passed_pawns: EvalComponent,
}

pub fn eval_components(
    game: &Game,
    pawn_structure_tt: &mut TranspositionTable<PhasedEval>,
) -> EvalComponents {
    let eval = absolute_eval(game, pawn_structure_tt);
    let phase_value = game.incremental_eval.phase_value;

    let piece_square_eval = piece_square_tables::eval_by_player(&game.board);
    let passed_pawns_eval = pawn_structure::eval_passed_pawns_by_player(&game.board);

    EvalComponents {
        eval,
        phase_value,

        piece_square: EvalComponent::from_phased_eval(piece_square_eval, phase_value),
        passed_pawns: EvalComponent::from_phased_eval(passed_pawns_eval, phase_value),
    }
}
