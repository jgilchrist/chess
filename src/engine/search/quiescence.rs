use super::{SearchContext, MAX_SEARCH_DEPTH};
use crate::chess::game::Game;
use crate::chess::moves::Move;
use crate::engine::eval;
use crate::engine::eval::Eval;
use crate::engine::search::move_picker::MovePicker;
use crate::engine::search::transposition::{NodeBound, SearchTranspositionTableData};

pub fn quiescence(
    game: &mut Game,
    mut alpha: Eval,
    beta: Eval,
    plies: u8,
    ctx: &mut SearchContext<'_>,
) -> Result<Eval, ()> {
    ctx.max_depth_reached = ctx.max_depth_reached.max(plies);
    ctx.nodes_visited += 1;

    if plies == MAX_SEARCH_DEPTH {
        return Ok(eval::eval(game));
    }

    if game.is_repeated_position()
        || game.is_stalemate_by_fifty_move_rule()
        || game.is_stalemate_by_insufficient_material()
    {
        return Ok(Eval::DRAW);
    }

    // Check periodically to see if we're out of time. If we are, we shouldn't continue the search
    // so we return Err to signal to the caller that the search did not complete.
    if ctx.time_control.should_stop(ctx.nodes_visited) {
        return Err(());
    }

    let mut previous_best_move: Option<Move> = None;

    if let Some(tt_entry) = ctx.tt.get(&game.zobrist) {
        let tt_score = tt_entry.eval.with_mate_distance_from_root(plies);

        match tt_entry.bound {
            NodeBound::Exact => return Ok(tt_score),
            NodeBound::Upper if tt_entry.eval <= alpha => return Ok(tt_score),
            NodeBound::Lower if tt_entry.eval >= beta => return Ok(tt_score),
            _ => {}
        }

        previous_best_move = tt_entry.best_move;
    }

    let eval = eval::eval(game);

    if eval >= beta {
        return Ok(eval);
    }

    if eval > alpha {
        alpha = eval;
    }

    let mut best_eval = eval;
    let mut tt_node_bound = NodeBound::Upper;
    let mut best_move = None;

    let mut moves = MovePicker::new_loud(previous_best_move);
    while let Some(mv) = moves.next(game, ctx, plies) {
        game.make_move(mv);

        let move_score = -quiescence(game, -beta, -alpha, plies + 1, ctx)?;

        game.undo_move();

        if move_score > best_eval {
            best_eval = move_score;
        }

        // Cutoff: This move is so good that our opponent won't let it be played.
        if move_score >= beta {
            tt_node_bound = NodeBound::Lower;
            break;
        }

        if move_score > alpha {
            tt_node_bound = NodeBound::Exact;
            alpha = move_score;
            best_move = Some(mv);
        }
    }

    let tt_data = SearchTranspositionTableData {
        bound: tt_node_bound,
        eval: best_eval.with_mate_distance_from_position(plies),
        best_move,
        age: ctx.tt.generation,
        depth: plies,
    };

    ctx.tt.insert(&game.zobrist, tt_data);

    Ok(best_eval)
}
