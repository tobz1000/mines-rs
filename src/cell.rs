use std::collections::HashSet;
use action_queue::ActionQueue;


#[derive(Debug)]
pub struct CellAction {
    pub index: usize,
    pub action_type: CellActionType
}

#[derive(Clone, Copy, Debug)]
pub enum CellActionType {
    MarkSurrEmpty { surr: usize },
    MarkSurrMine { surr: usize },
    TryComplete,
    ClientClear { mines: usize },
    ServerClear,
    Flag,
}

#[derive(Clone, Debug)]
pub struct OngoingCell {
    index: usize,
    unknown_surr: HashSet<usize>,
    total_surr: HashSet<usize>,
    total_surr_mines: Option<usize>,
    known_surr_mines: usize,
    known_surr_empty: usize,
}

#[derive(Clone, Debug)]
pub enum Cell {
    Ongoing(OngoingCell),
    Complete,
}

impl Cell {
    pub fn apply_action(
        &mut self,
        actions: &mut ActionQueue,
        action: CellActionType,
    ) {
        let mut complete = false;

        if let &mut Cell::Ongoing(ref mut ongoing) = self {
            complete = ongoing.apply_action(actions, action);
        }

        if complete {
            *self = Cell::Complete;
        }
    }

    pub fn new(index: usize, surr_indices: HashSet<usize>) -> Self {
        Cell::Ongoing(OngoingCell {
            index,
            unknown_surr: surr_indices.clone(),
            total_surr: surr_indices,
            total_surr_mines: None,
            known_surr_mines: 0,
            known_surr_empty: 0,
        })
    }
}

impl OngoingCell {
    // Returns true if cell is Complete as a result of action.
    fn apply_action(
        &mut self,
        actions: &mut ActionQueue,
        action: CellActionType
    ) -> bool {
        use self::CellActionType::*;

        match action {
            MarkSurrEmpty { surr } => { self.mark_surr_empty(actions, surr); },
            MarkSurrMine { surr } => { self.mark_surr_mine(actions, surr); },
            TryComplete => { return self.try_complete(actions); },
            ClientClear { mines } => { self.client_clear(actions, mines); },
            ServerClear => { self.server_clear(actions); },
            Flag => {
                self.flag(actions);
                return true;
            }
        }

        false
    }

    fn mark_surr_empty(&mut self, actions: &mut ActionQueue, surr: usize) {
        self.unknown_surr.remove(&surr);
        self.known_surr_empty += 1;

        actions.push(CellAction {
            index: self.index,
            action_type: CellActionType::TryComplete
        });
    }

    fn mark_surr_mine(&mut self, actions: &mut ActionQueue, surr: usize) {
        self.unknown_surr.remove(&surr);
        self.known_surr_mines += 1;

        actions.push(CellAction {
            index: self.index,
            action_type: CellActionType::TryComplete
        });
    }

    fn try_complete(&mut self, actions: &mut ActionQueue) -> bool {
        if let &mut OngoingCell {
            total_surr_mines: Some(total_surr_mines),
            known_surr_mines,
            ref unknown_surr,
            ..
        } = self {
            let unknown_mines = total_surr_mines - known_surr_mines;

            if let Some(action_type) = if unknown_mines == 0 {
                Some(CellActionType::ServerClear)
            } else if unknown_mines == unknown_surr.len() {
                Some(CellActionType::Flag)
            } else { None } {
                for &surr in unknown_surr.iter() {
                    actions.push(CellAction { index: surr, action_type });
                }

                return true;
            }
        }

        false
    }

    fn client_clear(&mut self, actions: &mut ActionQueue, mines: usize) {
        self.total_surr_mines = Some(mines);

        for &surr in self.total_surr.iter() {
            actions.push(CellAction {
                index: surr,
                action_type: CellActionType::MarkSurrEmpty {
                    surr: self.index
                }
            })
        }

        actions.push(CellAction {
            index: self.index,
            action_type: CellActionType::TryComplete
        });
    }

    fn server_clear(&mut self, actions: &mut ActionQueue) {
        if self.total_surr_mines == None {
            actions.add_to_clear(self.index);
        }
    }

    fn flag(&mut self, actions: &mut ActionQueue) {
        actions.add_to_flag(self.index);

        for &surr in self.total_surr.iter() {
            actions.push(CellAction {
                index: surr,
                action_type: CellActionType::MarkSurrMine {
                    surr: self.index
                }
            })
        }
    }
}