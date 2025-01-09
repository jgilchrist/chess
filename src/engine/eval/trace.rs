use crate::chess::piece::PieceKind;
use crate::chess::player::Player;
use crate::chess::square::Square;

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
    pub material: [TraceComponent; PieceKind::N],

    pub pawn_pst: [TraceComponent; Square::N],
    pub knight_pst: [TraceComponent; Square::N],
    pub bishop_pst: [TraceComponent; Square::N],
    pub rook_pst: [TraceComponent; Square::N],
    pub queen_pst: [TraceComponent; Square::N],
    pub king_pst: [TraceComponent; Square::N],

    pub passed_pawn_pst: [TraceComponent; Square::N],

    pub knight_mobility: [TraceComponent; 9],
    pub bishop_mobility: [TraceComponent; 14],
    pub rook_mobility: [TraceComponent; 15],
    pub queen_mobility: [TraceComponent; 28],

    pub attacked_king_squares: [TraceComponent; 9],

    pub bishop_pair: TraceComponent,
    pub connected_rooks: TraceComponent,
}

impl Trace {
    pub const SIZE: usize = size_of::<Self>() / size_of::<TraceComponent>();

    pub fn new() -> Self {
        Self {
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
            connected_rooks: TraceComponent::default(),
        }
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
            .add(&[self.connected_rooks])
            .get()
    }
}

#[derive(Clone)]
pub struct NonZeroCoefficient {
    pub idx: usize,
    pub value: f32,
}

impl NonZeroCoefficient {
    pub fn new(idx: usize, value: f32) -> Self {
        Self { idx, value }
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
