use std::cmp::{max, min};
use std::collections::HashSet;

use crate::client::action_queue::ActionQueue;

#[derive(Clone, Copy, Debug)]
pub enum Action {
    Single {
        index: usize,
        action_type: SingleCellAction,
    },
    Pair {
        index1: usize,
        index2: usize,
        action_type: CellPairAction,
    },
}

#[derive(Clone, Copy, Debug)]
pub enum SingleCellAction {
    MarkSurrEmpty { surr: usize },
    MarkSurrMine { surr: usize },
    ClientClear { mines: usize },
    ServerClear,
    Flag,
}

#[derive(Clone, Copy, Debug)]
pub enum CellPairAction {
    CompareSurr,
}

#[derive(Clone, Debug)]
pub enum Cell {
    Ongoing(OngoingCell),
    Complete,
}
#[derive(Clone, Debug)]
pub struct OngoingCell {
    index: usize,
    unknown_surr: HashSet<usize>,
    total_surr: HashSet<usize>,
    total_surr_mines: Option<usize>,
    known_surr_mines: usize,
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

    // Whether this cell has been cleared/flagged. Doesn't check for
    // to-be-cleared cells.
    pub fn is_marked(&self) -> bool {
        match self {
            &Cell::Ongoing(OngoingCell {
                total_surr_mines: None,
                ..
            }) => false,
            _ => true,
        }
    }
}

impl OngoingCell {
    fn unknown_surr_mines(&self) -> Option<usize> {
        Some(self.total_surr_mines? - self.known_surr_mines)
    }

    // Returns true if cell is Complete as a result of action.
    pub fn apply_action(&mut self, actions: &mut ActionQueue, action: SingleCellAction) -> bool {
        use self::SingleCellAction::*;

        match action {
            MarkSurrEmpty { surr } => {
                self.mark_surr_empty(surr);
                self.try_complete(actions)
            }
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
                self.try_complete(actions)
            }
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
        action: CellPairAction,
    ) {
        use self::CellPairAction::*;

        match action {
            CompareSurr => {
                self.compare_surr(other, actions);
            }
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
                action_type: SingleCellAction::MarkSurrEmpty { surr: self.index },
            })
        }
    }

    fn server_clear(&mut self, actions: &mut ActionQueue) {
        if self.total_surr_mines == None {
            actions.add_to_clear(self.index);
        }

        // Not necessary to mark as empty at this point, but allows for
        // some more eager clearing/flagging on the current turn
        for &surr in self.total_surr.iter() {
            actions.push(Action::Single {
                index: surr,
                action_type: SingleCellAction::MarkSurrEmpty { surr: self.index },
            })
        }
    }

    fn flag(&mut self, actions: &mut ActionQueue) {
        actions.add_to_flag(self.index);

        for &surr in self.total_surr.iter() {
            actions.push(Action::Single {
                index: surr,
                action_type: SingleCellAction::MarkSurrMine { surr: self.index },
            })
        }
    }

    fn try_complete(&mut self, actions: &mut ActionQueue) -> bool {
        if let Some(unknown_surr_mines) = self.unknown_surr_mines() {
            if try_mark_cell_set(unknown_surr_mines, self.unknown_surr.iter(), actions) {
                return true;
            }
        }

        for &surr in self.total_surr.iter() {
            actions.push(Action::Pair {
                index1: self.index,
                index2: surr,
                action_type: CellPairAction::CompareSurr,
            });
        }

        return false;
    }

    fn compare_surr(&mut self, other: &mut OngoingCell, actions: &mut ActionQueue) {
        if let (Some(self_unknown_mines), Some(other_unknown_mines)) =
            (self.unknown_surr_mines(), other.unknown_surr_mines())
        {
            let self_excl = || self.unknown_surr.difference(&other.unknown_surr);
            let other_excl = || other.unknown_surr.difference(&self.unknown_surr);
            let common = || self.unknown_surr.intersection(&other.unknown_surr);

            if let Some((self_count, mid_count, other_count)) = solve_linear_constraints(
                (self_excl().count(), common().count(), other_excl().count()),
                (self_unknown_mines, other_unknown_mines),
            ) {
                try_mark_cell_set(self_count, self_excl(), actions);
                try_mark_cell_set(mid_count, common(), actions);
                try_mark_cell_set(other_count, other_excl(), actions);
            }
        }
    }
}

fn try_mark_cell_set<'a, I: Iterator<Item = &'a usize> + Clone>(
    mine_count: usize,
    set_iter: I,
    actions: &mut ActionQueue,
) -> bool {
    let action_type = match mine_count {
        0 => SingleCellAction::ServerClear,
        c if c == set_iter.clone().count() => SingleCellAction::Flag,
        _ => {
            return false;
        }
    };

    for &index in set_iter {
        actions.push(Action::Single { index, action_type });
    }

    return true;
}

fn solve_linear_constraints(
    (x_max, y_max, z_max): (usize, usize, usize),
    (x_add_y, y_add_z): (usize, usize),
) -> Option<(usize, usize, usize)> {
    let x_max = min(x_max, x_add_y);
    let y_max = min(min(y_max, x_add_y), y_add_z);
    let z_max = min(z_max, y_add_z);

    let x_min = max(0, x_add_y - y_max);
    let y_min = max(max(0, x_add_y - x_max), y_add_z - z_max);
    let z_min = max(0, y_add_z - y_max);

    match (x_max - x_min, y_max - y_min, z_max - z_min) {
        (0, _, _) => {
            let x = x_max;
            let y = x_add_y - x;
            let z = y_add_z - y;
            Some((x, y, z))
        }
        (_, 0, _) => {
            let y = y_max;
            let x = x_add_y - y;
            let z = y_add_z - y;
            Some((x, y, z))
        }
        (_, _, 0) => {
            let z = z_max;
            let y = y_add_z - z;
            let x = x_add_y - y;
            Some((x, y, z))
        }
        _ => None,
    }
}
