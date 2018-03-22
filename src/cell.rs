use std::collections::{HashSet, VecDeque};


#[derive(Debug)]
pub struct CellAction {
    pub index: usize,
    pub action_type: CellActionType
}

#[derive(Clone, Copy, Debug)]
pub enum CellActionType {
    MarkSurrEmpty { surr: usize },
    MarkSurrMine { surr: usize },
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
        actions: &mut VecDeque<CellAction>,
        server_to_clear: &mut HashSet<usize>,
        server_to_flag: &mut HashSet<usize>,
    ) {
        let mut complete = false;

        if let &mut Cell::Ongoing(ref mut ongoing) = self {
            if let Some(action_type) = match action {
                CellActionType::MarkSurrEmpty { surr } => {
                    ongoing.unknown_surr.remove(&surr);
                    ongoing.known_surr_empty += 1;

                    ongoing.try_complete()
                },
                CellActionType::MarkSurrMine { surr } => {
                    ongoing.unknown_surr.remove(&surr);
                    ongoing.known_surr_mines += 1;

                    ongoing.try_complete()
                },
                CellActionType::ClientClear { mines } => {
                    ongoing.total_surr_mines = Some(mines);

                    for &surr in ongoing.total_surr.iter() {
                        actions.push_back(CellAction {
                            index: surr,
                            action_type: CellActionType::MarkSurrEmpty {
                                surr: ongoing.index
                            }
                        })
                    }

                    ongoing.try_complete()
                },
                CellActionType::ServerClear => {
                    if ongoing.total_surr_mines == None {
                        server_to_clear.insert(ongoing.index);
                    }

                    None
                },
                CellActionType::Flag => {
                    server_to_flag.insert(ongoing.index);

                    for &surr in ongoing.total_surr.iter() {
                        actions.push_back(CellAction {
                            index: surr,
                            action_type: CellActionType::MarkSurrMine {
                                surr: ongoing.index
                            }
                        })
                    }

                    complete = true;

                    None
                }
            } {
                for &surr in ongoing.unknown_surr.iter() {
                    actions.push_back(CellAction { index: surr, action_type });
                }

                complete = true;
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

impl OngoingCell {
    fn try_complete(&self) -> Option<CellActionType> {
        if let Some(total_surr_mines) = self.total_surr_mines {
            let unknown_mines = total_surr_mines - self.known_surr_mines;

            if unknown_mines == 0 {
                Some(CellActionType::ServerClear)
            } else if unknown_mines == self.unknown_surr.len() {
                Some(CellActionType::Flag)
            } else { None }
        } else { None }
    }
}