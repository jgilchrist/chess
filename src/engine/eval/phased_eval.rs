use crate::chess::board::Board;
use crate::chess::piece::PieceKind;
use crate::chess::square::Square;
use crate::engine::eval::WhiteEval;
use crate::engine::transposition_table::TTOverwriteable;

const PHASE_COUNT_MAX: i64 = 24;

/// A midgame and endgame evaluation
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct PhasedEval(i32);

impl PhasedEval {
    pub const ZERO: Self = Self(0);

    pub const fn new(midgame: i16, endgame: i16) -> Self {
        Self(((endgame as i32) << 16) + midgame as i32)
    }

    #[expect(
        clippy::cast_possible_truncation,
        reason = "Intentional truncation to get the bottom 16 bits"
    )]
    pub fn midgame(self) -> WhiteEval {
        WhiteEval(self.0 as i16)
    }

    pub fn endgame(self) -> WhiteEval {
        WhiteEval(((self.0 + 0x8000) >> 16) as i16)
    }

    pub fn for_phase(self, phase_value: i16) -> WhiteEval {
        // Switch to 64 bit calculations to avoid overflow
        let phase_value = i64::from(phase_value);

        let midgame_phase_value = phase_value.min(PHASE_COUNT_MAX);
        let endgame_phase_value = PHASE_COUNT_MAX - phase_value;

        let midgame_eval = i64::from(self.midgame().0);
        let endgame_eval = i64::from(self.endgame().0);

        let eval = (midgame_eval * midgame_phase_value + endgame_eval * endgame_phase_value) / 24;
        WhiteEval(i16::try_from(eval).unwrap())
    }
}

impl std::fmt::Debug for PhasedEval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "PhasedEval({}, {})", self.midgame().0, self.endgame().0)
    }
}

impl TTOverwriteable for PhasedEval {
    fn should_overwrite_with(&self, _: &Self) -> bool {
        true
    }
}

impl std::ops::Add for PhasedEval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for PhasedEval {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for PhasedEval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for PhasedEval {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Neg for PhasedEval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.saturating_neg())
    }
}

pub fn piece_phase_value_contribution(kind: PieceKind) -> i16 {
    match kind {
        PieceKind::Pawn | PieceKind::King => 0,
        PieceKind::Knight | PieceKind::Bishop => 1,
        PieceKind::Rook => 2,
        PieceKind::Queen => 4,
    }
}

pub fn phase_value(board: &Board) -> i16 {
    let mut v = 0;

    for idx in 0..Square::N {
        let square = Square::from_array_index(idx);

        let maybe_piece = board.piece_at(square);

        if let Some(piece) = maybe_piece {
            v += piece_phase_value_contribution(piece.kind);
        }
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_calculation_does_not_overflow() {
        let phased_eval = PhasedEval::new(2456, 2393);
        let phase_value = 9;

        let eval = phased_eval.for_phase(phase_value);
        assert!(eval > WhiteEval(0));
    }

    #[test]
    fn test_eval_retains_midgame_and_endgame_vals() {
        let phased_eval = PhasedEval::new(2456, 2393);

        assert_eq!(phased_eval.midgame().0, 2456);
        assert_eq!(phased_eval.endgame().0, 2393);
    }
}
