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
        action: CellActionType,
        actions: &mut ActionQueue,
    ) {
        let mut complete = false;

        if let &mut Cell::Ongoing(ref mut ongoing) = self {
            match action {
                CellActionType::MarkSurrEmpty { surr } => {
                    ongoing.unknown_surr.remove(&surr);
                    ongoing.known_surr_empty += 1;

                    actions.push(CellAction {
                        index: ongoing.index,
                        action_type: CellActionType::TryComplete
                    });
                },
                CellActionType::MarkSurrMine { surr } => {
                    ongoing.unknown_surr.remove(&surr);
                    ongoing.known_surr_mines += 1;

                    actions.push(CellAction {
                        index: ongoing.index,
                        action_type: CellActionType::TryComplete
                    });
                },
                CellActionType::TryComplete => {
                    if let &mut OngoingCell {
                        total_surr_mines: Some(total_surr_mines),
                        known_surr_mines,
                        ref unknown_surr,
                        ..
                    } = ongoing {
                        let unknown_mines = total_surr_mines - known_surr_mines;

                        if unknown_mines == 0 {
                            for &surr in ongoing.unknown_surr.iter() {
                                actions.push(CellAction {
                                    index: surr,
                                    action_type: CellActionType::ServerClear
                                });
                            }

                            complete = true;
                        } else if unknown_mines == unknown_surr.len() {
                            for &surr in ongoing.unknown_surr.iter() {
                                actions.push(CellAction {
                                    index: surr,
                                    action_type: CellActionType::Flag
                                });
                            }

                            complete = true;
                        }
                    }
                },
                CellActionType::ClientClear { mines } => {
                    ongoing.total_surr_mines = Some(mines);

                    for &surr in ongoing.total_surr.iter() {
                        actions.push(CellAction {
                            index: surr,
                            action_type: CellActionType::MarkSurrEmpty {
                                surr: ongoing.index
                            }
                        })
                    }

                    actions.push(CellAction {
                        index: ongoing.index,
                        action_type: CellActionType::TryComplete
                    });
                },
                CellActionType::ServerClear => {
                    if ongoing.total_surr_mines == None {
                        actions.add_to_clear(ongoing.index);
                    }
                },
                CellActionType::Flag => {
                    actions.add_to_flag(ongoing.index);

                    for &surr in ongoing.total_surr.iter() {
                        actions.push(CellAction {
                            index: surr,
                            action_type: CellActionType::MarkSurrMine {
                                surr: ongoing.index
                            }
                        })
                    }

                    complete = true;
                }
            }
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