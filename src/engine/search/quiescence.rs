use crate::chess::game::Game;
use crate::engine::eval;
use crate::engine::eval::Eval;
use crate::engine::search::move_picker::MovePicker;
use crate::engine::search::time_control::TimeStrategy;

use super::{PersistentState, SearchState, MAX_SEARCH_DEPTH};

pub fn quiescence(
    game: &mut Game,
    mut alpha: Eval,
    beta: Eval,
    plies: u8,
    time_control: &mut TimeStrategy,
    persistent_state: &mut PersistentState,
    state: &mut SearchState,
) -> Result<Eval, ()> {
    state.max_depth_reached = state.max_depth_reached.max(plies);
    state.nodes_visited += 1;

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
    if time_control.should_stop(state.nodes_visited) {
        return Err(());
    }

    let eval = eval::eval(game);

    if eval >= beta {
        return Ok(eval);
    }

    if eval > alpha {
        alpha = eval;
    }

    let mut best_eval = eval;

    let mut moves = MovePicker::new_loud();
    while let Some(mv) = moves.next(game, persistent_state, state, plies) {
        game.make_move(mv);

        let move_score = -quiescence(
            game,
            -beta,
            -alpha,
            plies + 1,
            time_control,
            persistent_state,
            state,
        )?;

        game.undo_move();

        if move_score > best_eval {
            best_eval = move_score;
        }

        // Cutoff: This move is so good that our opponent won't let it be played.
        if move_score >= beta {
            break;
        }

        if move_score > alpha {
            alpha = move_score;
        }
    }

    Ok(best_eval)
}
