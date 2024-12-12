#[warn(unused_variables)]

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Cell {
    Empty,
    Attacker,
    Defender,
    King,
    Corner
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub board: Vec<Vec<Cell>>, // 2D grid representing the board
    pub current_turn: Cell,    // Attacker or Defender
    pub game_over: bool,       // Indicates if the game has ended
    pub winner: Option<Cell>,  // Stores the winner (None if ongoing)
    pub click_count: u32,      // Number of clicks
    pub from: (usize, usize),  // From position
}

impl GameState {
    /// Creates a new game with the initial Hnefatafl board.
    pub fn new() -> Self {
        let mut board = vec![vec![Cell::Empty; 11]; 11];

        // Place attackers (black)
        for &pos in &[
            (0, 3), (0, 4), (0, 5), (0, 6), (0, 7),
            (1, 5),
            (3, 0), (4, 0), (5, 0), (6, 0), (7, 0),
            (5, 1),
            (10, 3), (10, 4), (10, 5), (10, 6), (10, 7),
            (9, 5),
            (3, 10), (4, 10), (5, 10), (6, 10), (7, 10),
            (5, 9),
        ] {
            board[pos.0][pos.1] = Cell::Attacker;
        }

        // Place defenders (white)
        for &pos in &[
            (3, 5),
            (4, 4), (4, 5), (4, 6),            
            (5, 3), (5, 4), (5, 6), (5, 7),
            (6, 4), (6, 5), (6, 6),
            (7, 5),
        ] {
            board[pos.0][pos.1] = Cell::Defender;
        }

        // Place corners
        for &pos in &[
            (0, 0), (0, 10), (10, 0), (10, 10)
        ] {
            board[pos.0][pos.1] = Cell::Corner;
        }

        // Place the king
        board[5][5] = Cell::King;

        GameState {
            board,
            current_turn: Cell::Attacker,
            game_over: false,
            winner: None,
            click_count: 1,
            from: (0, 0),
        }
    }

    pub fn process_click(&mut self, row: usize, col: usize) -> Result<(), String> {
        // Validate and process the click based on the game state
        if row >= self.board.len() || col >= self.board[0].len() {
            return Err("Invalid cell coordinates.".to_string());
        }

        if self.click_count % 2 == 1 {
            // First click
            if self.board[row][col] == Cell::Empty || self.board[row][col] == Cell::Corner {
                return Err("Select a piece to move.".to_string());
            }
            else if self.board[row][col] == Cell::Defender && self.current_turn == Cell::Attacker {
                return Err("Cannot move the defender's piece.".to_string());
            }
            else if self.board[row][col] == Cell::Attacker && self.current_turn == Cell::Defender {
                return Err("Cannot move the attacker's piece.".to_string());
            }            
            else {
                self.click_count += 1;
                self.from = (row, col);
            }
        } else {
            // Second click
            if self.board[row][col] != Cell::Empty && self.board[row][col] != Cell::Corner {
                self.click_count -= 1;
                return Err("Select an empty cell to move to.".to_string());
            }  
            else if self.board[row][col] == Cell::Corner && self.board[self.from.0][self.from.1] != Cell::King {
                self.click_count -= 1;
                return Err("Cannot move to the corner.".to_string());
            }
            else {
                // Make the move              
                match self.make_move(self.from, (row, col)) {
                    Ok(_) => {
                        self.click_count += 1;
                    }
                    Err(error_message) => {
                        return Err(error_message.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> Result<(), String> {
        if self.game_over {
            return Err("Game is already over.".to_string());
        }

        // Validate the move
        if !self.is_valid_move(from, to) {
            return Err("Invalid move.".to_string());
        }

        // Make the move
        self.board[to.0][to.1] = self.board[from.0][from.1].clone();
        self.board[from.0][from.1] = Cell::Empty;

        // Check for captures
        self.check_captures(to)?;

        // Check win conditions
        if let Some(winner) = self.check_win_condition() {
            self.game_over = true;
            self.winner = Some(winner);
        } else {
            // Switch turns
            self.current_turn = if self.current_turn == Cell::Attacker {
                Cell::Defender
            } else {
                Cell::Attacker
            };
        }

        Ok(())
    }

    fn check_captures(&mut self, pos: (usize, usize)) -> Result<(), String> {
        let neighbors = [
            (pos.0 - 1, pos.1), // Up
            (pos.0 + 1, pos.1), // Down
            (pos.0, pos.1 - 1), // Left
            (pos.0, pos.1 + 1), // Right
        ];

        let cell = self.board[pos.0][pos.1];

        for (i, &(nx, ny)) in neighbors.iter().enumerate() {
            if self.is_within_bounds((nx, ny)) {
                let (nnx, nny) = match i {
                    0 => (nx - 1, ny),      // Up (check the cell above the neighbor)
                    1 => (nx + 1, ny),      // Down (check the cell below the neighbor)
                    2 => (nx, ny - 1),      // Left (check the cell to the left of the neighbor)
                    3 => (nx, ny + 1),      // Right (check the cell to the right of the neighbor)
                    _ => unreachable!(),
                    };

                if self.board[nx][ny] == (if cell == Cell::Attacker { Cell::Defender } else { Cell::Attacker }) 
                    && self.board[nnx][nny] == cell 
                {
                    self.board[nx][ny] = Cell::Empty;
                }
            }
            
        }

        Ok(())
    }


    /// Checks if the given position is within board bounds.
    fn is_within_bounds(&self, pos: (usize, usize)) -> bool {
        let size = self.board.len();
        pos.0 < size && pos.1 < size
    }

    /// Checks if the path between two points is clear.
    fn is_path_clear(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let (row, col) = from;
        if row == to.0 {
            // Horizontal move
            let range: Vec<_> = if col < to.1 {
                (col + 1..=to.1).collect()
            } else {
                (to.1..=col - 1).rev().collect()
            };
            range.iter().all(|&c| self.board[row][c] == Cell::Empty)
        } else if col == to.1 {
            // Vertical move
            let range: Vec<_> = if row < to.0 {
                (row + 1..=to.0).collect()
            } else {
                (to.0..=row - 1).rev().collect()
            };
            range.iter().all(|&r| self.board[r][col] == Cell::Empty)
        } else {
            false
        }
    }

    fn is_valid_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if from == to || !self.is_within_bounds(to) {
            return false;
        }

        let piece = &self.board[from.0][from.1];
        if *piece != self.current_turn && *piece != Cell::King {
            return false; // Must move your own piece
        }

        if self.board[to.0][to.1] != Cell::Empty {
            return false; // Destination must be empty
        }

        // Ensure it's a straight-line move
        if from.0 != to.0 && from.1 != to.1 {
            return false;
        }

        // Ensure path is clear
        if !self.is_path_clear(from, to) {
            return false;
        }

        true
    }

    fn check_win_condition(&self) -> Option<Cell> {
        // Check if the king reached an edge
        for i in 0..self.board.len() {
            if self.board[0][i] == Cell::King
                || self.board[self.board.len() - 1][i] == Cell::King
                || self.board[i][0] == Cell::King
                || self.board[i][self.board.len() - 1] == Cell::King
            {
                return Some(Cell::King); // King wins
            }
        }

        // Check if the king is surrounded
        let king_pos = self
            .board
            .iter()
            .enumerate()
            .find_map(|(r, row)| row.iter().position(|&c| c == Cell::King).map(|c| (r, c)));

        if let Some((kr, kc)) = king_pos {
            let neighbors = [
                (kr.wrapping_sub(1), kc),
                (kr + 1, kc),
                (kr, kc.wrapping_sub(1)),
                (kr, kc + 1),
            ];

            if neighbors
                .iter()
                .all(|&(nr, nc)| self.is_within_bounds((nr, nc)) && self.board[nr][nc] == Cell::Attacker)
            {
                return Some(Cell::Attacker); // Attackers win
            }
        }

        None
    }

}
