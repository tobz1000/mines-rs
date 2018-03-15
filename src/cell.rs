pub enum ClientAction {
    IncSurrEmpty,
    IncSurrMine,
    Clear(usize),
    Flag,
}

#[derive(Clone, Copy)]
pub enum ServerAction { Clear, Flag }

#[derive(Clone, Copy)]
pub enum Cell {
    Ongoing {
        total_surr: usize,
        total_surr_mines: Option<usize>,
        known_surr_empty: usize,
        known_surr_mines: usize,
    },
    Clear,
    Mine
}

impl Cell {
    pub fn new(total_surr: usize) -> Self {
        Cell::Ongoing {
            total_surr,
            total_surr_mines: None,
            known_surr_empty: 0,
            known_surr_mines: 0
        }

    }

    fn try_mark_surrounding(&mut self) -> Option<ServerAction> {
        match self {
            &mut Cell::Ongoing {
                total_surr,
                total_surr_mines: Some(total_surr_mines),
                known_surr_empty,
                known_surr_mines
            } => {
                if known_surr_mines == total_surr_mines {
                    *self = Cell::Clear;
                    Some(ServerAction::Clear)
                } else if known_surr_empty == total_surr - total_surr_mines {
                    *self = Cell::Clear;
                    Some(ServerAction::Flag)
                } else { None }
            },
            _ => None
        }
    }

    pub fn set_mine(&mut self) {
        match *self {
            Cell::Ongoing { .. } => { *self = Cell::Mine; },
            Cell::Mine => (),
            _ => panic!("Tried to flag previously-cleared cell")
        }
    }

    pub fn set_clear(&mut self, surr_mine_count: usize) -> Option<ServerAction> {
        match *self {
            Cell::Ongoing {
                total_surr_mines: ref mut total_surr_mines @ None,
                ..
            } => {
                *total_surr_mines = Some(surr_mine_count);
            },
            Cell::Ongoing { .. } => (),
            Cell::Clear => (),
            Cell::Mine => { panic!("Tried to clear previously-flagged cell"); }
        }

        self.try_mark_surrounding()
    }

    pub fn inc_surr_empty(&mut self) -> Option<ServerAction> {
        if let &mut Cell::Ongoing { ref mut known_surr_empty, .. } = self {
            *known_surr_empty += 1;
        }

        self.try_mark_surrounding()
    }

    pub fn inc_surr_mine(&mut self) -> Option<ServerAction> {
        if let &mut Cell::Ongoing { ref mut known_surr_mines, .. } = self {
            *known_surr_mines += 1;
        }

        self.try_mark_surrounding()
    }
}