#[macro_use]
mod macros;
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
use crate::chess::piece::{Piece, PieceKind};
use crate::chess::player::ByPlayer;
use crate::chess::player::Player;
use crate::chess::square::Square;
pub use crate::engine::eval::phased_eval::PhasedEval;

parameters!(
    (material, PieceKind::N, array, "PIECE_VALUES"),
    (pawn_pst, Square::N, pst, "PAWNS"),
    (knight_pst, Square::N, pst, "KNIGHTS"),
    (bishop_pst, Square::N, pst, "BISHOPS"),
    (rook_pst, Square::N, pst, "ROOKS"),
    (queen_pst, Square::N, pst, "QUEENS"),
    (king_pst, Square::N, pst, "KING"),
    (passed_pawn_pst, Square::N, pst, "PASSED_PAWNS"),
    (knight_mobility, 9, array, "KNIGHT_MOBILITY"),
    (bishop_mobility, 14, array, "BISHOP_MOBILITY"),
    (rook_mobility, 15, array, "ROOK_MOBILITY"),
    (queen_mobility, 28, array, "QUEEN_MOBILITY"),
    (attacked_king_squares, 9, array, "ATTACKED_KING_SQUARES"),
    (bishop_pair, 1, single, "BISHOP_PAIR_BONUS"),
    (doubled_pawns, 1, single, "DOUBLED_PAWN_MALUS"),
);

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

pub fn eval(game: &Game) -> Eval {
    let absolute_eval = absolute_eval(game);
    Eval::from_white_eval(absolute_eval, game.player)
}

pub fn absolute_eval(game: &Game) -> WhiteEval {
    let mut trace = Trace::new();
    absolute_eval_with_trace::<false>(game, &mut trace)
}

pub fn absolute_eval_with_trace<const TRACE: bool>(game: &Game, trace: &mut Trace) -> WhiteEval {
    if TRACE {
        // Material counts and PSTs are updated incrementally so if we're tuning we need
        // to account for those manually here in the trace.
        material::trace_psts_and_material(game, trace);
    }

    let eval = game.incremental_eval.piece_square_tables
        + material::eval::<TRACE>(game, trace)
        + mobility_and_king_safety::eval::<TRACE>(game, trace)
        + pawn_structure::eval::<TRACE>(game, trace);

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

pub fn eval_components(game: &Game) -> EvalComponents {
    let eval = absolute_eval(game);
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
