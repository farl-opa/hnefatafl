#[warn(unused_variables)]

use warp::Filter;
use warp::sse::Event;
use warp::sse::reply;
use warp::reply::html;
use warp::reject::Reject;
use warp::http::{Response, header::SET_COOKIE};
use tokio::sync::broadcast;
use std::sync::Arc;
use std::fmt;
mod hnefatafl;
use hnefatafl::{GameState, Cell, CellType};
use serde::Deserialize;
use tokio::sync::RwLock;
use std::collections::HashMap;
use uuid::Uuid;


#[derive(Deserialize)]
struct CellClick {
    row: usize,
    col: usize,
}

#[derive(Clone)]
struct AppState {
    pub games: Arc<RwLock<Vec<Option<GameState>>>>, // Use Option to mark ended games
    players: Arc<RwLock<HashMap<String, String>>>, // Maps session IDs to usernames
}

#[derive(Debug)]
struct MissingUsername;

impl fmt::Display for MissingUsername {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing username")
    }
}

impl Reject for MissingUsername {}

// Helper to get the session ID from the cookie
fn get_session_id_from_cookie(headers: &warp::http::HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find_map(|cookie| {
                    if cookie.trim_start().starts_with("session_id=") {
                        Some(cookie.trim_start()[11..].to_string()) // Extract session ID
                    } else {
                        None
                    }
                })
        })
}

#[tokio::main]
async fn main() {
    // Static file serving for images
    let static_files = warp::path("images").and(warp::fs::dir("./static/images"));

    // Initialize application state
    let state = AppState {
        games: Arc::new(RwLock::new(Vec::new())),
        players: Arc::new(RwLock::new(HashMap::new())),
    };

    let state_filter = warp::any().map(move || state.clone());

    // Root route to show the username form
    let username_form = warp::path::end()
        .and(warp::get())
        .map(|| {
            let html_content = r#"
            <!DOCTYPE html>
            <html lang="en">
            <head>
                <meta charset="UTF-8">
                <meta name="viewport" content="width=device-width, initial-scale=1.0">
                <title>Enter Username</title>
            </head>
            <body>
                <h1 style="text-align: center;">Enter Your Username</h1>
                <form action="/main" method="post" style="text-align: center; margin-top: 20px;">
                    <input type="text" name="username" placeholder="Enter username" required>
                    <button type="submit">Submit</button>
                </form>
            </body>
            </html>
            "#;

            // Return the HTML as a response
            html(html_content)
        });

    // Handle POST request for username submission and show main page
    let main_page_post = warp::path("main")
        .and(warp::post()) // POST method
        .and(warp::body::form()) // To receive form data
        .and(state_filter.clone()) // Access app state
        .and(warp::header::headers_cloned()) // Get the headers (to check cookies)
        .and_then(|form: HashMap<String, String>, state: AppState, headers: warp::http::HeaderMap| async move {
            // Check if the session_id exists in the cookies
            if let Some(session_id) = get_session_id_from_cookie(&headers) {
                // If session_id exists, use the existing username
                let players = state.players.read().await;
                if let Some(username) = players.get(&session_id) {
                    // Session already exists, don't ask for the username again
                    let players_html: String = players
                        .values()
                        .map(|username| format!("<p>{}</p>", username))
                        .collect();

                    // HTML response for the main page
                    let response = format!(
                        r#"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head>
                            <meta charset="UTF-8">
                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                            <title>Hnefatafl - Main</title>
                            <style>
                                body {{
                                    font-family: Arial, sans-serif;
                                    display: flex;
                                    flex-direction: column;
                                    align-items: center;
                                    justify-content: center;
                                    height: 100vh;
                                    margin: 0;
                                    text-align: center;
                                }}
                                .container {{
                                    width: 100%;
                                    max-width: 600px;
                                }}
                                h1, h2 {{
                                    margin-bottom: 20px;
                                }}
                                p {{
                                    font-size: 18px;
                                    margin: 5px 0;
                                }}
                                form {{
                                    margin-top: 20px;
                                }}
                                button {{
                                    padding: 10px 20px;
                                    font-size: 16px;
                                    cursor: pointer;
                                }}
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <h1>Welcome back, {}!</h1>
                                <h2>Players Online</h2>
                                <div>{}</div>
                                <form action="/new" method="post">
                                    <button type="submit">Start New Game</button>
                                </form>
                                <form action="/rules" method="get">
                                    <button type="submit">Game Rules</button>
                                </form>
                                <form action="/signout" method="post">
                                    <input type="hidden" name="session_id" value="{}">
                                    <button type="submit">Sign Out</button>
                                </form>
                            </div>
                        </body>
                        </html>
                        "#,
                        username, players_html, session_id
                    );

                    return Ok::<_, warp::Rejection>(Response::builder().body(response).unwrap());
                }
            }

            // If no session exists, proceed with creating a new session
            if let Some(username) = form.get("username") {
                let session_id = Uuid::new_v4().to_string(); // Generate a unique session ID
                state.players.write().await.insert(session_id.clone(), username.clone());

                // Set session_id in a cookie
                let cookie = format!("session_id={}; Path=/; HttpOnly;", session_id);

                // Now show the main page with the list of players
                let players = state.players.read().await; // Read the list of connected players

                // Build the players list in HTML
                let players_html: String = players
                    .values()
                    .map(|username| format!("<p>{}</p>", username))
                    .collect();

                // HTML response for the main page
                let response = format!(
                    r#"
                    <!DOCTYPE html>
                    <html lang="en">
                    <head>
                        <meta charset="UTF-8">
                        <meta name="viewport" content="width=device-width, initial-scale=1.0">
                        <title>Hnefatafl - Main</title>
                        <style>
                            body {{
                                font-family: Arial, sans-serif;
                                display: flex;
                                flex-direction: column;
                                align-items: center;
                                justify-content: center;
                                height: 100vh;
                                margin: 0;
                                text-align: center;
                            }}
                            .container {{
                                width: 100%;
                                max-width: 600px;
                            }}
                            h1, h2 {{
                                margin-bottom: 20px;
                            }}
                            p {{
                                font-size: 18px;
                                margin: 5px 0;
                            }}
                            form {{
                                margin-top: 20px;
                            }}
                            button {{
                                padding: 10px 20px;
                                font-size: 16px;
                                cursor: pointer;
                            }}
                        </style>
                    </head>
                    <body>
                        <div class="container">
                            <h1>Welcome to the Hnefatafl Server!</h1>
                            <h2>Players Online</h2>
                            <div>{}</div>
                            <form action="/new" method="post">
                                <button type="submit">Start New Game</button>
                            </form>
                            <form action="/rules" method="get">
                                <button type="submit">Game Rules</button>
                            </form>
                            <form action="/signout" method="post">
                                <input type="hidden" name="session_id" value="{}">
                                <button type="submit">Sign Out</button>
                            </form>
                        </div>
                    </body>
                    </html>
                    "#,
                    players_html, session_id
                );

                return Ok::<_, warp::Rejection>(Response::builder()
                    .header(SET_COOKIE, cookie)
                    .body(response)
                    .unwrap());
            }

            // If no username, reject the request
            Err(warp::reject::custom(MissingUsername))
        });


    // Handle GET request for the main page
    let main_page_get = warp::path("main")
        .and(warp::get()) // GET method
        .and(state_filter.clone()) // Access app state
        .and(warp::header::headers_cloned()) // Get the headers (to check cookies)
        .and_then(|state: AppState, headers: warp::http::HeaderMap| async move {
            // Extract session_id from the cookie
            if let Some(session_id) = get_session_id_from_cookie(&headers) {
                // Check if the session_id exists in the players map
                let players = state.players.read().await;
                if let Some(username) = players.get(&session_id) {
                    // Build the players list in HTML
                    let players_html: String = players
                        .values()
                        .map(|username| format!("<p>{}</p>", username))
                        .collect();

                    // HTML response for the main page
                    let response = format!(
                        r#"
                        <!DOCTYPE html>
                        <html lang="en">
                        <head>
                            <meta charset="UTF-8">
                            <meta name="viewport" content="width=device-width, initial-scale=1.0">
                            <title>Hnefatafl - Main</title>
                            <style>
                                body {{
                                    font-family: Arial, sans-serif;
                                    display: flex;
                                    flex-direction: column;
                                    align-items: center;
                                    justify-content: center;
                                    height: 100vh;
                                    margin: 0;
                                    text-align: center;
                                }}
                                .container {{
                                    width: 100%;
                                    max-width: 600px;
                                }}
                                h1, h2 {{
                                    margin-bottom: 20px;
                                }}
                                p {{
                                    font-size: 18px;
                                    margin: 5px 0;
                                }}
                                form {{
                                    margin-top: 20px;
                                }}
                                button {{
                                    padding: 10px 20px;
                                    font-size: 16px;
                                    cursor: pointer;
                                }}
                            </style>
                        </head>
                        <body>
                            <div class="container">
                                <h1>Welcome to the Hnefatafl Server, {}!</h1>
                                <h2>Players Online</h2>
                                <div>{}</div>
                                <form action="/new" method="post">
                                    <button type="submit">Start New Game</button>
                                </form>
                                <form action="/rules" method="get">
                                    <button type="submit">Game Rules</button>
                                </form>
                                <form action="/signout" method="post">
                                    <input type="hidden" name="session_id" value="{}">
                                    <button type="submit">Sign Out</button>
                                </form>
                            </div>
                        </body>
                        </html>
                        "#,
                        username, players_html, session_id
                    );

                    Ok::<_, warp::Rejection>(Response::builder()
                        .body(response)
                        .unwrap())
                } else {
                    // If no username is found for the session, redirect to a login page
                    Err(warp::reject::not_found())
                }
            } else {
                // If no session_id is found in cookies, redirect to login page
                Err(warp::reject::not_found())
            }
        });


    // Handle POST request for signing out
    let sign_out_post = warp::path("signout")
        .and(warp::post()) // POST method
        .and(warp::body::form()) // To receive form data
        .and(state_filter.clone())
        .and_then(|form: HashMap<String, String>, state: AppState| async move {
            let session_id = form.get("session_id").expect("Session ID must be present");

            let mut players = state.players.write().await;
            players.remove(session_id); // Remove the player from the list

            // Redirect to the username input page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", "/")
                .body("Redirecting...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });


    // Endpoint: Display the rules
    let rules = warp::path("rules")
    .and(warp::get())
    .and_then(|| async {
        let response = r#"<!DOCTYPE html>
        <html lang="en">
        <head>
            <meta charset="UTF-8">
            <meta name="viewport" content="width=device-width, initial-scale=1.0">
            <title>Game Rules - Hnefatafl</title>
            <style>
                body {
                    line-height: 1.6;
                    margin: 20px;
                }
                h1 {
                    text-align: center;
                    color: #555;
                }
                h2 {
                    text-align: center;
                }
                ul {
                    list-style-position: inside;
                    padding: 0;
                }
                ul li {
                    display: block;
                    text-align: center;
                }
                p, ul {
                    text-align: center;
                    margin-bottom: 1.2em;
                    max-width: 600px;
                    margin-left: auto;
                    margin-right: auto;
                }
            </style>
        </head>
        <body>
            <h1>Hnefatafl Game Rules</h1>
            <div>
                <p>Hnefatafl is an ancient Norse strategy board game. Two sides compete in an asymmetrical battle. The Defenders protect their King, while the Attackers attempt to capture him.</p>
                
                <h2>Objective</h2>
                <ul>
                    <li><strong>Defenders:</strong> Help the King escape to one of the four corners of the board.</li>
                    <li><strong>Attackers:</strong> Capture the King by surrounding him on all four sides.</li>
                </ul>
                
                <h2>Board Setup</h2>
                <ul>
                    <li>The King starts on the central square (the Throne).</li>
                    <li>Defenders are symmetrically placed around the King.</li>
                    <li>Attackers are placed on the board edges, forming a cross-like pattern.</li>
                </ul>
                
                <h2>Movement</h2>
                <ul>
                    <li>All pieces move horizontally or vertically, like a rook in chess.</li>
                    <li>Pieces cannot move through or land on other pieces.</li>
                    <li>Only the King may occupy the Throne or escape to a corner square.</li>
                </ul>
                
                <h2>Capturing</h2>
                <ul>
                    <li>A piece is captured by being sandwiched between two opposing pieces.</li>
                    <li>The King is captured by surrounding him on all four sides.</li>
                    <li>If adjacent to the Throne or an edge, the King is captured when surrounded on the remaining three sides.</li>
                </ul>
                
                <h2>Reinforced Corners Rule</h2>
                <p>The Throne and corners can act as allies for capturing enemy pieces. For example, if an Attacker is sandwiched between a Defender and the Throne, the Attacker is captured.</p>
                
                <h2>Game End</h2>
                <ul>
                    <li>The Defenders win if the King escapes to a corner.</li>
                    <li>The Attackers win if the King is captured.</li>
                </ul>
                
                <div class="back-link">
                    <form action="/main" method="get" style="text-align: center; margin-top: 20px;">
                        <button type="submit" class="back-button">Back to Home</button>
                    </form>
                </div>

            </div>
        </body>
        </html>"#;

        Ok::<_, warp::Rejection>(warp::reply::html(response.to_string()))
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
            let board_message = game.board_message.clone();        
            games.push(Some(game)); // Store the new game
            let response = format!(
                r#"<!DOCTYPE html>
                <html lang="en">
                <html>
                <head>
                    <meta charset="UTF-8">
                    <meta name="viewport" content="width=device-width, initial-scale=1.0">
                    <h1 style="text-align: center;">Hnefatafl Game</h1>
                    <h2 style="text-align: center;">{}</h2>
                    {}
                    <script>
                        // Establish an SSE connection
                        const eventSource = new EventSource('/board-updates');

                        eventSource.onmessage = function(event) {{
                            const data = JSON.parse(event.data);
                            document.getElementById('board-container').innerHTML = data.board_html;
                            document.querySelector('h2').innerText = data.board_message;
                        }};

                        function handleCellClick(row, col) {{
                            fetch('/cell-click', {{
                                method: 'POST',
                                headers: {{
                                    'Content-Type': 'application/json'
                                }},
                                body: JSON.stringify({{ row: row, col: col }})
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
                board_message, CSS, board_html
            );
            Ok::<_, warp::Rejection>(warp::reply::html(response))
        });

    // Create a broadcast channel for board updates
    let board_updates_tx = Arc::new(broadcast::channel::<String>(100).0);

    let board_updates = warp::path("board-updates")
        .and(warp::get())
        .and({
            let board_updates_tx = board_updates_tx.clone();
            warp::any().map(move || board_updates_tx.subscribe())
        })
        .map(|mut rx: broadcast::Receiver<String>| {
            reply(warp::sse::keep_alive().stream(async_stream::stream! {
                while let Ok(message) = rx.recv().await {
                    yield Ok::<_, warp::Error>(Event::default()
                        .data(message));
                }
            }))
        });


    // Endpoint to handle cell clicks
    let cell_click = warp::path("cell-click")
        .and(warp::post())
        .and(warp::body::json())
        .and(state_filter.clone())
        .and(warp::any().map({
            let board_updates_tx = board_updates_tx.clone();
            move || board_updates_tx.clone()
        }))
        .and_then(
            |click: CellClick, state: AppState, board_updates_tx: Arc<broadcast::Sender<String>>| async move {
                let mut games = state.games.write().await;
                if let Some(game) = games.last_mut().and_then(Option::as_mut) {
                    match game.process_click(click.row, click.col) {
                        Ok(_) => {
                            let board_html = render_board_as_html(&game.board);
                            let board_message = game.board_message.clone();

                            // Broadcast the new board state
                            let update = serde_json::to_string(&serde_json::json!({
                                "board_html": board_html,
                                "board_message": board_message,
                            }))
                            .unwrap();
                            let _ = board_updates_tx.send(update); // Ignore errors if no subscribers

                            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "success": true,
                                "board_html": board_html,
                                "board_message": board_message,
                            })))
                        }
                        Err(error_message) => {
                            let board_html = render_board_as_html(&game.board);
                            let board_message = game.board_message.clone();

                            // Broadcast the new board state even if there's an error
                            let update = serde_json::to_string(&serde_json::json!({
                                "board_html": board_html,
                                "board_message": board_message,
                            }))
                            .unwrap();
                            let _ = board_updates_tx.send(update); // Ignore errors if no subscribers

                            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": error_message,
                                "board_html": board_html,
                                "board_message": board_message,
                            })))
                        }
                    }
                } else {
                    Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                        "success": false,
                        "error": "No active game",
                        })))
                }
            },
        );

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
    
    // Endpoint: Refresh the board
    let refresh_board = warp::path("refresh-board")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let games = state.games.read().await;
            if let Some(Some(game)) = games.last() {
                let board_html = render_board_as_html(&game.board);
                let board_message = game.board_message.clone();
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": true,
                    "board_html": board_html,
                    "board_message": board_message,
                })))
            } else {
                Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "No active game"
                })))
            }
        });


    // Endpoint: Continue the last game
    let continue_game = warp::path("continue")
    .and(warp::post())
    .and(state_filter.clone())
    .and_then(|state: AppState| async move {
        let games = state.games.write().await;
        let response = if let Some(Some(_game)) = games.last() {
            warp::reply::html("Continuing the last game...")
        } else {
            warp::reply::html("No game to continue!")
        };
        Ok::<_, warp::Rejection>(response)
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
        .or(username_form)
        .or(main_page_get)
        .or(main_page_post)
        .or(sign_out_post)
        .or(rules)
        .or(new_game)
        .or(list_games)
        .or(query_game)
        .or(end_game)
        .or(make_move)
        .or(continue_game)
        .or(cell_click)
        .or(refresh_board)
        .or(board_updates);

    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
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

            // If the cell is a throne, you can add specific styles or content for corners
            let throne_class = if cell.is_throne {" throne-cell" } else { "" };

            // If the cell is selected, you can add specific styles or content for corners
            let selected_class = if cell.is_selected {" selected-cell" } else { "" };

            let possible_class = if cell.is_possible_move {" possible-cell" } else { "" };

            // Render the cell as an HTML table cell (<td>)
            html.push_str(&format!(
                r#"<td id="cell-{}-{}" class="{}{}{}{}{}" onclick="handleCellClick({}, {})">{}</td>"#,
                row_idx, col_idx, class, corner_class, throne_class, selected_class, possible_class, row_idx, col_idx, content
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
    .throne-cell { background-color: #D53E3E}
    .selected-cell { background-color: #8c8c8c; }
    .possible-cell { 
        position: relative;
    }
    .possible-cell::before {
        content: '';
        position: absolute;
        top: 50%;
        left: 50%;
        width: 10px;
        height: 10px;
        background-color: green;
        border-radius: 50%;
        transform: translate(-50%, -50%);
    }
    .coordinates {
        font-size: 12px;
        font-weight: normal;
    }
</style>


"#;
