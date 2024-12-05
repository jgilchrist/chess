use crate::chess::game::Game;

use crate::chess::bitboard::{bitboards, Bitboard};
use crate::chess::board::Board;
use crate::chess::player::{ByPlayer, Player};
use crate::chess::square;
use crate::chess::square::{Rank, Square};
use crate::engine::eval::params::{PieceSquareTableDefinition, DOUBLED_PAWN};
use crate::engine::eval::piece_square_tables::{flatten, flip, negate, PieceSquareTable};
use crate::engine::eval::{params, PhasedEval};

pub fn eval(game: &Game) -> PhasedEval {
    let eval = eval_passed_pawns(game) + evaluate_doubled_pawns(game);

    eval
}

static mut ENEMY_PASSED_PAWN_MASKS: [[Bitboard; Square::N]; Player::N] =
    [[Bitboard::EMPTY; Square::N]; Player::N];

static mut PASSED_PAWN_PST: [[PhasedEval; Square::N]; Player::N] =
    [[PhasedEval::ZERO; Square::N]; Player::N];

pub fn white_pst(def: PieceSquareTableDefinition) -> PieceSquareTable {
    flatten(flip(def))
}

pub fn black_pst(def: PieceSquareTableDefinition) -> PieceSquareTable {
    negate(flatten(def))
}

fn generate_passed_pawn_mask(player: Player, square: Square) -> Bitboard {
    let our_bank_rank = bitboards::back_rank(player);
    let their_pawn_back_rank = bitboards::pawn_back_rank(player.other());

    // Pawns cannot be on our back rank
    if (square.bb() & our_bank_rank).any() {
        return Bitboard::EMPTY;
    }

    // Pawns on their pawn rank cannot be blocked, as pawns cannot be on their back rank
    if (square.bb() & their_pawn_back_rank).any() {
        return Bitboard::EMPTY;
    }

    let file = square.file().bitboard();
    let file_left = file.west();
    let file_right = file.east();

    let relevant_files = file_left | file | file_right;

    let rank = square.rank();
    let mut relevant_ranks = Bitboard::FULL;

    let back_rank_idx = match player {
        Player::White => Rank::R1,
        Player::Black => Rank::R8,
    };

    let distance_from_back_rank = back_rank_idx.array_idx().abs_diff(rank.array_idx());

    for _ in 0..=distance_from_back_rank {
        relevant_ranks = relevant_ranks.forward(player);
    }

    let enemy_pawns_mask = relevant_files & relevant_ranks;
    enemy_pawns_mask
}

fn enemy_passed_pawn_mask(player: Player, square: Square) -> Bitboard {
    *unsafe {
        ENEMY_PASSED_PAWN_MASKS
            .get_unchecked(player.array_idx())
            .get_unchecked(square.array_idx())
    }
}

fn pst_value(player: Player, square: Square) -> PhasedEval {
    *unsafe {
        PASSED_PAWN_PST
            .get_unchecked(player.array_idx())
            .get_unchecked(square.array_idx())
    }
}

pub fn eval_passed_pawns(game: &Game) -> PhasedEval {
    let white_bonus = calculate_passed_pawn_bonus(&game.board, Player::White);
    let black_bonus = calculate_passed_pawn_bonus(&game.board, Player::Black);

    white_bonus + black_bonus
}

pub fn eval_passed_pawns_by_player(board: &Board) -> ByPlayer<PhasedEval> {
    let white_bonus = calculate_passed_pawn_bonus(board, Player::White);
    let black_bonus = calculate_passed_pawn_bonus(board, Player::Black);

    ByPlayer::new(white_bonus, black_bonus)
}

pub fn is_passed(pawn: Square, player: Player, their_pawns: Bitboard) -> bool {
    (enemy_passed_pawn_mask(player, pawn) & their_pawns).is_empty()
}

fn calculate_passed_pawn_bonus(board: &Board, player: Player) -> PhasedEval {
    let mut bonus = PhasedEval::ZERO;

    let our_pawns = board.pawns(player);
    let their_pawns = board.pawns(player.other());

    for pawn in our_pawns {
        if is_passed(pawn, player, their_pawns) {
            bonus += pst_value(player, pawn);
        }
    }

    bonus
}

fn evaluate_doubled_pawns(game: &Game) -> PhasedEval {
    let mut eval = PhasedEval::ZERO;

    let white_pawns = game.board.pawns(Player::White);
    let black_pawns = game.board.pawns(Player::Black);

    for file in square::FILES {
        let bb = file.bitboard();

        if (white_pawns & bb).count() > 1 {
            eval += DOUBLED_PAWN;
        }

        if (black_pawns & bb).count() > 1 {
            eval -= DOUBLED_PAWN;
        }
    }

    eval
}

pub fn init() {
    for player in [Player::White, Player::Black] {
        for square in Bitboard::FULL {
            let mask = generate_passed_pawn_mask(player, square);
            unsafe {
                ENEMY_PASSED_PAWN_MASKS[player.array_idx()][square.array_idx()] = mask;
            }
        }
    }

    unsafe {
        PASSED_PAWN_PST[Player::White.array_idx()] = white_pst(params::PASSED_PAWNS);
        PASSED_PAWN_PST[Player::Black.array_idx()] = black_pst(params::PASSED_PAWNS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chess::game::Game;
    use crate::chess::square::squares::all::*;

    #[test]
    pub fn test_is_passed_white() {
        crate::init();

        let game = Game::from_fen("4k3/4p3/7p/2PK2p1/3P2P1/8/8/8 w - - 0 1").unwrap();

        assert!(is_passed(
            C5,
            Player::White,
            game.board.pawns(Player::Black)
        ));
    }

    #[test]
    pub fn test_is_passed_black() {
        crate::init();

        let game = Game::from_fen("4k3/4p3/7p/2PK2p1/1P4P1/8/8/8 w - - 0 1").unwrap();

        assert!(is_passed(
            E7,
            Player::Black,
            game.board.pawns(Player::White)
        ));

        let game = Game::from_fen("4k3/8/7p/2PK2p1/1P4P1/8/4p3/8 w - - 0 1").unwrap();

        assert!(is_passed(
            E2,
            Player::Black,
            game.board.pawns(Player::White)
        ));

        let game = Game::from_fen("4k3/8/7p/2PK2p1/1P4P1/8/3Pp3/8 w - - 0 1").unwrap();

        assert!(is_passed(
            E2,
            Player::Black,
            game.board.pawns(Player::White)
        ));

        let game = Game::from_fen("4k3/8/7p/2PK2p1/1P4P1/3P4/3Pp3/8 w - - 0 1").unwrap();

        assert!(is_passed(
            E2,
            Player::Black,
            game.board.pawns(Player::White)
        ));
    }
}
