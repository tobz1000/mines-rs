use std::collections::{VecDeque, HashSet};
use cell::CellAction;

pub struct ActionQueue {
    actions: VecDeque<CellAction>,
    server_to_clear: HashSet<usize>,
    server_to_flag: HashSet<usize>
}

impl ActionQueue {
    pub fn new() -> Self {
        ActionQueue {
            actions: VecDeque::new(),
            server_to_clear: HashSet::new(),
            server_to_flag: HashSet::new(),
        }
    }

    pub fn push(&mut self, action: CellAction) {
        self.actions.push_back(action);
    }

    pub fn pull(&mut self) -> Option<CellAction> {
        self.actions.pop_front()
    }

    pub fn add_to_clear(&mut self, index: usize) {
        self.server_to_clear.insert(index);
    }

    pub fn add_to_flag(&mut self, index: usize) {
        self.server_to_flag.insert(index);
    }

    pub fn get_to_clear(&self) -> impl Iterator<Item=&usize> {
        self.server_to_clear.iter()
    }

    pub fn get_to_flag(&self) -> impl Iterator<Item=&usize> {
        self.server_to_flag.iter()
    }
}