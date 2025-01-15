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
    pub is_selected: bool,
    pub is_possible_move: bool,
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
        if self.is_selected {
            display_str.push_str(" (Selected)");
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
    pub board_message: String, // Message to display on the board
    pub game_title: String,
    pub last_click: (usize, usize), // Last clicked cell
    pub id: usize,
    pub move_done: bool,
}

impl GameState {
    /// Creates a new game with the initial Hnefatafl board.
    pub fn new(id: usize) -> Self {
        let mut board = vec![
            vec![
                Cell {
                    cell_type: CellType::Empty,
                    is_corner: false,
                    is_throne: false,
                    is_selected: false,
                    is_possible_move: false,
                }; 7
            ];
            7
        ];

        // Place attackers (black)
        for &pos in &[ 
            (0, 3),
            (1, 3),
            (3, 0), (3, 1), (3, 5), (3, 6),
            (5, 3),
            (6, 3),
        ] {
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Attacker,
                is_corner: false,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,  
            };
        }

        // Place defenders (white)
        for &pos in &[ 
            (2, 3),
            (3, 2), (3, 4),            
            (4, 3), 
        ] {
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Defender,
                is_corner: false,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            };
        }

        // Place corners (marking corner cells with is_corner = true)
        for &pos in &[ 
            (0, 0), (0, 6), (6, 0), (6, 6)
        ] {
            board[pos.0][pos.1] = Cell {
                cell_type: CellType::Empty, // Corner is empty but still marked as a corner
                is_corner: true,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            };
        }

        // Place the king (with a specific cell type)
        board[3][3] = Cell {
            cell_type: CellType::King,
            is_corner: false,
            is_throne: true,
            is_selected: false,
            is_possible_move: false,
        };

        // Return the GameState instance
        GameState {
            board,
            current_turn: Cell {
                cell_type: CellType::Attacker,
                is_corner: false,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            },
            game_over: false,
            winner: None,
            click_count: 1,
            from: (0, 0),
            board_message: "Current turn: Attacker".to_string(),
            game_title:"Brandubh".to_string(),
            last_click: (0, 0),
            id: id,
            move_done: false,
        }
    }
    
    pub fn process_click(&mut self, row: usize, col: usize) -> Result<(), String> {
        // Validate and process the click based on the game state
        if row >= self.board.len() || col >= self.board[0].len() {
            return Err("Invalid cell coordinates.".to_string());
        }

        if self.game_over {
            return Err("Game is already over.".to_string());
        }
        
        self.board[self.last_click.0][self.last_click.1].is_selected = false;   // Deselect last clicked cell
        self.board[row][col].is_selected = true;                                // Select the clicked cell
        let clicked_cell = &self.board[row][col];                        // Get the clicked cell
        self.last_click = (row, col);                                           // Update the last clicked cell

    
        if self.click_count % 2 == 1 {
            // First click: Select a piece to move
            if clicked_cell.cell_type == CellType::Empty {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type == CellType::Defender && self.current_turn.cell_type == CellType::Attacker {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type == CellType::King && self.current_turn.cell_type == CellType::Attacker {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type == CellType::Attacker && self.current_turn.cell_type == CellType::Defender {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else {
                self.click_count += 1;
                self.from = (row, col);

                let possible_moves = self.calculate_valid_moves(self.from);

                for cell in possible_moves {
                    self.board[cell.0][cell.1].is_possible_move = true;
                }
                self.move_done = false;
            }
        } else {
            // Second click: Select an empty cell to move to
            if clicked_cell.cell_type == CellType::Defender && self.current_turn.cell_type == CellType::Attacker {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type == CellType::King && self.current_turn.cell_type == CellType::Attacker {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type == CellType::Attacker && self.current_turn.cell_type == CellType::Defender {
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.cell_type != CellType::Empty {

                if self.board[self.last_click.0][self.last_click.1].cell_type == clicked_cell.cell_type {
                    for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                    self.board[self.last_click.0][self.last_click.1].is_selected = false;  
                    self.board[row][col].is_selected = true;                                                    
                    self.last_click = (row, col);
                    self.from = (row, col);
                    let possible_moves = self.calculate_valid_moves(self.from);

                    for cell in possible_moves {
                        self.board[cell.0][cell.1].is_possible_move = true;
                    }
                    self.move_done = false;
                } else {
                    self.click_count -= 1;
                    self.board[self.last_click.0][self.last_click.1].is_selected = false;
                    for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                            cell.is_possible_move = false;
                        }
                }
                
                return Ok(());
            }
            else if clicked_cell.is_corner && self.board[self.from.0][self.from.1].cell_type != CellType::King {
                self.click_count -= 1;
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else if clicked_cell.is_throne && self.board[self.from.0][self.from.1].cell_type != CellType::King {
                self.click_count -= 1;
                self.board[self.last_click.0][self.last_click.1].is_selected = false;
                for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                        cell.is_possible_move = false;
                    }
                return Ok(());
            }
            else {
                // Make the move
                match self.make_move(self.from, (row, col)) {
                    Ok(_) => {
                        self.click_count += 1;

                        for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                            cell.is_possible_move = false;
                        }
                    }
                    Err(error_message) => {
                        return Err(error_message.to_string());
                    }
                }
            }
        }
        Ok(())
    }

    pub fn calculate_valid_moves(&self, start: (usize, usize)) -> Vec<(usize, usize)> {
        let mut valid_moves = Vec::new();
        let (start_row, start_col) = start;

        if !self.is_within_bounds(start) {
            return valid_moves;
        }

        let cell = self.board[start_row][start_col];
        if cell.cell_type == CellType::Empty {
            return valid_moves; // Cannot move from an empty cell
        }

        // Directions: up, down, left, right
        let directions = [
            (-1, 0), // Up
            (1, 0),  // Down
            (0, -1), // Left
            (0, 1),  // Right
        ];

        for &(d_row, d_col) in &directions {
            let mut row = start_row as isize;
            let mut col = start_col as isize;

            loop {
                row += d_row;
                col += d_col;

                if row < 0 || col < 0 || row >= self.board.len() as isize || col >= self.board[0].len() as isize {
                    break; // Out of bounds
                }

                let next_cell = &self.board[row as usize][col as usize];

                if cell.cell_type != CellType::King{
                    if next_cell.cell_type != CellType::Empty || next_cell.is_corner{
                        break; // Stop if cell is not empty or is a corner, and piece is not a king
                    }
                } else {
                    if next_cell.cell_type != CellType::Empty{
                        break; // Stop if cell is not empty, and piece is a king
                    }
                }

                valid_moves.push((row as usize, col as usize));
            }
        }

        if cell.cell_type != CellType::King {
            valid_moves.retain(|&x| x != (3, 3));
        }
        
        valid_moves
    }
    
    pub fn make_move(&mut self, from: (usize, usize), to: (usize, usize)) -> Result<(), String> {    
        // Validate the move
        if !self.is_valid_move(from, to) {
            self.board[self.last_click.0][self.last_click.1].is_selected = false;
            for cell in self.board.iter_mut().flat_map(|r| r.iter_mut()) {
                    cell.is_possible_move = false;
                }
            return Ok(());
        }
    
        // Make the move
        let mut moved_piece = self.board[from.0][from.1].clone();
        if moved_piece.is_throne {
            moved_piece.is_throne = false;
        }

        // Place the piece at the new position
        if !self.board[to.0][to.1].is_throne && !self.board[to.0][to.1].is_corner {
            self.board[to.0][to.1] = moved_piece; 
        } else if self.board[to.0][to.1].is_throne { 
            self.board[to.0][to.1] = moved_piece;
            self.board[to.0][to.1].is_throne = true;
        } else if self.board[to.0][to.1].is_corner {
            self.board[to.0][to.1] = moved_piece;
            self.board[to.0][to.1].is_corner = true;            
        }

        // Clear the original position
        if self.board[from.0][from.1].is_throne {
            self.board[from.0][from.1] = Cell {
                cell_type: CellType::Empty,
                is_corner: false,
                is_throne: true,
                is_selected: false,
                is_possible_move: false,
            };
        } else if self.board[from.0][from.1].is_corner {
            self.board[from.0][from.1] = Cell {
                cell_type: CellType::Empty,
                is_corner: true,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            };
        } else {
            self.board[from.0][from.1] = Cell {
                cell_type: CellType::Empty,
                is_corner: false,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
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
            self.board_message = win_msg;
        } else {
            // Switch turns
            if self.current_turn.cell_type == CellType::Attacker {
                self.current_turn =  Cell {
                    cell_type: CellType::Defender,
                    is_corner: false,
                    is_throne: false,
                    is_selected: false,
                    is_possible_move: false,
                };
                self.board_message = "Current turn: Defender".to_string();
                self.move_done = true;
            } else {
                self.current_turn =  Cell {
                    cell_type: CellType::Attacker,
                    is_corner: false,
                    is_throne: false,
                    is_selected: false,
                    is_possible_move: false,
                };
                self.board_message = "Current turn: Attacker".to_string();
                self.move_done = true;
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
                let opposite = if cell.cell_type == CellType::Defender || cell.cell_type == CellType::King {
                    CellType::Attacker
                } else {
                    CellType::Defender
                };

                if opposite == CellType::Attacker {
                    // Check if the neighbor is an opponent's piece and the adjacent piece is the same player's or a corner
                    if self.board[nx][ny].cell_type == opposite
                        && self.is_within_bounds((nnx, nny))
                        && (self.board[nnx][nny].cell_type == cell.cell_type 
                            || self.board[nnx][nny].is_corner 
                            || self.board[nnx][nny].cell_type == CellType::King
                            || self.board[nnx][nny].cell_type == CellType::Defender)
                    {
                        // Capture the opponent's piece by setting it to Empty
                        self.board[nx][ny] = Cell {
                            cell_type: CellType::Empty,
                            is_corner: false, // Reset the corner status after capture
                            is_throne: false,
                            is_selected: false,
                            is_possible_move: false,
                        };
                    }
                } else {
                    // Check if the neighbor is an opponent's piece and the adjacent piece is the same player's or a corner
                    if self.board[nx][ny].cell_type == opposite
                        && self.is_within_bounds((nnx, nny))
                        && (self.board[nnx][nny].cell_type == cell.cell_type 
                            || self.board[nnx][nny].is_corner)
                    {
                        // Capture the opponent's piece by setting it to Empty
                        self.board[nx][ny] = Cell {
                            cell_type: CellType::Empty,
                            is_corner: false, // Reset the corner status after capture
                            is_throne: false,
                            is_selected: false,
                            is_possible_move: false,
                        };
                    }
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
            range.iter().all(|&c| self.board[row][c].cell_type == CellType::Empty )
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
            || self.board[self.board.len() - 1][self.board.len() - 1].cell_type == CellType::King
            || self.board[0][0].cell_type == CellType::King
        {
            return Some(Cell {
                cell_type: CellType::Defender,
                is_corner: false, // This is up to your game logic to define
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            });
        }
    
        // Check if the king is surrounded
        let king_pos = self
            .board
            .iter()
            .enumerate()
            .find_map(|(r, row)| row.iter().position(|c| c.cell_type == CellType::King).map(|c| (r, c)));
        
        if self.current_turn.cell_type == CellType::Attacker {
            if let Some((kr, kc)) = king_pos {
                let neighbors = [
                    (kr.wrapping_sub(1), kc),
                    (kr + 1, kc),
                    (kr, kc.wrapping_sub(1)),
                    (kr, kc + 1),
                ];
        
                if neighbors
                    .iter()
                    .filter(|&&(nr, nc)| self.is_within_bounds((nr, nc)))
                    .filter(|&&(nr, nc)| {
                        nr == kr && (nc == kc.wrapping_sub(1) || nc == kc + 1) && self.board[nr][nc].cell_type == CellType::Attacker})
                    .count() == 2 
                {
                    return Some(Cell {
                        cell_type: CellType::Attacker,
                        is_corner: false,
                        is_throne: false,
                        is_selected: false,
                        is_possible_move: false,
                    }); // Attackers win
                } else if neighbors
                    .iter()
                    .filter(|&&(nr, nc)| self.is_within_bounds((nr, nc)))
                    .filter(|&&(nr, nc)| {
                        nc == kc && (nr == kr.wrapping_sub(1) || nr == kr + 1) && self.board[nr][nc].cell_type == CellType::Attacker})
                    .count() == 2 
                {
                    return Some(Cell {
                        cell_type: CellType::Attacker,
                        is_corner: false,
                        is_throne: false,
                        is_selected: false,
                        is_possible_move: false,
                    }); // Attackers win
                }
            }
        }

        // Check if there are no valid moves for any defender
        let no_valid_moves = self.board.iter().enumerate().all(|(r, row)| {
            row.iter().enumerate().all(|(c, cell)| {
                if cell.cell_type == CellType::Defender || cell.cell_type == CellType::King {
                    // Check if this defender has any valid moves
                    self.calculate_valid_moves((r, c)).is_empty()
                } else {
                    true
                }
            })
        });
        
        if no_valid_moves {
            return Some(Cell {
                cell_type: CellType::Attacker,
                is_corner: false,
                is_throne: false,
                is_selected: false,
                is_possible_move: false,
            }); // Attackers win
        }
    
        None
    }
    
}
