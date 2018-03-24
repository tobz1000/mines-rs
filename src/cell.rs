use std::collections::HashSet;
use std::cmp::{max, min};
use action_queue::ActionQueue;


pub enum Action {
    Single { index: usize, action_type: SingleCellAction },
    Pair { index1: usize, index2: usize, action_type: CellPairAction }
}

#[derive(Clone, Copy, Debug)]
pub enum SingleCellAction {
    MarkSurrEmpty { surr: usize },
    MarkSurrMine { surr: usize },
    ClientClear { mines: usize },
    ServerClear,
    Flag,
}

pub enum CellPairAction {
    CompareSurr
}

#[derive(Clone, Debug)]
pub enum Cell {
    Ongoing(OngoingCell),
    Complete,
}

impl Cell {
    pub fn new(index: usize, surr_indices: HashSet<usize>) -> Self {
        Cell::Ongoing(OngoingCell {
            index,
            unknown_surr: surr_indices.clone(),
            total_surr: surr_indices,
            total_surr_mines: None,
            known_surr_mines: 0,
        })
    }
}

#[derive(Clone, Debug)]
pub struct OngoingCell {
    index: usize,
    unknown_surr: HashSet<usize>,
    total_surr: HashSet<usize>,
    total_surr_mines: Option<usize>,
    known_surr_mines: usize,
}

impl OngoingCell {
    fn total_surr_empty(&self) -> Option<usize> {
        Some(self.total_surr.len() - self.total_surr_mines?)
    }

    fn known_surr(&self) -> usize {
        self.total_surr.len() - self.unknown_surr.len()
    }

    fn known_surr_empty(&self) -> usize {
        self.known_surr() - self.known_surr_mines
    }

    fn unknown_surr_mines(&self) -> Option<usize> {
        Some(self.total_surr_mines? - self.known_surr_mines)
    }

    fn unknown_surr_empty(&self) -> Option<usize> {
        Some(self.total_surr_empty()? - self.known_surr_empty())
    }

    // Returns true if cell is Complete as a result of action.
    pub fn apply_action(
        &mut self,
        actions: &mut ActionQueue,
        action: SingleCellAction
    ) -> bool {
        use self::SingleCellAction::*;

        match action {
            MarkSurrEmpty { surr } => {
                self.mark_surr_empty(surr);
                self.try_complete(actions)
            },
            MarkSurrMine { surr } => {
                self.mark_surr_mine(surr);
                self.try_complete(actions)
            }
            ClientClear { mines } => {
                self.client_clear(actions, mines);
                self.try_complete(actions)
            }
            ServerClear => {
                self.server_clear(actions);
                false
            },
            Flag => {
                self.flag(actions);
                true
            }
        }
    }

    pub fn apply_pair_action(
        &mut self,
        other: &mut OngoingCell,
        actions: &mut ActionQueue,
        action: CellPairAction
    ) {
        use self::CellPairAction::*;

        match action {
            CompareSurr => { self.compare_surr(other, actions); }
        }
    }

    fn mark_surr_empty(&mut self, surr: usize) {
        self.unknown_surr.remove(&surr);
    }

    fn mark_surr_mine(&mut self, surr: usize) {
        self.unknown_surr.remove(&surr);
        self.known_surr_mines += 1;
    }

    fn client_clear(&mut self, actions: &mut ActionQueue, mines: usize) {
        self.total_surr_mines = Some(mines);

        for &surr in self.total_surr.iter() {
            actions.push(Action::Single {
                index: surr,
                action_type: SingleCellAction::MarkSurrEmpty {
                    surr: self.index
                }
            })
        }
    }

    fn server_clear(&mut self, actions: &mut ActionQueue) {
        if self.total_surr_mines == None {
            actions.add_to_clear(self.index);
        }
    }

    fn flag(&mut self, actions: &mut ActionQueue) {
        actions.add_to_flag(self.index);

        for &surr in self.total_surr.iter() {
            actions.push(Action::Single {
                index: surr,
                action_type: SingleCellAction::MarkSurrMine {
                    surr: self.index
                }
            })
        }
    }

    fn try_complete(&mut self, actions: &mut ActionQueue) -> bool {
        if let Some(unknown_mines) = self.unknown_surr_mines() {
            let single_action = if unknown_mines == 0 {
                Some(SingleCellAction::ServerClear)
            } else if unknown_mines == self.unknown_surr.len() {
                Some(SingleCellAction::Flag)
            } else { None };

            if let Some(action_type) = single_action {
                for &surr in self.unknown_surr.iter() {
                    actions.push(Action::Single { index: surr, action_type });
                }

                return true;
            } else {
                for &surr in self.total_surr.iter() {
                    actions.push(Action::Pair {
                        index1: self.index,
                        index2: surr,
                        action_type: CellPairAction::CompareSurr
                    });
                }
            }
        }

        false
    }

    fn compare_surr(
        &mut self,
        other: &mut OngoingCell,
        actions: &mut ActionQueue
    ) {
        if let (
            Some(self_unknown_mines),
            Some(other_unknown_mines)
        ) = (self.unknown_surr_mines(), other.unknown_surr_mines()) {
            let self_excl: Vec<usize> = self.unknown_surr
                .difference(&other.unknown_surr)
                .map(|&i| i)
                .collect();
            let other_excl: Vec<usize> = other.unknown_surr
                .difference(&self.unknown_surr)
                .map(|&i| i)
                .collect();
            let common: Vec<usize> = self.unknown_surr
                .intersection(&other.unknown_surr)
                .map(|&i| i)
                .collect();

            let (self_count, mid_count, other_count) = solve_linear_constraints(
                (self_excl.len(), common.len(), other_excl.len()),
                (self_unknown_mines, other_unknown_mines)
            );

            for &(count, ref set) in [
                (self_count, self_excl),
                (mid_count, common),
                (other_count, other_excl)
            ].into_iter() {
                let action_type = match count {
                    Some(0) => SingleCellAction::ServerClear,
                    Some(c) if c == set.len() => SingleCellAction::Flag,
                    _ => { continue; }
                };

                for &index in set.iter() {
                    actions.push(Action::Single { index, action_type });
                }
            }
        }
    }
}

fn solve_linear_constraints(
    (x_max, y_max, z_max): (usize, usize, usize),
    (x_add_y, y_add_z): (usize, usize)
) -> (Option<usize>, Option<usize>, Option<usize>) {
    let x_max = min(x_max, x_add_y);
    let y_max = min(min(y_max, x_add_y), y_add_z);
    let z_max = min(z_max, y_add_z);

    let x_min = max(0, x_add_y - y_max);
    let y_min = max(max(0, x_add_y - x_max), y_add_z - z_max);
    let z_min = max(0, y_add_z - y_max);

    (
        if x_max == x_min { Some(x_max) } else { None },
        if y_max == y_min { Some(y_max) } else { None },
        if z_max == z_min { Some(z_max) } else { None },
    )
}