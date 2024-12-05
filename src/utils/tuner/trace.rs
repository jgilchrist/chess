use crate::chess::bitboard::Bitboard;
use crate::chess::board::Board;
use crate::chess::game::Game;
use crate::chess::movegen::tables;
use crate::chess::piece::PieceKind;
use crate::chess::player::Player;
use crate::chess::square;
use crate::chess::square::Square;
use crate::engine::eval;
use crate::utils::tuner::NonZeroCoefficient;

#[derive(Default, Copy, Clone)]
pub struct TraceComponent(i32);

impl TraceComponent {
    pub fn incr(&mut self, player: Player) {
        self.add(player, 1);
    }

    pub fn add(&mut self, player: Player, n: i32) {
        let multiplier = if player == Player::White { 1 } else { -1 };

        self.0 += n * multiplier;
    }
}

pub struct Trace {
    material: [TraceComponent; PieceKind::N],

    pawn_pst: [TraceComponent; Square::N],
    knight_pst: [TraceComponent; Square::N],
    bishop_pst: [TraceComponent; Square::N],
    rook_pst: [TraceComponent; Square::N],
    queen_pst: [TraceComponent; Square::N],
    king_pst: [TraceComponent; Square::N],

    passed_pawn_pst: [TraceComponent; Square::N],

    knight_mobility: [TraceComponent; 9],
    bishop_mobility: [TraceComponent; 14],
    rook_mobility: [TraceComponent; 15],
    queen_mobility: [TraceComponent; 28],

    attacked_king_squares: [TraceComponent; 9],

    bishop_pair: TraceComponent,
    doubled_pawn: TraceComponent,
}

impl Trace {
    pub const SIZE: usize = size_of::<Self>() / size_of::<TraceComponent>();

    fn trace_for_player(trace: &mut Self, board: &Board, player: Player) {
        let mut number_of_bishops = 0;

        let mut attacked_squares = Bitboard::EMPTY;

        let occupancy = board.occupancy();

        let our_pawns = board.pawns(player);
        let their_pawns = board.pawns(player.other());

        for sq in board.pawns(player) {
            trace.material[PieceKind::Pawn.array_idx()].incr(player);
            trace.pawn_pst[sq.array_idx()].incr(player);

            // Evaluate passer masks from white's perspective
            if eval::pawn_structure::is_passed(sq, Player::White, their_pawns) {
                trace.passed_pawn_pst[sq.array_idx()].incr(player);
            }
        }

        for file in square::FILES {
            let bb = file.bitboard();

            if (our_pawns & bb).count() > 1 {
                trace.doubled_pawn.incr(player);
            }
        }

        for sq in board.knights(player) {
            trace.material[PieceKind::Knight.array_idx()].incr(player);
            trace.knight_pst[sq.array_idx()].incr(player);

            let attacks = tables::knight_attacks(sq);
            attacked_squares |= attacks;

            let mobility = attacks.count();
            trace.knight_mobility[mobility as usize].incr(player);
        }

        for sq in board.bishops(player) {
            trace.material[PieceKind::Bishop.array_idx()].incr(player);
            trace.bishop_pst[sq.array_idx()].incr(player);

            let attacks = tables::bishop_attacks(sq, occupancy);
            attacked_squares |= attacks;

            let mobility = attacks.count();
            trace.bishop_mobility[mobility as usize].incr(player);

            number_of_bishops += 1;
        }

        if number_of_bishops > 1 {
            trace.bishop_pair.incr(player);
        }

        for sq in board.rooks(player) {
            trace.material[PieceKind::Rook.array_idx()].incr(player);
            trace.rook_pst[sq.array_idx()].incr(player);

            let attacks = tables::rook_attacks(sq, occupancy);
            attacked_squares |= attacks;

            let mobility = attacks.count();
            trace.rook_mobility[mobility as usize].incr(player);
        }

        for sq in board.queens(player) {
            trace.material[PieceKind::Queen.array_idx()].incr(player);
            trace.queen_pst[sq.array_idx()].incr(player);

            let attacks =
                tables::rook_attacks(sq, occupancy) | tables::bishop_attacks(sq, occupancy);
            attacked_squares |= attacks;

            let mobility = attacks.count();
            trace.queen_mobility[mobility as usize].incr(player);
        }

        for sq in board.king(player) {
            trace.material[PieceKind::King.array_idx()].incr(player);
            trace.king_pst[sq.array_idx()].incr(player);
        }

        let enemy_king = board.king(player.other()).single();
        let enemy_king_surrounding_squares = tables::king_attacks(enemy_king);
        let attacks_on_enemy_king = attacked_squares & enemy_king_surrounding_squares;
        trace.attacked_king_squares[attacks_on_enemy_king.count() as usize].incr(player.other());
    }

    pub fn for_game(game: &Game) -> Self {
        let mut trace = Self {
            material: [TraceComponent::default(); PieceKind::N],
            pawn_pst: [TraceComponent::default(); Square::N],
            knight_pst: [TraceComponent::default(); Square::N],
            bishop_pst: [TraceComponent::default(); Square::N],
            rook_pst: [TraceComponent::default(); Square::N],
            queen_pst: [TraceComponent::default(); Square::N],
            king_pst: [TraceComponent::default(); Square::N],

            passed_pawn_pst: [TraceComponent::default(); Square::N],

            knight_mobility: [TraceComponent::default(); 9],
            bishop_mobility: [TraceComponent::default(); 14],
            rook_mobility: [TraceComponent::default(); 15],
            queen_mobility: [TraceComponent::default(); 28],

            attacked_king_squares: [TraceComponent::default(); 9],

            bishop_pair: TraceComponent::default(),
            doubled_pawn: TraceComponent::default(),
        };

        Self::trace_for_player(&mut trace, &game.board, Player::White);
        Self::trace_for_player(&mut trace, &game.board.flip_vertically(), Player::Black);

        trace
    }

    pub fn non_zero_coefficients(&self) -> Vec<NonZeroCoefficient> {
        CoefficientBuilder::new()
            .add(&self.material)
            .add(&self.pawn_pst)
            .add(&self.knight_pst)
            .add(&self.bishop_pst)
            .add(&self.rook_pst)
            .add(&self.queen_pst)
            .add(&self.king_pst)
            .add(&self.passed_pawn_pst)
            .add(&self.knight_mobility)
            .add(&self.bishop_mobility)
            .add(&self.rook_mobility)
            .add(&self.queen_mobility)
            .add(&self.attacked_king_squares)
            .add(&[self.bishop_pair])
            .add(&[self.doubled_pawn])
            .get()
    }
}

struct CoefficientBuilder {
    value: Vec<NonZeroCoefficient>,
    idx: usize,
}

impl CoefficientBuilder {
    pub fn new() -> Self {
        Self {
            value: Vec::new(),
            idx: 0,
        }
    }

    #[expect(clippy::cast_precision_loss, reason = "known cast from i32 to f32")]
    pub fn add(&mut self, s: &[TraceComponent]) -> &mut Self {
        for (i, component) in s.iter().enumerate() {
            let coefficient = component.0;

            if coefficient != 0 {
                self.value
                    .push(NonZeroCoefficient::new(self.idx + i, coefficient as f32));
            }
        }

        self.idx += s.len();
        self
    }

    pub fn get(&self) -> Vec<NonZeroCoefficient> {
        assert_eq!(Trace::SIZE, self.idx);
        self.value.clone()
    }
}
