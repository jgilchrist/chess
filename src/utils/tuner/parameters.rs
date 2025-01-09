use crate::chess::bitboard::{bitboards, Bitboard};
use crate::chess::piece::PieceKind;
use crate::chess::square::Square;
use crate::engine::eval::trace::Trace;
use crate::engine::eval::PhasedEval;
use crate::utils::tuner::tuner_eval::TunerEval;
use std::fmt::Formatter;

pub struct ParametersBuilder {
    parameters: [PhasedEval; Trace::SIZE],
    idx: usize,
}

impl ParametersBuilder {
    pub fn new(parameters: &[PhasedEval; Trace::SIZE]) -> Self {
        Self {
            parameters: *parameters,
            idx: 0,
        }
    }

    pub fn copy_to(mut self, ps: &mut [PhasedEval]) -> Self {
        ps.copy_from_slice(&self.parameters[self.idx..self.idx + ps.len()]);
        self.idx += ps.len();
        self
    }
}

#[derive(Clone)]
pub struct Parameters {
    material: [PhasedEval; PieceKind::N],

    pawn_pst: [PhasedEval; Square::N],
    knight_pst: [PhasedEval; Square::N],
    bishop_pst: [PhasedEval; Square::N],
    rook_pst: [PhasedEval; Square::N],
    queen_pst: [PhasedEval; Square::N],
    king_pst: [PhasedEval; Square::N],

    passed_pawn_pst: [PhasedEval; Square::N],

    knight_mobility: [PhasedEval; 9],
    bishop_mobility: [PhasedEval; 14],
    rook_mobility: [PhasedEval; 15],
    queen_mobility: [PhasedEval; 28],

    attacked_king_squares: [PhasedEval; 9],

    bishop_pair: [PhasedEval; 1],
    connected_rooks: [PhasedEval; 1],
}

impl Parameters {
    pub fn new() -> Self {
        Self {
            material: [PhasedEval::ZERO; PieceKind::N],

            pawn_pst: [PhasedEval::ZERO; Square::N],
            knight_pst: [PhasedEval::ZERO; Square::N],
            bishop_pst: [PhasedEval::ZERO; Square::N],
            rook_pst: [PhasedEval::ZERO; Square::N],
            queen_pst: [PhasedEval::ZERO; Square::N],
            king_pst: [PhasedEval::ZERO; Square::N],

            passed_pawn_pst: [PhasedEval::ZERO; Square::N],

            knight_mobility: [PhasedEval::ZERO; 9],
            bishop_mobility: [PhasedEval::ZERO; 14],
            rook_mobility: [PhasedEval::ZERO; 15],
            queen_mobility: [PhasedEval::ZERO; 28],

            attacked_king_squares: [PhasedEval::ZERO; 9],

            bishop_pair: [PhasedEval::ZERO; 1],
            connected_rooks: [PhasedEval::ZERO; 1],
        }
    }

    pub fn from_array(arr: &[TunerEval; Trace::SIZE]) -> Self {
        let mut evals = [PhasedEval::ZERO; Trace::SIZE];

        for (i, param) in arr.iter().enumerate() {
            evals[i] = param.to_phased_eval();
        }

        let mut parameter_components = Self::new();

        ParametersBuilder::new(&evals)
            .copy_to(&mut parameter_components.material)
            .copy_to(&mut parameter_components.pawn_pst)
            .copy_to(&mut parameter_components.knight_pst)
            .copy_to(&mut parameter_components.bishop_pst)
            .copy_to(&mut parameter_components.rook_pst)
            .copy_to(&mut parameter_components.queen_pst)
            .copy_to(&mut parameter_components.king_pst)
            .copy_to(&mut parameter_components.passed_pawn_pst)
            .copy_to(&mut parameter_components.knight_mobility)
            .copy_to(&mut parameter_components.bishop_mobility)
            .copy_to(&mut parameter_components.rook_mobility)
            .copy_to(&mut parameter_components.queen_mobility)
            .copy_to(&mut parameter_components.attacked_king_squares)
            .copy_to(&mut parameter_components.bishop_pair)
            .copy_to(&mut parameter_components.connected_rooks);

        parameter_components.rebalance();

        parameter_components
    }

    fn rebalance_pst(
        pst: &mut [PhasedEval; Square::N],
        material: &mut [PhasedEval; PieceKind::N],
        piece: PieceKind,
        ignore_mask: Bitboard,
    ) {
        let mut midgame_sum = 0;
        let mut endgame_sum = 0;

        // We won't include some squares in the calculation - e.g. ranks where pawns can never be
        let squares = Bitboard::FULL & !ignore_mask;

        // First, calculate the average across all non-zero squares in the PST
        // Do our computations in 32 bits to avoid overflowing i16 for large PST values.
        for sq in squares {
            let v = pst[sq.array_idx()];
            midgame_sum += i32::from(v.midgame().0);
            endgame_sum += i32::from(v.endgame().0);
        }

        let mg_average = midgame_sum / i32::from(squares.count());
        let eg_average = endgame_sum / i32::from(squares.count());

        let average = PhasedEval::new(
            i16::try_from(mg_average).unwrap(),
            i16::try_from(eg_average).unwrap(),
        );
        material[piece.array_idx()] += average;

        for sq in squares {
            pst[sq.array_idx()] -= average;
        }
    }

    pub fn rebalance(&mut self) {
        Self::rebalance_pst(
            &mut self.pawn_pst,
            &mut self.material,
            PieceKind::Pawn,
            bitboards::RANK_1 | bitboards::RANK_8,
        );
        Self::rebalance_pst(
            &mut self.knight_pst,
            &mut self.material,
            PieceKind::Knight,
            Bitboard::EMPTY,
        );
        Self::rebalance_pst(
            &mut self.bishop_pst,
            &mut self.material,
            PieceKind::Bishop,
            Bitboard::EMPTY,
        );
        Self::rebalance_pst(
            &mut self.rook_pst,
            &mut self.material,
            PieceKind::Rook,
            Bitboard::EMPTY,
        );
        Self::rebalance_pst(
            &mut self.queen_pst,
            &mut self.material,
            PieceKind::Queen,
            Bitboard::EMPTY,
        );
    }
}

fn print_param(f: &mut Formatter<'_>, p: PhasedEval) -> std::fmt::Result {
    let (mg, eg) = (p.midgame().0, p.endgame().0);
    write!(f, "s({mg: >5}, {eg: >5})")
}

fn print_array(f: &mut Formatter<'_>, ps: &[PhasedEval], name: &str) -> std::fmt::Result {
    let size = ps.len();
    writeln!(f, "pub const {name}: [PhasedEval; {size}] = [")?;

    for param in ps {
        write!(f, "    ")?;
        print_param(f, *param)?;
        writeln!(f, ",")?;
    }

    writeln!(f, "];\n")?;

    Ok(())
}

fn print_pst(f: &mut Formatter<'_>, pst: &[PhasedEval; Square::N], name: &str) -> std::fmt::Result {
    writeln!(f, "pub const {name}: PieceSquareTableDefinition = [")?;

    for rank in (0..8).rev() {
        write!(f, "    [")?;

        for file in 0..8 {
            let idx = Square::from_idxs(file, rank).array_idx();
            print_param(f, pst[idx])?;

            if file != 7 {
                write!(f, ", ")?;
            }
        }

        writeln!(f, "],")?;
    }

    writeln!(f, "];\n")?;

    Ok(())
}

fn print_single(f: &mut Formatter<'_>, p: [PhasedEval; 1], name: &str) -> std::fmt::Result {
    write!(f, "pub const {name}: PhasedEval = ")?;
    print_param(f, p[0])?;
    writeln!(f, ";\n")?;

    Ok(())
}

impl std::fmt::Display for Parameters {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        print_array(f, &self.material, "PIECE_VALUES")?;
        print_pst(f, &self.pawn_pst, "PAWNS")?;
        print_pst(f, &self.knight_pst, "KNIGHTS")?;
        print_pst(f, &self.bishop_pst, "BISHOPS")?;
        print_pst(f, &self.rook_pst, "ROOKS")?;
        print_pst(f, &self.queen_pst, "QUEENS")?;
        print_pst(f, &self.king_pst, "KING")?;
        print_pst(f, &self.passed_pawn_pst, "PASSED_PAWNS")?;
        print_array(f, &self.knight_mobility, "KNIGHT_MOBILITY")?;
        print_array(f, &self.bishop_mobility, "BISHOP_MOBILITY")?;
        print_array(f, &self.rook_mobility, "ROOK_MOBILITY")?;
        print_array(f, &self.queen_mobility, "QUEEN_MOBILITY")?;
        print_array(f, &self.attacked_king_squares, "ATTACKED_KING_SQUARES")?;
        print_single(f, self.bishop_pair, "BISHOP_PAIR_BONUS")?;
        print_single(f, self.connected_rooks, "CONNECTED_ROOK_BONUS")?;

        Ok(())
    }
}
