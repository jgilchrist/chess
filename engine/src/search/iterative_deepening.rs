use crate::eval::Eval;
use crate::game::EngineGame;
use crate::options::EngineOptions;
use crate::search::time_control::TimeStrategy;
use crate::search::transposition::{NodeBound, SearchTranspositionTable};
use crate::search::{negamax, SearchState, MAX_SEARCH_DEPTH};
use crate::strategy::{
    Control, Reporter, SearchInfo, SearchRestrictions, SearchScore, SearchStats,
};
use crate::util;
use chess::game::Game;
use chess::moves::Move;

pub fn search(
    game: &mut EngineGame,
    tt: &mut SearchTranspositionTable,
    search_restrictions: &SearchRestrictions,
    _options: &EngineOptions,
    state: &mut SearchState,
    time_control: &TimeStrategy,
    control: &impl Control,
    reporter: &impl Reporter,
) -> (Move, Eval) {
    let mut overall_best_move: Option<Move> = None;
    let mut overall_eval: Option<Eval> = None;

    let max_search_depth = search_restrictions.depth.unwrap_or(MAX_SEARCH_DEPTH);
    state.max_depth_reached = 0;

    for depth in 1..=max_search_depth {
        // TODO: Are we counting nodes searched at this depth?
        state.nodes_visited = 0;

        let Ok(eval) = negamax::negamax(
            game,
            Eval::MIN,
            Eval::MAX,
            depth,
            0,
            tt,
            time_control,
            state,
            control,
        ) else {
            // TODO: Send results, even if the search is cancelled, since they may still be better
            // than whatever we found at the previous depth even if we didn't finish the search.
            break;
        };

        let score = if let Some(nmoves) = eval.is_mate_in_moves() {
            SearchScore::Mate(nmoves)
        } else {
            SearchScore::Centipawns(eval.0)
        };

        let pv = get_pv(depth, game.game.clone(), tt);

        let best_move = pv.first().unwrap();

        overall_best_move = Some(*best_move);
        overall_eval = Some(eval);

        reporter.report_search_progress(SearchInfo {
            depth,
            seldepth: state.max_depth_reached,
            score,
            pv: pv.clone(),
            hashfull: tt.occupancy(),
            stats: SearchStats {
                time: time_control.elapsed(),
                nodes: state.nodes_visited,
                nodes_per_second: util::metrics::nodes_per_second(
                    state.nodes_visited,
                    time_control.elapsed(),
                ),
            },
        });
    }

    (overall_best_move.unwrap(), overall_eval.unwrap())
}

fn get_pv(depth: u8, game: Game, tt: &SearchTranspositionTable) -> Vec<Move> {
    let mut current_position = game;
    let mut pv = Vec::new();

    for _ in 0..depth {
        let Some(tt_entry) = tt.get(&current_position.zobrist) else {
            break;
        };

        if tt_entry.bound != NodeBound::Exact {
            // TODO: Log this to a file
            assert!(!pv.is_empty());
            return pv;
        }

        let best_move_in_position = tt_entry.best_move.as_ref().unwrap().to_move();
        pv.push(best_move_in_position);
        current_position.make_move(&best_move_in_position);
    }

    pv
}
