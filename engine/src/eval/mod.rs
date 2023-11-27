pub mod piece_square_tables;

use crate::game::EngineGame;

pub fn init() {
    piece_square_tables::init();
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub struct Eval(pub i16);

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

#[allow(clippy::cast_precision_loss)]
impl std::fmt::Display for Eval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let formatted_value = f32::from(self.0) / 100.0;
        write!(f, "{formatted_value}")
    }
}

pub fn eval(game: &EngineGame) -> Eval {
    piece_square_tables::tapered_eval(game.phase_value, game.midgame_eval, game.endgame_eval)
}

#[derive(Debug)]
pub struct EvalComponents {
    pub eval: Eval,
    pub piece_square_midgame: Eval,
    pub piece_square_endgame: Eval,
    pub phase_value: i16,
    pub piece_square_tables: Eval,
}

pub fn eval_components(game: &EngineGame) -> EvalComponents {
    let eval = eval(game);

    let (midgame_pst, endgame_pst) = piece_square_tables::phase_evals(&game.game.board);
    let phase_value = piece_square_tables::phase_value(&game.game.board);

    let pst_eval = piece_square_tables::tapered_eval(phase_value, midgame_pst, endgame_pst);

    EvalComponents {
        eval,
        piece_square_midgame: midgame_pst,
        piece_square_endgame: endgame_pst,
        phase_value,
        piece_square_tables: pst_eval,
    }
}
