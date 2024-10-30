use super::{params, PersistentState, SearchState, MAX_SEARCH_DEPTH};
use crate::chess::game::Game;
use crate::chess::moves::Move;
use crate::engine::eval;
use crate::engine::eval::Eval;
use crate::engine::search::move_picker::MovePicker;
use crate::engine::search::principal_variation::PrincipalVariation;
use crate::engine::search::quiescence::quiescence;
use crate::engine::search::tables::lmr_table::lmr_reduction;
use crate::engine::search::time_control::TimeStrategy;
use crate::engine::search::transposition::{NodeBound, SearchTranspositionTableData};

pub fn negamax(
    game: &mut Game,
    mut alpha: Eval,
    beta: Eval,
    mut depth: u8,
    plies: u8,
    persistent_state: &mut PersistentState,
    pv: &mut PrincipalVariation,
    time_control: &mut TimeStrategy,
    state: &mut SearchState,
) -> Eval {
    let is_root = plies == 0;
    let is_pv = alpha != beta - Eval(1);

    // Check periodically to see if we're out of time. If we are, we shouldn't continue the search
    // so we return Err to signal to the caller that the search did not complete.
    if time_control.should_stop(state.nodes_visited) {
        return Eval::ABORTED;
    }

    state.max_depth_reached = state.max_depth_reached.max(plies);

    if !is_root
        && (game.is_repeated_position()
            || game.is_stalemate_by_fifty_move_rule()
            || game.is_stalemate_by_insufficient_material())
    {
        return Eval::DRAW;
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
        );
    }

    if !is_root {
        state.nodes_visited += 1;
    }

    let mut previous_best_move: Option<Move> = None;

    if let Some(tt_entry) = persistent_state.tt.get(&game.zobrist) {
        if !is_root && !is_pv && tt_entry.depth >= depth {
            let tt_score = tt_entry.eval.with_mate_distance_from_root(plies);

            match tt_entry.bound {
                NodeBound::Exact => return tt_score,
                NodeBound::Upper if tt_entry.eval <= alpha => return tt_score,
                NodeBound::Lower if tt_entry.eval >= beta => return tt_score,
                _ => {}
            }
        }

        previous_best_move = tt_entry.best_move;
    }

    let eval = eval::eval(game);

    if !is_root && !is_pv && !in_check {
        // Reverse futility pruning
        if depth <= params::REVERSE_FUTILITY_PRUNE_DEPTH
            && eval - params::REVERSE_FUTILITY_PRUNE_MARGIN_PER_PLY * i16::from(depth) > beta
        {
            return beta;
        }

        // Null move pruning
        if depth >= params::NULL_MOVE_PRUNING_DEPTH_LIMIT
            && eval >= beta
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
                &mut PrincipalVariation::new(),
                time_control,
                state,
            );

            game.undo_null_move();

            if null_score == Eval::ABORTED {
                return Eval::ABORTED;
            }

            if null_score >= beta {
                return null_score;
            }
        }
    }

    let mut tt_node_bound = NodeBound::Upper;
    let mut best_move = None;
    let mut best_eval = Eval::MIN;

    let mut moves = MovePicker::new(previous_best_move);
    let mut number_of_legal_moves = 0;
    let mut node_pv = PrincipalVariation::new();

    while let Some(mv) = moves.next(game, persistent_state, state, plies) {
        node_pv.clear();

        // Futility pruning
        if number_of_legal_moves > 0
            && !is_pv
            && !mv.is_capture()
            && !in_check
            && depth <= params::FUTILITY_PRUNE_DEPTH
            && eval + params::FUTILITY_PRUNE_MAX_MOVE_VALUE < alpha
        {
            continue;
        }

        game.make_move(mv);
        number_of_legal_moves += 1;

        let move_score = if number_of_legal_moves == 1 {
            -negamax(
                game,
                -beta,
                -alpha,
                depth - 1,
                plies + 1,
                persistent_state,
                &mut node_pv,
                time_control,
                state,
            )
        } else {
            let reduction = if depth >= params::LMR_DEPTH
                && number_of_legal_moves >= params::LMR_MOVE_THRESHOLD
                && !in_check
            {
                lmr_reduction(depth, number_of_legal_moves)
            } else {
                1
            };

            // We already found a good move (i.e. we raised alpha).
            // Now, we just need to prove that the other moves are worse.
            // We search them with a reduced window to prove that they are at least worse.
            let pvs_score = -negamax(
                game,
                -alpha - Eval(1),
                -alpha,
                depth.saturating_sub(reduction),
                plies + 1,
                persistent_state,
                &mut node_pv,
                time_control,
                state,
            );

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
                    &mut node_pv,
                    time_control,
                    state,
                )
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
            tt_node_bound = NodeBound::Lower;
            break;
        }

        if move_score > alpha {
            alpha = move_score;
            tt_node_bound = NodeBound::Exact;
            pv.push(mv, &node_pv);
        }

        if move_score == Eval::ABORTED {
            return Eval::ABORTED;
        }
    }

    if number_of_legal_moves == 0 {
        return if game.is_king_in_check() {
            Eval::mated_in(plies)
        } else {
            Eval::DRAW
        };
    }

    if tt_node_bound == NodeBound::Lower {
        let mv = best_move.unwrap();

        // 'Killers': if a move was so good that it caused a beta cutoff,
        // but it wasn't a capture, we remember it so that we can try it
        // before other quiet moves.
        if !mv.is_capture() {
            state.killer_moves.try_push(plies, mv);

            persistent_state
                .history_table
                .add_bonus_for(game.player, mv, depth);
        }
    }

    let tt_data = SearchTranspositionTableData {
        bound: tt_node_bound,
        eval: best_eval.with_mate_distance_from_position(plies),
        best_move,
        age: persistent_state.tt.generation,
        depth,
    };

    persistent_state.tt.insert(&game.zobrist, tt_data);

    best_eval
}
