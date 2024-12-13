#[warn(unused_variables)]

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CellType {
    Empty,
    Attacker,
    Defender,
    King,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Cell {
    pub cell_type: CellType,
    pub is_corner: bool,
    pub is_throne: bool,
}


impl fmt::Display for CellType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CellType::Empty => write!(f, "Empty"),
            CellType::Attacker => write!(f, "Attacker"),
            CellType::Defender => write!(f, "Defender"),
            CellType::King => write!(f, "King"),
        }
    }
}

impl fmt::Display for Cell {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // Build the string for cell type and additional information (Corner and/or Throne)
        let mut display_str = self.cell_type.to_string(); // Get the cell's type as string

        // Append Corner or Throne information
        if self.is_corner {
            display_str.push_str(" (Corner)");
        }
        if self.is_throne {
            display_str.push_str(" (Throne)");
        }

        // Write the final string to the formatter
        write!(f, "{}", display_str)
    }
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
        let mut board = vec![
            vec![
                Cell {
                    cell_type: CellType::Empty,
                    is_corner: false,
                    is_throne: false
                }; 11
            ];
            11
        ];

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
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Attacker,
                is_corner: false,
                is_throne: false,
            };
        }

        // Place defenders (white)
        for &pos in &[ 
            (3, 5),
            (4, 4), (4, 5), (4, 6),            
            (5, 3), (5, 4), (5, 6), (5, 7),
            (6, 4), (6, 5), (6, 6),
            (7, 5),
        ] {
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Defender,
                is_corner: false,
                is_throne: false,
            };
        }

        // Place corners (marking corner cells with is_corner = true)
        for &pos in &[ 
            (0, 0), (0, 10), (10, 0), (10, 10)
        ] {
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Empty, // Corner is empty but still marked as a corner
                is_corner: true,
                is_throne: false,
            };
        }

        // Place the king (with a specific cell type)
        board[5][5] = Cell {
            cell_type: CellType::King,
            is_corner: false,
            is_throne: true,
        };

        // Return the GameState instance
        GameState {
            board,
            current_turn: Cell {
                cell_type: CellType::Attacker,
                is_corner: false,
                is_throne: false, // Current turn doesn't relate to corners directly
            },
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
    
        let clicked_cell = &self.board[row][col];
    
        if self.click_count % 2 == 1 {
            // First click: Select a piece to move
            if clicked_cell.cell_type == CellType::Empty {
                return Err("Select a piece to move.".to_string());
            }
            else if clicked_cell.cell_type == CellType::Defender && self.current_turn.cell_type == CellType::Attacker {
                return Err("Cannot move the defender's piece.".to_string());
            }
            else if clicked_cell.cell_type == CellType::King && self.current_turn.cell_type == CellType::Attacker {
                return Err("Cannot move the defender's piece.".to_string());
            }
            else if clicked_cell.cell_type == CellType::Attacker && self.current_turn.cell_type == CellType::Defender {
                return Err("Cannot move the attacker's piece.".to_string());
            }
            else {
                self.click_count += 1;
                self.from = (row, col);
            }
        } else {
            // Second click: Select an empty cell to move to
            if clicked_cell.cell_type != CellType::Empty {
                self.click_count -= 1;
                return Err("Select an empty cell to move to.".to_string());
            }
            else if clicked_cell.is_corner && self.board[self.from.0][self.from.1].cell_type != CellType::King {
                self.click_count -= 1;
                return Err("Cannot move to the corner.".to_string());
            }
            else if clicked_cell.is_throne && self.board[self.from.0][self.from.1].cell_type != CellType::King {
                self.click_count -= 1;
                return Err("Cannot move to the throne.".to_string());
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
        let mut moved_piece = self.board[from.0][from.1].clone();
        if moved_piece.is_throne == true{
            moved_piece.is_throne = false;
        }

        // Place the piece at the new position
        if self.board[to.0][to.1].is_throne == false{
            self.board[to.0][to.1] = moved_piece; 
        } else { 
            self.board[to.0][to.1] = moved_piece;
            self.board[to.0][to.1].is_throne = true;
        }
    
        // Clear the original position
        if self.board[from.0][from.1].is_throne {
            self.board[from.0][from.1] = Cell {
                cell_type: CellType::Empty,
                is_corner: false,
                is_throne: true, // The original cell was the throne
            };
        } else {
            self.board[from.0][from.1] = Cell {
                cell_type: CellType::Empty,
                is_corner: false,
                is_throne: false, // The original cell was not the throne
            };
        }
    
        // Check for captures at the new position
        self.check_captures(to)?;
    
        // Check win conditions
        if let Some(winner) = self.check_win_condition() {
            self.game_over = true;
            self.winner = Some(winner);
            let mut win_msg = winner.to_string();
            win_msg.push_str(" wins!");
            return Err(win_msg);
        } else {
            // Switch turns
            self.current_turn = if self.current_turn.cell_type == CellType::Attacker {
                Cell {
                    cell_type: CellType::Defender,
                    is_corner: false, // or true, depending on your game rules
                    is_throne: false,
                }
            } else {
                Cell {
                    cell_type: CellType::Attacker,
                    is_corner: false, // or true, depending on your game rules
                    is_throne: false,
                }
            };
        }
    
        Ok(())
    }
    
       

    pub fn check_captures(&mut self, pos: (usize, usize)) -> Result<(), String> {
        let neighbors = [
            (pos.0.wrapping_sub(1), pos.1), // Up
            (pos.0 + 1, pos.1), // Down
            (pos.0, pos.1.wrapping_sub(1)), // Left
            (pos.0, pos.1 + 1), // Right
        ];
    
        let cell = self.board[pos.0][pos.1].clone(); // Clone the current cell (with cell_type and is_corner)
    
        for (i, &(nx, ny)) in neighbors.iter().enumerate() {
            if self.is_within_bounds((nx, ny)) {
                let (nnx, nny) = match i {
                    0 => if nx > 0 { (nx - 1, ny) } else { continue },      // Up (check the cell above the neighbor)
                    1 => (nx + 1, ny),                                      // Down (check the cell below the neighbor)
                    2 => if ny > 0 { (nx, ny - 1) } else { continue },      // Left (check the cell to the left of the neighbor)
                    3 => (nx, ny + 1),                                      // Right (check the cell to the right of the neighbor)
                    _ => unreachable!(),
                };
    
                // Determine the opposite piece
                let opposite = if cell.cell_type == CellType::Attacker {
                    CellType::Defender
                } else {
                    CellType::Attacker
                };
    
                // Check if the neighbor is an opponent's piece and the adjacent piece is the same player's or a corner
                if self.board[nx][ny].cell_type == opposite
                    && self.is_within_bounds((nnx, nny))
                    && (self.board[nnx][nny].cell_type == cell.cell_type || self.board[nnx][nny].is_corner || self.board[nnx][nny].cell_type == CellType::King)
                {
                    // Capture the opponent's piece by setting it to Empty
                    self.board[nx][ny] = Cell {
                        cell_type: CellType::Empty,
                        is_corner: false, // Reset the corner status after capture
                        is_throne: false,
                    };
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
            range.iter().all(|&c| self.board[row][c].cell_type == CellType::Empty)
        } else if col == to.1 {
            // Vertical move
            let range: Vec<_> = if row < to.0 {
             (row + 1..=to.0).collect()
            } else {
                (to.0..=row - 1).rev().collect()
            };
            range.iter().all(|&r| self.board[r][col].cell_type == CellType::Empty)
        } else {
            false
        }
    }

    fn is_valid_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        if from == to || !self.is_within_bounds(to) {
            return false;
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
        // Check if the king reached an edge (the edges are corners)
        if self.board[0][self.board.len() - 1].cell_type == CellType::King
            || self.board[self.board.len() - 1][0].cell_type == CellType::King
            || self.board[self.board.len() - 1][0].cell_type == CellType::King
            || self.board[0][self.board.len() - 1].cell_type == CellType::King
        {
            return Some(Cell {
                cell_type: CellType::King,
                is_corner: false, // This is up to your game logic to define
                is_throne: false,
            });
        }
    
        // Check if the king is surrounded
        let king_pos = self
            .board
            .iter()
            .enumerate()
            .find_map(|(r, row)| row.iter().position(|c| c.cell_type == CellType::King).map(|c| (r, c)));
    
        if let Some((kr, kc)) = king_pos {
            let neighbors = [
                (kr.wrapping_sub(1), kc),
                (kr + 1, kc),
                (kr, kc.wrapping_sub(1)),
                (kr, kc + 1),
            ];
    
            if neighbors
                .iter()
                .all(|&(nr, nc)| {
                    self.is_within_bounds((nr, nc))
                        && self.board[nr][nc].cell_type == CellType::Attacker
                })
            {
                return Some(Cell {
                    cell_type: CellType::Attacker,
                    is_corner: false,
                    is_throne: false
                }); // Attackers win
            }
        }
    
        None
    }
    
}
