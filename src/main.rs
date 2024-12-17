#[warn(unused_variables)]
use std::collections::HashMap;
use std::fmt;
use std::fs;
use std::sync::Arc;

use serde::Deserialize;
use tokio::sync::{broadcast, RwLock};

use uuid::Uuid;
use warp::{
    self,
    Filter,
    http::{Response, Method, header::SET_COOKIE},
    reject::Reject,
    reply::html,
    sse::{Event, reply},
    cors,
};

mod hnefatafl;
use hnefatafl::{GameState, Cell, CellType};



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

// Helper function to read the HTML template from a file
fn read_html_template(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
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
            let html_content = read_html_template("templates/username_form.html").unwrap();

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

                    // Read the HTML template from file
                    let template_path = "templates/main_page.html";
                    let template = read_html_template(template_path).unwrap();

                    // Replace placeholders in the template with dynamic content
                    let response = template
                        .replace("{welcome_message}", &format!("Welcome to the Hnefatafl server, {}!", username))
                        .replace("{players_html}", &players_html)
                        .replace("{session_id}", &session_id);

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

                // Read the HTML template from file
                let template_path = "templates/main_page.html";
                let template = read_html_template(template_path).unwrap();

                // Replace placeholders in the template with dynamic content
                let response = template
                    .replace("{welcome_message}", &format!("Welcome to the Hnefatafl server, {}!", username))
                    .replace("{players_html}", &players_html)
                    .replace("{session_id}", &session_id);

                return Ok::<_, warp::Rejection>(Response::builder()
                    .header(SET_COOKIE, cookie)
                    .body(response)
                    .unwrap());
            }

            // If no username, reject the request
            Err(warp::reject::custom(MissingUsername))
        });

    // Handle GET request for the main page (similarly as in the previous example)
    let main_page_get = warp::path("main")
        .and(warp::get()) // GET method
        .and(state_filter.clone()) // Access app state
        .and(warp::header::headers_cloned()) // Get the headers (to check cookies)
        .and_then(|state: AppState, headers: warp::http::HeaderMap| async move {
            if let Some(session_id) = get_session_id_from_cookie(&headers) {
                // Check if the session_id exists in the players map
                let players = state.players.read().await;
                if let Some(username) = players.get(&session_id) {
                    // Build the players list in HTML
                    let players_html: String = players
                        .values()
                        .map(|username| format!("<p>{}</p>", username))
                        .collect();

                    // Read the HTML template from file
                    let template_path = "templates/main_page.html";
                    let template = read_html_template(template_path).unwrap();

                    // Replace placeholders in the template with dynamic content
                    let response = template
                        .replace("{welcome_message}", &format!("Welcome to the Hnefatafl server, {}!", username))
                        .replace("{players_html}", &players_html)
                        .replace("{session_id}", &session_id);

                    return Ok::<_, warp::Rejection>(Response::builder().body(response).unwrap());
                }
            }

            // If no session exists, redirect to login page (you can return a 404 or redirect)
            Err(warp::reject::not_found())
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
        .map(move || {
            // Read the HTML template from file (assuming the file exists)
            let template_path = "templates/rules.html";
            let template = read_html_template(template_path).unwrap(); // We assume the file exists and unwrap the result

            // Return the template as a valid HTML response
            warp::reply::html(template)
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

            // Read the HTML template from file
            let template_path = "templates/game.html";
            let template = read_html_template(template_path).unwrap();

            // Replace placeholders in the template with dynamic content
            let response = template
                .replace("{board_message}", &format!("{}", &board_message))
                .replace("{board_html}", &board_html);

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
        .or(board_updates)
        .with(cors().allow_any_origin().allow_methods(vec![Method::GET, Method::POST]));


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


