#![cfg_attr(any(), rustfmt::skip)]

use crate::chess::piece::PieceKind;
use crate::chess::square::{File, Rank};
use crate::engine::eval::phased_eval::s;
use crate::engine::eval::PhasedEval;

pub type PieceSquareTableDefinition = [[PhasedEval; File::N]; Rank::N];

pub const PIECE_VALUES: [PhasedEval; PieceKind::N] = [
    s(  115,   222),
    s(  390,   483),
    s(  411,   506),
    s(  555,   904),
    s( 1176,  1670),
    s(    0,     0)
];

pub const PAWNS: PieceSquareTableDefinition = [
    [s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0)],
    [s(   82,   181), s(  114,   170), s(   83,   171), s(  125,   103), s(  100,    96), s(   77,   111), s(  -18,   175), s(  -55,   195)],
    [s(  -30,    98), s(  -11,   109), s(   36,    60), s(   43,    31), s(   47,    19), s(   79,    -2), s(   51,    64), s(  -10,    61)],
    [s(  -52,    -2), s(  -17,   -16), s(  -12,   -44), s(  -10,   -56), s(   20,   -69), s(    9,   -65), s(   15,   -38), s(  -19,   -37)],
    [s(  -67,   -36), s(  -28,   -40), s(  -30,   -65), s(   -7,   -69), s(   -6,   -72), s(  -17,   -70), s(   -5,   -53), s(  -37,   -64)],
    [s(  -69,   -45), s(  -34,   -42), s(  -34,   -67), s(  -32,   -49), s(  -11,   -59), s(  -27,   -64), s(   15,   -56), s(  -27,   -69)],
    [s(  -68,   -38), s(  -33,   -36), s(  -39,   -56), s(  -53,   -46), s(  -25,   -40), s(   -4,   -57), s(   28,   -57), s(  -38,   -68)],
    [s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0), s(    0,     0)],
];

pub const KNIGHTS: PieceSquareTableDefinition = [
    [s( -204,  -103), s( -158,   -23), s(  -65,    -5), s(  -18,   -15), s(   27,   -11), s(  -54,   -44), s( -132,   -15), s( -123,  -137)],
    [s(  -16,   -33), s(    8,    -3), s(   45,     9), s(   66,     7), s(   44,    -4), s(  133,   -26), s(    6,    -9), s(   39,   -56)],
    [s(    4,   -11), s(   52,    11), s(   77,    34), s(   93,    36), s(  145,    13), s(  146,     5), s(   86,    -5), s(   41,   -26)],
    [s(   -2,     6), s(   17,    36), s(   51,    50), s(   82,    53), s(   56,    55), s(   90,    46), s(   32,    34), s(   46,    -5)],
    [s(  -20,     9), s(    1,    21), s(   22,    54), s(   24,    54), s(   37,    58), s(   29,    44), s(   27,    23), s(   -6,    -4)],
    [s(  -47,   -13), s(  -15,    13), s(    4,    28), s(    8,    45), s(   24,    43), s(   10,    22), s(   15,     6), s(  -24,   -12)],
    [s(  -66,   -26), s(  -48,    -5), s(  -25,     9), s(   -9,    15), s(   -7,    14), s(   -4,     6), s(  -23,   -18), s(  -27,   -13)],
    [s( -127,   -37), s(  -51,   -54), s(  -69,   -12), s(  -49,    -8), s(  -42,    -6), s(  -25,   -23), s(  -48,   -44), s(  -84,   -55)],
];

pub const BISHOPS: PieceSquareTableDefinition = [
    [s(  -37,   -11), s(  -61,     1), s(  -48,    -1), s( -108,    17), s(  -90,     9), s(  -66,    -5), s(  -27,   -11), s(  -76,   -18)],
    [s(  -14,   -33), s(   19,    -5), s(   10,     0), s(  -14,     6), s(   29,    -9), s(   27,    -8), s(   15,     1), s(    1,   -34)],
    [s(    0,     7), s(   33,     2), s(   33,    17), s(   65,     0), s(   47,     8), s(   92,     9), s(   60,    -1), s(   41,    -2)],
    [s(  -12,     1), s(    7,    25), s(   39,    18), s(   53,    39), s(   49,    24), s(   43,    23), s(    8,    19), s(  -11,     1)],
    [s(  -22,    -5), s(   -3,    21), s(    5,    32), s(   34,    27), s(   30,    29), s(    8,    24), s(   -2,    17), s(  -11,   -18)],
    [s(   -7,    -6), s(    3,    10), s(    3,    20), s(    7,    21), s(    9,    26), s(    2,    22), s(    5,    -2), s(   11,   -19)],
    [s(   -4,   -12), s(   -2,   -13), s(   14,   -15), s(  -17,     6), s(   -6,     8), s(   12,    -7), s(   21,    -3), s(    2,   -40)],
    [s(  -36,   -37), s(   -6,   -12), s(  -29,   -40), s(  -40,   -11), s(  -34,   -15), s(  -36,   -14), s(   -1,   -32), s(  -21,   -56)],
];

pub const ROOKS: PieceSquareTableDefinition = [
    [s(   42,    11), s(   27,    23), s(   37,    35), s(   45,    29), s(   71,    16), s(   95,     1), s(   68,     5), s(   99,    -3)],
    [s(   14,    12), s(   14,    28), s(   41,    34), s(   68,    22), s(   48,    22), s(   89,     2), s(   72,    -4), s(  114,   -23)],
    [s(  -15,    13), s(   15,    16), s(   17,    18), s(   22,    16), s(   62,    -3), s(   63,   -11), s(  116,   -23), s(   84,   -31)],
    [s(  -37,    16), s(  -17,    13), s(  -15,    26), s(   -3,    21), s(    5,    -1), s(    8,    -9), s(   20,   -14), s(   22,   -23)],
    [s(  -64,     5), s(  -60,    12), s(  -46,    15), s(  -28,    13), s(  -28,     6), s(  -50,     4), s(  -16,   -15), s(  -27,   -22)],
    [s(  -73,    -1), s(  -60,    -1), s(  -48,    -2), s(  -50,     5), s(  -41,    -2), s(  -44,   -13), s(    5,   -42), s(  -26,   -41)],
    [s(  -77,    -8), s(  -60,    -4), s(  -39,    -2), s(  -44,     1), s(  -38,   -11), s(  -35,   -17), s(   -9,   -31), s(  -52,   -23)],
    [s(  -51,   -15), s(  -48,    -2), s(  -35,    10), s(  -28,     8), s(  -22,    -3), s(  -36,   -10), s(  -15,   -17), s(  -48,   -27)],
];

pub const QUEENS: PieceSquareTableDefinition = [
    [s(  -51,     1), s(  -42,    22), s(    4,    44), s(   51,    24), s(   47,    23), s(   62,     6), s(   86,   -54), s(    9,    -9)],
    [s(    1,   -48), s(  -30,    13), s(  -22,    62), s(  -32,    86), s(  -23,   110), s(   30,    52), s(    1,    28), s(   63,    -8)],
    [s(    1,   -33), s(   -2,    -7), s(   -5,    51), s(   18,    55), s(   25,    74), s(   83,    45), s(   86,    -9), s(   82,   -26)],
    [s(  -22,   -18), s(  -16,    14), s(  -11,    36), s(  -12,    67), s(   -9,    87), s(   10,    66), s(    9,    45), s(   18,    13)],
    [s(  -20,   -23), s(  -23,    19), s(  -24,    31), s(  -12,    60), s(  -13,    56), s(  -15,    43), s(    1,    16), s(    5,    -2)],
    [s(  -23,   -38), s(  -13,   -15), s(  -20,    19), s(  -21,    13), s(  -17,    19), s(   -7,     8), s(   11,   -24), s(    2,   -42)],
    [s(  -26,   -46), s(  -18,   -39), s(   -4,   -44), s(   -4,   -30), s(   -6,   -26), s(    6,   -63), s(   14,  -104), s(   30,  -146)],
    [s(  -29,   -53), s(  -43,   -43), s(  -33,   -39), s(  -11,   -52), s(  -24,   -45), s(  -43,   -45), s(   -9,   -90), s(  -21,   -89)],
];

pub const KING: PieceSquareTableDefinition = [
    [s(    5,  -111), s(  -22,   -42), s(   27,   -29), s( -168,    41), s(  -89,    12), s(  -16,    16), s(   55,     4), s(  150,  -135)],
    [s( -150,    18), s(  -91,    59), s( -152,    77), s(   -2,    50), s(  -73,    79), s(  -67,    96), s(  -10,    81), s(  -40,    37)],
    [s( -176,    39), s(  -30,    65), s( -125,    92), s( -152,   107), s(  -97,   106), s(   11,    95), s(  -17,    94), s(  -67,    53)],
    [s( -129,    25), s( -146,    73), s( -166,    98), s( -230,   116), s( -213,   116), s( -159,   107), s( -157,    95), s( -191,    59)],
    [s( -117,     8), s( -134,    52), s( -176,    86), s( -215,   106), s( -211,   105), s( -160,    87), s( -164,    70), s( -198,    47)],
    [s(  -57,    -8), s(  -35,    26), s( -115,    56), s( -133,    74), s( -124,    73), s( -121,    61), s(  -56,    33), s(  -80,    16)],
    [s(   70,   -38), s(   10,     1), s(  -10,    19), s(  -58,    35), s(  -61,    39), s(  -36,    26), s(   34,    -1), s(   47,   -26)],
    [s(   62,   -87), s(   96,   -60), s(   58,   -32), s(  -85,    -6), s(    7,   -42), s(  -49,    -8), s(   68,   -46), s(   70,   -87)],
];

pub const BISHOP_PAIR_BONUS: PhasedEval = s(   28,    92);
