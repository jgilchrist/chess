use crate::chess::player::Player;
use std::ops::Div;

use crate::engine::eval::WhiteEval;

/// An evaluation from the active player's perspective
///
/// In algorithms like negamax, in order for the same code to work
/// for both players, we need to players to try maximising their score.
/// Therefore, we need a way to represent an evaluation of the board as
/// seen by the active player in any given game state.
///
/// This can be easily turned back into a 'classical' evaluation (i.e.
/// from white's perspective).
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Eval(pub i16);

impl Eval {
    pub(crate) const MAX: Self = Self(i16::MAX);
    pub(crate) const MIN: Self = Self(i16::MIN + 1);
    pub(crate) const ABORTED: Self = Self(i16::MIN);
    pub(crate) const DRAW: Self = Self(0);

    const MATE: i16 = 32000;
    const MATED: i16 = -Self::MATE;
    const MATE_THRESHOLD: i16 = Self::MATE - 100;
    const MATED_THRESHOLD: i16 = -Self::MATE_THRESHOLD;

    pub const fn new(eval: i16) -> Self {
        Self(eval)
    }

    #[expect(unused, reason = "Unused")]
    pub fn mate_in(ply: u8) -> Self {
        Self(Self::MATE - i16::from(ply))
    }

    pub fn mated_in(ply: u8) -> Self {
        Self(-Self::MATE + i16::from(ply))
    }

    pub fn is_mate_in_moves(self) -> Option<i16> {
        if self.0 > Self::MATE_THRESHOLD {
            return Some((Self::MATE - self.0 + 1) / 2);
        }

        if self.0 < Self::MATED_THRESHOLD {
            return Some((Self::MATED - self.0) / 2);
        }

        None
    }

    // When searching, mate scores are relative to the root position.
    // However, we may see the same position at different depths of the
    // tree due to transpositions.
    // As a result, when caching mate evaluations, we need to store them
    // as relative to the position at that point in the tree, rather than
    // relative to the root (by accounting for the difference between the
    // root and the current depth).
    pub fn with_mate_distance_from_position(self, plies: u8) -> Self {
        let mut adjusted_value = self.0;

        if adjusted_value > Self::MATE_THRESHOLD {
            adjusted_value += i16::from(plies);
        }

        if adjusted_value < Self::MATED_THRESHOLD {
            adjusted_value -= i16::from(plies);
        }

        Self(adjusted_value)
    }

    pub fn with_mate_distance_from_root(self, plies: u8) -> Self {
        let mut adjusted_value = self.0;

        if adjusted_value > Self::MATE_THRESHOLD {
            adjusted_value -= i16::from(plies);
        }

        if adjusted_value < Self::MATED_THRESHOLD {
            adjusted_value += i16::from(plies);
        }

        Self(adjusted_value)
    }

    #[expect(unused, reason = "Unused")]
    pub fn to_white_eval(self, player: Player) -> WhiteEval {
        match player {
            Player::White => WhiteEval(self.0),
            Player::Black => -WhiteEval(self.0),
        }
    }

    pub fn from_white_eval(eval: WhiteEval, player: Player) -> Self {
        match player {
            Player::White => Self(eval.0),
            Player::Black => Self((-eval).0),
        }
    }
}

impl std::ops::Add for Eval {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0 + rhs.0)
    }
}

impl std::ops::AddAssign for Eval {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl std::ops::Sub for Eval {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0 - rhs.0)
    }
}

impl std::ops::SubAssign for Eval {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl std::ops::Mul<i16> for Eval {
    type Output = Self;

    fn mul(self, rhs: i16) -> Self::Output {
        Self(self.0 * rhs)
    }
}

impl std::ops::Neg for Eval {
    type Output = Self;

    fn neg(self) -> Self::Output {
        Self(self.0.saturating_neg())
    }
}

impl Div<i16> for Eval {
    type Output = Self;

    fn div(self, rhs: i16) -> Self::Output {
        Self(self.0 / rhs)
    }
}
