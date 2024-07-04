use crate::chess::game::Game;
use crate::chess::moves::Move;
use crate::engine::eval;
use crate::engine::eval::Eval;
use crate::engine::search::move_provider::MoveProvider;
use crate::engine::search::quiescence::quiescence;
use crate::engine::search::time_control::TimeStrategy;
use crate::engine::search::transposition::{NodeBound, SearchTranspositionTableData, TTMove};

use super::{move_ordering, params, Control, PersistentState, SearchState, MAX_SEARCH_DEPTH};

pub fn negamax(
    game: &mut Game,
    mut alpha: Eval,
    beta: Eval,
    mut depth: u8,
    plies: u8,
    persistent_state: &mut PersistentState,
    time_control: &TimeStrategy,
    state: &mut SearchState,
    control: &impl Control,
) -> Result<Eval, ()> {
    let is_root = plies == 0;

    // Check periodically to see if we're out of time. If we are, we shouldn't continue the search
    // so we return Err to signal to the caller that the search did not complete.
    if (is_root || (state.nodes_visited % params::CHECK_TERMINATION_NODE_FREQUENCY == 0))
        && (time_control.should_stop() || control.should_stop())
    {
        return Err(());
    }

    state.max_depth_reached = state.max_depth_reached.max(plies);

    // Keep track of whether we're doing a full search. If we raised alpha at this node, we've found
    // a new PV (or re-confirmed the PV we found at a previous search depth) - so for the remainder
    // of moves we search, we just need to check that they're worse. We can do this with more restrictive
    // alpha & beta bounds, and thus search less of the tree.
    let mut full_pv_search = true;

    if !is_root
        && (game.is_repeated_position()
            || game.is_stalemate_by_fifty_move_rule()
            || game.is_stalemate_by_insufficient_material())
    {
        return Ok(Eval::DRAW);
    }

    // Check extension: If we're about to finish searching, but we are in check, we
    // should keep going.
    let in_check = game.is_king_in_check();
    if in_check && depth < MAX_SEARCH_DEPTH {
        depth += 1;
    }

    if depth == 0 {
        return quiescence(
            game,
            alpha,
            beta,
            plies,
            time_control,
            persistent_state,
            state,
            control,
        );
    }

    if !is_root {
        state.nodes_visited += 1;
    }

    let mut previous_best_move: Option<Move> = None;

    if let Some(tt_entry) = persistent_state.tt.get(&game.zobrist) {
        if !is_root && tt_entry.depth >= depth {
            match tt_entry.bound {
                NodeBound::Exact => return Ok(tt_entry.eval.with_mate_distance_from_root(plies)),
                NodeBound::Upper if tt_entry.eval <= alpha => return Ok(alpha),
                NodeBound::Lower if tt_entry.eval >= beta => return Ok(beta),
                _ => {}
            }
        }

        previous_best_move = tt_entry.best_move.as_ref().map(TTMove::to_move);
    }

    if !is_root && !in_check {
        let eval = eval::eval(game);

        // Reverse futility pruning
        if depth <= params::REVERSE_FUTILITY_PRUNE_DEPTH
            && eval - params::REVERSE_FUTILITY_PRUNE_MARGIN_PER_PLY * i16::from(depth) > beta
        {
            return Ok(beta);
        }

        // Null move pruning
        if depth >= params::NULL_MOVE_PRUNING_DEPTH_LIMIT
            && eval > beta
            // Don't let a player play a null move in response to a null move
            && game.history.last().map_or(true, |m| m.mv.is_some())
        {
            game.make_null_move();

            let null_score = -negamax(
                game,
                -beta,
                -beta + Eval(1),
                depth - 1 - params::NULL_MOVE_PRUNING_DEPTH_REDUCTION,
                plies + 1,
                persistent_state,
                time_control,
                state,
                control,
            )?;

            game.undo_null_move();

            if null_score >= beta {
                return Ok(null_score);
            }
        }
    }

    let mut tt_node_bound = NodeBound::Upper;
    let mut best_move = None;
    let mut best_eval = Eval::MIN;

    let mut moves = MoveProvider::new(previous_best_move);
    let mut number_of_legal_moves = 0;

    while let Some(mv) = moves.next(game, persistent_state, state, plies as usize) {
        number_of_legal_moves += 1;

        game.make_move(mv);

        let move_score = if full_pv_search {
            -negamax(
                game,
                -beta,
                -alpha,
                depth - 1,
                plies + 1,
                persistent_state,
                time_control,
                state,
                control,
            )?
        } else {
            // We already found a good move (i.e. we raised alpha).
            // Now, we just need to prove that the other moves are worse.
            // We search them with a reduced window to prove that they are at least worse.
            let pvs_score = -negamax(
                game,
                -alpha - Eval(1),
                -alpha,
                depth - 1,
                plies + 1,
                persistent_state,
                time_control,
                state,
                control,
            )?;

            // Turns out the move we just searched could be better than our current PV, so we re-search
            // with the normal alpha/beta bounds.
            if pvs_score > alpha && pvs_score < beta {
                -negamax(
                    game,
                    -beta,
                    -alpha,
                    depth - 1,
                    plies + 1,
                    persistent_state,
                    time_control,
                    state,
                    control,
                )?
            } else {
                pvs_score
            }
        };

        game.undo_move();

        if move_score > best_eval {
            best_move = Some(mv);
            best_eval = move_score;
        }

        // Cutoff: This move is so good that our opponent won't let it be played.
        if move_score >= beta {
            let tt_data = SearchTranspositionTableData {
                bound: NodeBound::Lower,
                eval: move_score.with_mate_distance_from_position(plies),
                best_move: None,
                depth,
                age: persistent_state.tt.generation,
            };

            persistent_state.tt.insert(&game.zobrist, tt_data);

            // 'Killers': if a move was so good that it caused a beta cutoff,
            // but it wasn't a capture, we remember it so that we can try it
            // before other quiet moves.
            if game.board.piece_at(mv.dst).is_none() {
                let killer_1 = state.killer_moves[plies as usize][0];

                if Some(mv) != killer_1 {
                    state.killer_moves[plies as usize][1] = killer_1;
                    state.killer_moves[plies as usize][0] = Some(mv);
                }

                let new_history = persistent_state.history[game.player.array_idx()]
                    [mv.src.array_idx()][mv.dst.array_idx()]
                    + i32::from(depth) * i32::from(depth);

                persistent_state.history[game.player.array_idx()][mv.src.array_idx()]
                    [mv.dst.array_idx()] =
                    std::cmp::min(new_history, move_ordering::HISTORY_MAX_SCORE);
            }

            return Ok(beta);
        }

        if move_score > alpha {
            alpha = move_score;
            tt_node_bound = NodeBound::Exact;

            // We've found a PV move, so we can try and prove that the rest of the moves in this
            // position are worse.
            full_pv_search = false;
        }
    }

    if number_of_legal_moves == 0 {
        return Ok(if game.is_king_in_check() {
            Eval::mated_in(plies)
        } else {
            Eval::DRAW
        });
    }

    let tt_data = SearchTranspositionTableData {
        bound: tt_node_bound,
        eval: alpha,
        best_move: best_move.map(TTMove::from_move),
        age: persistent_state.tt.generation,
        depth,
    };

    persistent_state.tt.insert(&game.zobrist, tt_data);

    Ok(alpha)
}
