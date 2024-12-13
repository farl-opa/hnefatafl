#[warn(unused_variables)]

use warp::Filter;
use std::sync::Arc;
mod hnefatafl;
use hnefatafl::{GameState, Cell, CellType};
use serde::Deserialize;
use tokio::sync::RwLock;

#[derive(Deserialize)]
struct CellClick {
    row: usize,
    col: usize,
}

#[derive(Clone)]
struct AppState {
    pub games: Arc<RwLock<Vec<Option<GameState>>>>, // Use Option to mark ended games
}

#[tokio::main]
async fn main() {

    // Static file serving for images
    let static_files = warp::path("images").and(warp::fs::dir("./static/images"));


    let state = AppState {
        games: Arc::new(RwLock::new(Vec::new())),
    };

    let state_filter = warp::any().map(move || state.clone());

    // Root endpoint to display the Starting Menu
    let root = warp::path::end()
    .and(warp::get())
    .and(state_filter.clone())
    .and_then(|state: AppState| async move {
        let games = state.games.write().await;
        if let Some(Some(_game)) = games.get(0) {
            let response = r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Hnefatafl</title>
                </head>
                <body>
                    <h1 style="text-align: center;">Welcome to the Hnefatafl Server!</h1>
                    <form action="/new" method="post" style="text-align: center; margin-top: 20px;">
                        <button type="submit">Start New Game</button>
                    </form>
                    <form action="/continue" method="post" style="text-align: center; margin-top: 20px;">
                        <button type="submit">Continue Last Game</button>
                    </form>
                </body>
                </html>"#;
            Ok::<_, warp::Rejection>(warp::reply::html(response.to_string()))
            
        } else {
            let response = r#"<!DOCTYPE html>
                <html lang="en">
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <title>Hnefatafl</title>
                </head>
                <body>
                    <h1 style="text-align: center;">Welcome to the Hnefatafl Server!</h1>
                    <form action="/new" method="post" style="text-align: center; margin-top: 20px;">
                        <button type="submit">Start New Game</button>
                    </form>
                </body>
                </html>"#;
            Ok::<_, warp::Rejection>(warp::reply::html(response.to_string()))
        }
    });

    // Endpoint: Create a new game
    let new_game = warp::path("new")
        .and(warp::post().or(warp::get())) // Accept both POST and GET
        .unify()
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let game = GameState::new();
            let board_html = render_board_as_html(&game.board);
            let current_turn = game.current_turn.cell_type.clone();
            games.push(Some(game)); // Store the new game
            let response = format!(
                r#"<!DOCTYPE html>
                <html lang="en">
                <html>
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <h1 style="text-align: center;">Hnefatafl Game</h1>
                    <h2 style="text-align: center;">Current turn: {}</h2>
                    {}
                    <script>
                        function handleCellClick(row, col) {{
                            fetch('/cell-click', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/json'
                                }},
                                body: JSON.stringify({{ row: row, col: col }})
                            }})
                            .then(response => response.json())
                            .then(data => {{
                                if (data.success) {{
                                    // Update the board on success
                                    document.getElementById('board-container').innerHTML = data.board_html;
                                    document.querySelector('h2').innerText = 'Current turn: ' + data.current_turn;
                                }} else {{
                                    alert(data.error || 'An error occurred');
                                }}
                            }})
                            .catch(error => console.error('Error:', error));
                        }}
                    </script>
                </head>
                <body>
                    <div id="board-container">
                        {}
                    </div>
                </body>
                </html>"#,
                current_turn, CSS, board_html
            );
            Ok::<_, warp::Rejection>(warp::reply::html(response))
        });

    // Endpoint to handle cell clicks
    let cell_click = warp::path("cell-click")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(|click: CellClick, state: AppState| async move {
            let mut games = state.games.write().await;
            if let Some(game) = games.last_mut().and_then(Option::as_mut) {
                // Call process_click and handle the result
                match game.process_click(click.row, click.col) {
                    Ok(_) => {
                        let board_html = render_board_as_html(&game.board);
                        let current_turn = game.current_turn.cell_type.clone();
                        Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                            "success": true,
                            "board_html": board_html,
                            "current_turn": current_turn
                        })))
                    }
                    Err(error_message) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                            "success": false,
                            "error": error_message
                        })))
                    }
                }
            } else {
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "No active game"
                })))
            }
        });


    // Endpoint: Make a move
    let make_move = warp::path("move")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and_then(|move_request: MoveRequest, state: AppState| async move {
            let mut games = state.games.write().await;
            if let Some(Some(game)) = games.get_mut(move_request.game_id) {
                match game.make_move(move_request.from, move_request.to) {
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&game)),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&e)),
                }
            } else {
                Ok::<_, warp::Rejection>(warp::reply::json(&"Game not found or ended"))
            }
        });


    // Endpoint: Continue the last game
    let continue_game = warp::path("continue")
    .and(warp::post())
    .and(state_filter.clone())
    .and_then(|state: AppState| async move {
        let games = state.games.write().await;
        if let Some(Some(_game)) = games.last() {
            Ok::<_, warp::Rejection>(warp::reply::html("Continuing the last game..."))
        } else {
            Ok::<_, warp::Rejection>(warp::reply::html("No game to continue!"))
        }
    });

    // Endpoint: List all games
    let list_games = warp::path("list")
        .and(warp::get())  
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let games = state.games.write().await;
            let game_list: Vec<(usize, String)> = games
                .iter()
                .enumerate()
                .filter_map(|(id, game)| {
                    game.as_ref().map(|g| {
                        let status = if g.game_over {
                            format!("Game over - Winner: {:?}", g.winner)
                        } else {
                            format!("In progress - Current turn: {:?}", g.current_turn)
                        };
                        (id, status)
                    })
                })
                .collect();
            Ok::<_, warp::Rejection>(warp::reply::json(&game_list))
        });

    // Endpoint: Query a game state
    let query_game = warp::path("query")
        .and(warp::get())
        .and(warp::path::param::<usize>()) // Accept game ID as a path parameter
        .and(state_filter.clone())
        .and_then(|game_id: usize, state: AppState| async move {
            let games = state.games.write().await;
            if let Some(Some(game)) = games.get(game_id) {
                Ok::<_, warp::Rejection>(warp::reply::json(&game))
            } else {
                Ok::<_, warp::Rejection>(warp::reply::json(&"Game not found or ended"))
            }
        });

    // Endpoint: End a game session
    let end_game = warp::path("end")
        .and(warp::post())
        .and(warp::path::param::<usize>()) // Accept game ID as a path parameter
        .and(state_filter.clone())
        .and_then(|game_id: usize, state: AppState| async move {
            let mut games = state.games.write().await;
            if let Some(game) = games.get_mut(game_id) {
                *game = None; // Mark the game as ended
                Ok::<_, warp::Rejection>(warp::reply::json(&"Game ended successfully"))
            } else {
                Ok::<_, warp::Rejection>(warp::reply::json(&"Game not found"))
            }
        });

    // Combine all routes
    let routes = static_files
        .or(new_game)
        .or(list_games)
        .or(query_game)
        .or(end_game)
        .or(make_move)
        .or(continue_game)
        .or(cell_click)
        .or(root);

    // Start the server
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await
}

#[derive(Deserialize)]
struct MoveRequest {
    game_id: usize,
    from: (usize, usize),
    to: (usize, usize),
}


/// Helper function to render the board as an HTML table
fn render_board_as_html(board: &Vec<Vec<Cell>>) -> String {
    let mut html = String::from("<table>");

    // Add rows with board cells and right-side coordinates
    for (row_idx, row) in board.iter().enumerate() {
        html.push_str("<tr>"); // Start a new row

        let mut col_idx = 0;
        for cell in row {
            // Determine the class and content based on the cell type
            let (class, content) = match cell.cell_type {
                CellType::Empty => ("empty", ""),
                CellType::Attacker => (
                    "attacker",
                    r#"<img src="/images/attacker.png" alt="Attacker" class="piece" />"#,
                ),
                CellType::Defender => (
                    "defender",
                    r#"<img src="/images/defender.png" alt="Defender" class="piece" />"#,
                ),
                CellType::King => (
                    "king",
                    r#"<img src="/images/king.png" alt="King" class="piece" />"#,
                ),
            };

            // If the cell is a corner, you can add specific styles or content for corners
            let corner_class = if cell.is_corner {" corner-cell" } else { "" };

            // Render the cell as an HTML table cell (<td>)
            html.push_str(&format!(
                r#"<td id="cell-{}-{}" class="{}{}" onclick="handleCellClick({}, {})">{}</td>"#,
                row_idx, col_idx, class, corner_class, row_idx, col_idx, content
            ));
            col_idx += 1;
        }

        // Add the row number as a right-side coordinate (no border)
        html.push_str(&format!(
            r#"<td class="coordinates" style="border: none;">{}</td>"#,
            11 - row_idx
        ));

        html.push_str("</tr>"); // End the current row
    }

    // Add a bottom row for column coordinates (no border)
    html.push_str("<tr>");
    for col in 0..board[0].len() {
        html.push_str(&format!(
            r#"<td class="coordinates" style="border: none;">{}</td>"#,
            (b'a' + col as u8) as char
        ));
    }
    html.push_str("</tr>");

    html.push_str("</table>");
    html
}




// Add CSS updates for coordinates
const CSS: &str = r#"
<style>
    table {
        border-collapse: collapse;
        margin: 20px auto;
    }
    td {
        width: 40px;
        height: 40px;
        text-align: center;
        border: 1px solid black;
        font-weight: bold;
        font-size: 16px;
    }
    .piece {
        width: 35px;
        height: 35px;
    }
    .empty { background-color: #f0f0f0; }
    .attacker { background-color: #f0f0f0; }
    .defender { background-color: #f0f0f0; }
    .king { background-color: #f0f0f0; }
    .corner-cell { background-color: #8cf367; }
    .coordinates {
        font-size: 12px;
        font-weight: normal;
    }
</style>


"#;
