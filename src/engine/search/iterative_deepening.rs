use crate::chess::game::Game;
use crate::chess::moves::Move;
use crate::engine::eval::Eval;
use crate::engine::options::EngineOptions;
use crate::engine::search::aspiration::aspiration_search;
use crate::engine::search::principal_variation::PrincipalVariation;
use crate::engine::search::time_control::TimeStrategy;
use crate::engine::search::{
    PersistentState, Reporter, SearchInfo, SearchRestrictions, SearchScore, SearchState,
    SearchStats, MAX_SEARCH_DEPTH,
};
use crate::engine::util;

pub fn search(
    game: &mut Game,
    persistent_state: &mut PersistentState,
    search_restrictions: &SearchRestrictions,
    _options: &EngineOptions,
    state: &mut SearchState,
    pv: &mut PrincipalVariation,
    time_control: &mut TimeStrategy,
    reporter: &mut impl Reporter,
) -> Option<Move> {
    let mut best_move: Option<Move> = None;
    let mut overall_eval: Option<Eval> = None;

    let max_search_depth = search_restrictions.depth.unwrap_or(MAX_SEARCH_DEPTH);
    state.max_depth_reached = 0;

    for depth in 1..=max_search_depth {
        if !time_control.should_start_new_search(depth) {
            break;
        }

        let eval = aspiration_search(
            game,
            depth,
            overall_eval,
            persistent_state,
            pv,
            state,
            time_control,
        );

        if eval == Eval::ABORTED {
            break;
        }

        let score = if let Some(nmoves) = eval.is_mate_in_moves() {
            SearchScore::Mate(nmoves)
        } else {
            SearchScore::Centipawns(eval.0)
        };

        best_move = Some(*pv.first().unwrap());
        overall_eval = Some(eval);

        reporter.report_search_progress(
            game,
            SearchInfo {
                depth,
                seldepth: state.max_depth_reached,
                score,
                pv: pv.clone(),
                hashfull: persistent_state.tt.occupancy(),
                stats: SearchStats {
                    time: time_control.elapsed(),
                    nodes: state.nodes_visited,
                    nodes_per_second: util::metrics::nodes_per_second(
                        state.nodes_visited,
                        time_control.elapsed(),
                    ),
                },
            },
        );
    }

    best_move
}
