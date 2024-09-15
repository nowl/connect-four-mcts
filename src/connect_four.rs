use yamcts::GameState;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Position {
    Red,
    Black,
    Empty,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CFMove {
    pub color: Position, // Red or Black
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct CFGameState {
    // 7 columns, 6 rows
    board: [Position; 7 * 6],
    prev_player: Position, // Red or Black
    next_player: Position, // Red or Black
}

impl CFGameState {
    pub fn new(prev_player: Position, next_player: Position) -> Self {
        Self {
            board: [Position::Empty; 7 * 6],
            prev_player,
            next_player,
        }
    }

    // col=0, row=0 is top-left
    pub fn pos(&self, col: usize, row: usize) -> Position {
        debug_assert!(col <= 7 && row <= 6);
        self.board[row * 7 + col]
    }

    fn same_vals(&self, pos: [(usize, usize); 4], val: Position) -> bool {
        pos.iter().all(|&(col, row)| self.pos(col, row) == val)
    }
}

impl GameState for CFGameState {
    type Move = CFMove;
    type UserData = Position;

    fn all_moves(&self) -> Vec<Self::Move> {
        (0..7)
            .filter_map(|i| {
                if self.board[i] == Position::Empty {
                    Some(CFMove {
                        color: self.next_player,
                        col: i,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    fn is_terminal_state(&self) -> Option<Self::UserData> {
        use Position::*;

        macro_rules! build_same {
            ($col:ident, $row:ident, $c1:expr, $r1:expr,$c2:expr, $r2:expr,$c3:expr, $r3:expr,$c4:expr, $r4:expr) => {
                if self.same_vals(
                    [
                        ($col + $c1, $row + $r1),
                        ($col + $c2, $row + $r2),
                        ($col + $c3, $row + $r3),
                        ($col + $c4, $row + $r4),
                    ],
                    Black,
                ) {
                    return Some(Black);
                }

                if self.same_vals(
                    [
                        ($col + $c1, $row + $r1),
                        ($col + $c2, $row + $r2),
                        ($col + $c3, $row + $r3),
                        ($col + $c4, $row + $r4),
                    ],
                    Red,
                ) {
                    return Some(Red);
                }
            };
        }

        macro_rules! direct_same {
            ($c1:expr, $r1:expr,$c2:expr, $r2:expr,$c3:expr, $r3:expr,$c4:expr, $r4:expr) => {
                if self.same_vals([($c1, $r1), ($c2, $r2), ($c3, $r3), ($c4, $r4)], Black) {
                    return Some(Black);
                }
                if self.same_vals([($c1, $r1), ($c2, $r2), ($c3, $r3), ($c4, $r4)], Red) {
                    return Some(Red);
                }
            };
        }

        // horizontal wins
        for row in 0..6 {
            for col in 0..4 {
                build_same!(col, row, 0, 0, 1, 0, 2, 0, 3, 0);
            }
        }

        // vertical wins
        for col in 0..7 {
            for row in 0..3 {
                build_same!(col, row, 0, 0, 0, 1, 0, 2, 0, 3);
            }
        }

        // diagonal wins
        direct_same!(0, 2, 1, 3, 2, 4, 3, 5);
        direct_same!(0, 1, 1, 2, 2, 3, 3, 4);
        direct_same!(1, 2, 2, 3, 3, 4, 4, 5);
        direct_same!(0, 0, 1, 1, 2, 2, 3, 3);
        direct_same!(1, 1, 2, 2, 3, 3, 4, 4);
        direct_same!(2, 2, 3, 3, 4, 4, 5, 5);
        direct_same!(1, 0, 2, 1, 3, 2, 4, 3);
        direct_same!(2, 1, 3, 2, 4, 3, 5, 4);
        direct_same!(3, 2, 4, 3, 5, 4, 6, 5);
        direct_same!(2, 0, 3, 1, 4, 2, 5, 3);
        direct_same!(3, 1, 4, 2, 5, 3, 6, 4);
        direct_same!(3, 0, 4, 1, 5, 2, 6, 3);

        direct_same!(6 - 0, 2, 6 - 1, 3, 6 - 2, 4, 6 - 3, 5);
        direct_same!(6 - 0, 1, 6 - 1, 2, 6 - 2, 3, 6 - 3, 4);
        direct_same!(6 - 1, 2, 6 - 2, 3, 6 - 3, 4, 6 - 4, 5);
        direct_same!(6 - 0, 0, 6 - 1, 1, 6 - 2, 2, 6 - 3, 3);
        direct_same!(6 - 1, 1, 6 - 2, 2, 6 - 3, 3, 6 - 4, 4);
        direct_same!(6 - 2, 2, 6 - 3, 3, 6 - 4, 4, 6 - 5, 5);
        direct_same!(6 - 1, 0, 6 - 2, 1, 6 - 3, 2, 6 - 4, 3);
        direct_same!(6 - 2, 1, 6 - 3, 2, 6 - 4, 3, 6 - 5, 4);
        direct_same!(6 - 3, 2, 6 - 4, 3, 6 - 5, 4, 6 - 6, 5);
        direct_same!(6 - 2, 0, 6 - 3, 1, 6 - 4, 2, 6 - 5, 3);
        direct_same!(6 - 3, 1, 6 - 4, 2, 6 - 5, 3, 6 - 6, 4);
        direct_same!(6 - 3, 0, 6 - 4, 1, 6 - 5, 2, 6 - 6, 3);

        // tie
        if (0..7).all(|col| self.pos(col, 0) != Empty) {
            return Some(Empty);
        }

        None
    }

    fn apply_move(&self, action: Self::Move) -> Self {
        use Position::*;

        debug_assert!(self.pos(action.col, 0) == Empty);
        let mut new_state = self.clone();

        // find row
        let mut row = 5;

        for r in 0..6 {
            if self.pos(action.col, r) != Empty {
                row = r - 1;
                break;
            }
        }

        new_state.board[row * 7 + action.col] = action.color;
        new_state.prev_player = action.color;
        new_state.next_player = match action.color {
            Red => Black,
            Black => Red,
            Empty => unreachable!(),
        };
        new_state
    }

    fn terminal_is_win(&self, condition: &Self::UserData) -> bool {
        self.prev_player == *condition
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn win_conditions() {
        use Position::*;
        let mut state = CFGameState::new(Red, Black);

        macro_rules! set_pos {
            ($c1:expr, $r1:expr,$c2:expr, $r2:expr,$c3:expr, $r3:expr,$c4:expr, $r4:expr, $v:expr) => {
                state.board[$r1 * 7 + $c1] = $v;
                state.board[$r2 * 7 + $c2] = $v;
                state.board[$r3 * 7 + $c3] = $v;
                state.board[$r4 * 7 + $c4] = $v;
            };
        }

        macro_rules! clear_pos {
            () => {
                for i in 0..7 * 6 {
                    state.board[i] = Empty;
                }
            };
        }

        assert_eq!(state.is_terminal_state(), None);

        set_pos!(1, 5, 2, 5, 3, 5, 4, 5, Black);
        assert_eq!(state.is_terminal_state(), Some(Black));

        clear_pos!();
        assert_eq!(state.is_terminal_state(), None);

        set_pos!(2, 5, 3, 5, 4, 5, 5, 5, Red);
        assert_eq!(state.is_terminal_state(), Some(Red));
        clear_pos!();

        set_pos!(1, 0, 1, 1, 1, 2, 1, 3, Red);
        assert_eq!(state.is_terminal_state(), Some(Red));
        clear_pos!();

        set_pos!(2, 2, 3, 3, 4, 4, 5, 5, Black);
        assert_eq!(state.is_terminal_state(), Some(Black));
        clear_pos!();

        set_pos!(2, 2, 3, 3, 4, 4, 5, 5, Black);
        set_pos!(5, 5, 5, 4, 5, 3, 5, 2, Red);
        assert_eq!(state.is_terminal_state(), Some(Red));
        clear_pos!();

        set_pos!(0, 3, 1, 2, 2, 1, 3, 0, Red);
        assert_eq!(state.is_terminal_state(), Some(Red));
        clear_pos!();

        set_pos!(2, 4, 3, 3, 4, 2, 5, 1, Red);
        assert_eq!(state.is_terminal_state(), Some(Red));
        clear_pos!();
    }
}
