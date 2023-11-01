use crate::bitboard::{bitboards, Bitboard};
use crate::square::squares;
use crate::{
    board::Board, direction::Direction, game::Game, move_tables, moves::Move,
    piece::PromotionPieceKind, player::Player,
};

struct Ctx {
    all_pieces: Bitboard,
    their_pieces: Bitboard,
    their_attacks: Bitboard,
    enemy_or_empty: Bitboard,
}

pub fn generate_all_attacks(board: &Board, player: Player) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;

    let our_pieces = board.player_pieces(player);
    let their_pieces = board.player_pieces(player.other()).all();
    let all_pieces = our_pieces.all() | their_pieces;

    for pawn in our_pieces.pawns {
        attacks |= move_tables::pawn_attacks(pawn, player);
    }

    for knight in our_pieces.knights {
        attacks |= move_tables::knight_attacks(knight);
    }

    for bishop in our_pieces.bishops {
        attacks |= move_tables::bishop_attacks(bishop, all_pieces);
    }

    for rook in our_pieces.rooks {
        attacks |= move_tables::rook_attacks(rook, all_pieces);
    }

    for queen in our_pieces.queens {
        attacks |= move_tables::queen_attacks(queen, all_pieces);
    }

    for king in our_pieces.king {
        attacks |= move_tables::king_attacks(king);
    }

    attacks
}

pub fn generate_moves(game: &Game) -> Vec<Move> {
    let ctx = get_ctx(game);

    let mut moves: Vec<Move> = Vec::with_capacity(64);
    generate_pawn_moves(&mut moves, game, &ctx);
    generate_knight_moves(&mut moves, game, &ctx);
    generate_bishop_moves(&mut moves, game, &ctx);
    generate_rook_moves(&mut moves, game, &ctx);
    generate_queen_moves(&mut moves, game, &ctx);
    generate_king_moves(&mut moves, game, &ctx);
    moves
}

fn get_ctx(game: &Game) -> Ctx {
    let our_pieces = game.board.player_pieces(game.player).all();
    let enemy_or_empty = our_pieces.invert();
    let their_pieces = game.board.player_pieces(game.player.other()).all();
    let their_attacks = generate_all_attacks(&game.board, game.player.other());
    let all_pieces = our_pieces | their_pieces;

    Ctx {
        all_pieces,
        their_pieces,
        their_attacks,
        enemy_or_empty,
    }
}

fn generate_pawn_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let pawns = game.board.player_pieces(game.player).pawns;

    let pawn_move_direction = Direction::pawn_move_direction(game.player);
    let back_rank = bitboards::pawn_back_rank(game.player);
    let will_promote_rank = bitboards::pawn_back_rank(game.player.other());

    for start in pawns {
        let will_promote = !((will_promote_rank & start).is_empty());

        // Move forward by 1
        let forward_one = start.in_direction(pawn_move_direction);

        if let Some(dst) = forward_one {
            if !ctx.all_pieces.contains(dst) {
                if will_promote {
                    for promotion in PromotionPieceKind::ALL {
                        moves.push(Move::new_with_promotion(start, dst, *promotion));
                    }
                } else {
                    moves.push(Move::new(start, dst));
                }
            }
        }

        // Capture
        let attacks = move_tables::pawn_attacks(start, game.player);

        for dst in attacks {
            if ctx.their_pieces.contains(dst) || game.en_passant_target == Some(dst) {
                if will_promote {
                    for promotion in PromotionPieceKind::ALL {
                        moves.push(Move::new_with_promotion(start, dst, *promotion));
                    }
                } else {
                    moves.push(Move::new(start, dst));
                }
            }
        }
    }

    for start in pawns & back_rank {
        // Move forward by 2
        let forward_one = start.in_direction(pawn_move_direction);

        if let Some(forward_one) = forward_one {
            if ctx.all_pieces.contains(forward_one) {
                // Cannot jump over pieces
                continue;
            }

            let forward_two = forward_one.in_direction(pawn_move_direction);

            if let Some(forward_two) = forward_two {
                if !ctx.all_pieces.contains(forward_two) {
                    moves.push(Move::new(start, forward_two));
                }
            }
        }
    }
}

fn generate_knight_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let knights = game.board.player_pieces(game.player).knights;

    for knight in knights {
        let destinations = move_tables::knight_attacks(knight) & ctx.enemy_or_empty;

        for dst in destinations {
            moves.push(Move::new(knight, dst));
        }
    }
}

fn generate_bishop_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let bishops = game.board.player_pieces(game.player).bishops;

    for bishop in bishops {
        let destinations = move_tables::bishop_attacks(bishop, ctx.all_pieces) & ctx.enemy_or_empty;

        for dst in destinations {
            moves.push(Move::new(bishop, dst));
        }
    }
}

fn generate_rook_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let rooks = game.board.player_pieces(game.player).rooks;

    for rook in rooks {
        let destinations = move_tables::rook_attacks(rook, ctx.all_pieces) & ctx.enemy_or_empty;

        for dst in destinations {
            moves.push(Move::new(rook, dst));
        }
    }
}

fn generate_queen_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let queens = game.board.player_pieces(game.player).queens;

    for queen in queens {
        let destinations = move_tables::queen_attacks(queen, ctx.all_pieces) & ctx.enemy_or_empty;

        for dst in destinations {
            moves.push(Move::new(queen, dst));
        }
    }
}

fn generate_king_moves(moves: &mut Vec<Move>, game: &Game, ctx: &Ctx) {
    let king = game.board.player_pieces(game.player).king.single();

    let destinations = move_tables::king_attacks(king) & ctx.enemy_or_empty;

    for dst in destinations {
        moves.push(Move::new(king, dst));
    }

    let king_start_square = squares::king_start(game.player);

    if king == king_start_square && !ctx.their_attacks.contains(king) {
        let castle_rights_for_player = match game.player {
            Player::White => game.white_castle_rights,
            Player::Black => game.black_castle_rights,
        };

        if castle_rights_for_player.can_castle() {
            if castle_rights_for_player.king_side {
                let kingside_required_empty_and_not_attacked_squares =
                    bitboards::kingside_required_empty_and_not_attacked_squares(game.player);

                let pieces_in_the_way =
                    kingside_required_empty_and_not_attacked_squares & ctx.all_pieces;
                let attacked_squares =
                    kingside_required_empty_and_not_attacked_squares & ctx.their_attacks;
                let squares_preventing_castling = pieces_in_the_way | attacked_squares;

                if squares_preventing_castling.is_empty() {
                    moves.push(Move::new(king, squares::kingside_castle_dest(game.player)));
                }
            }

            if castle_rights_for_player.queen_side {
                let queenside_required_empty_squares =
                    bitboards::queenside_required_empty_squares(game.player);
                let queenside_required_not_attacked_squares =
                    bitboards::queenside_required_not_attacked_squares(game.player);

                let pieces_in_the_way = queenside_required_empty_squares & ctx.all_pieces;
                let attacked_squares = queenside_required_not_attacked_squares & ctx.their_attacks;
                let squares_preventing_castling = pieces_in_the_way | attacked_squares;

                if squares_preventing_castling.is_empty() {
                    moves.push(Move::new(king, squares::queenside_castle_dest(game.player)));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::square::squares::all::*;
    use crate::square::Square;

    #[inline(always)]
    fn should_allow_move(fen: &str, squares: (Square, Square)) {
        crate::init();
        let mut game = Game::from_fen(fen).unwrap();

        let moves = game
            .pseudo_legal_moves()
            .into_iter()
            .filter(|m| {
                let player = game.player;
                game.make_move(m);
                let is_in_check = game.board.king_in_check(player);
                game.undo_move();
                !is_in_check
            })
            .collect::<Vec<_>>();

        let (src, dst) = squares;
        let mv = Move::new(src, dst);

        assert!(moves.iter().any(|m| *m == mv));
    }

    #[test]
    fn test_simple_rook_move() {
        should_allow_move(
            "rnbqkbnr/1ppppppp/p7/8/8/P7/1PPPPPPP/RNBQKBNR w KQkq - 0 2",
            (A1, A2),
        );
    }

    #[test]
    fn test_simple_bishop_move() {
        let fen = "rnbqkbnr/1ppppp1p/p5p1/8/8/1P6/PBPPPPPP/RN1QKBNR w KQkq - 0 3";
        should_allow_move(fen, (B2, C3));
        should_allow_move(fen, (B2, H8));
    }

    #[test]
    fn test_en_passant_bug_20230308() {
        should_allow_move(
            "rnbqkbnr/2pppppp/p7/Pp6/8/8/1PPPPPPP/RNBQKBNR w KQkq b6 0 3",
            (A5, B6),
        );
    }
}
