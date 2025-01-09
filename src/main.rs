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
    // sse::{Event, reply},
    cors,
};

mod tablut;
use tablut::{GameState as TablutGameState, Cell as TablutCell, CellType as TablutCellType};

mod hnefatafl;
use hnefatafl::{GameState as HnefataflGameState, Cell as HnefataflCell, CellType as HnefataflCellType};

mod brandubh;
use brandubh::{GameState as BrandubhGameState, Cell as BrandubhCell, CellType as BrandubhCellType};
use rand::Rng;

#[derive(Clone, Copy, Debug)]
pub enum GameMode {
    Local,
    Online,
}

#[derive(Clone)]
pub struct AppState {
    pub games: Arc<RwLock<Vec<Option<GameVariant>>>>, // Use Option to mark ended games
    pub players: Arc<RwLock<HashMap<String, (String, String)>>>, // Maps session IDs to usernames
    pub player_game_map: Arc<RwLock<HashMap<String, usize>>>, // Maps session IDs to game IDs
}

#[derive(Clone, Debug)]
pub enum GameVariant {
    Tablut(TablutGameState, TablutGameState, GameMode),
    Hnefatafl(HnefataflGameState, HnefataflGameState, GameMode),
    Brandubh(BrandubhGameState, BrandubhGameState, GameMode),
}


#[derive(Deserialize)]
struct CellClick {
    row: usize,
    col: usize,
    session_id: String
}


#[derive(Debug)]
struct MissingUsername;

impl fmt::Display for MissingUsername {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Missing username")
    }
}

impl Reject for MissingUsername {}

#[derive(Deserialize)]
struct FormData {
    game_mode: String,
    side: String,
}

#[tokio::main]
async fn main() {
    // Static file serving for images
    let static_files = warp::path("images").and(warp::fs::dir("./static/images"));

    // Initialize application state
    let state = AppState {
        games: Arc::new(RwLock::new(Vec::new())),
        players: Arc::new(RwLock::new(HashMap::new())),
        player_game_map: Arc::new(RwLock::new(HashMap::new())),
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
                if let Some((username, _)) = players.get(&session_id) {
                    // Session already exists, don't ask for the username again
                    let players_html: String = players
                        .values()
                        .map(|(username, _)| format!("<p>{}</p>", username))
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
                state.players.write().await.insert(session_id.clone(), (username.clone(), "local".to_string()));

                // Set session_id in a cookie
                let cookie = format!("session_id={}; Path=/; HttpOnly;", session_id);

                // Now show the main page with the list of players
                let players = state.players.read().await; // Read the list of connected players

                // Build the players list in HTML
                let players_html: String = players
                    .values()
                    .map(|(username, _)| format!("<p>{}</p>", username))
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
                if let Some((username, _)) = players.get(&session_id) {
                    // Build the players list in HTML
                    let players_html: String = players
                        .values()
                        .map(|(username, _)| format!("<p>{}</p>", username))
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
            Ok::<_, warp::Rejection>(Response::builder().status(404).body("Not Found".to_string()).unwrap())
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

    let game_mode_local = warp::path("game_mode_local")
        .and(warp::post())
        .map(move || {
            // Read the HTML template from file (assuming the file exists)
            let template_path = "templates/game_mode_local.html";
            let template = read_html_template(template_path).unwrap(); // We assume the file exists and unwrap the result

            // Return the template as a valid HTML response
            html(template)
        });

    let game_mode_online = warp::path("game_mode_online")
        .and(warp::post())
        .map(move || {
            // Read the HTML template from file (assuming the file exists)
            let template_path = "templates/game_mode_online.html";
            let template = read_html_template(template_path).unwrap(); // We assume the file exists and unwrap the result

            // Return the template as a valid HTML response
            html(template)
        });

    // Redirect endpoint
    let redirect_endpoint = warp::path("redirect_endpoint")
        .and(warp::post())
        .and(state_filter.clone())
        .and(warp::body::form()) // Parse form data
        .and(warp::cookie::optional("session_id")) // Retrieve session_id from cookies
        .and_then(|state: AppState, form: FormData, session_id: Option<String>| async move {
            if let Some(session_id) = session_id {
                let mut players = state.players.write().await; // Acquire write lock

                if let Some(player_data) = players.get_mut(&session_id) {
                    // Update the role field
                    player_data.1 = form.side.clone();
                } else {
                    // Session ID not found in players map
                    let response = warp::http::Response::builder()
                        .status(400)
                        .body("Session ID not found".to_string())
                        .unwrap();
                    return Ok::<warp::http::Response<String>, warp::Rejection>(response);
                }
            } else {
                // No session ID provided
                let response = warp::http::Response::builder()
                    .status(400)
                    .body("Missing session ID".to_string())
                    .unwrap();
                return Ok::<warp::http::Response<String>, warp::Rejection>(response);
            }

            // Determine redirect URL
            let redirect_url = match (form.game_mode.as_str(), form.side.as_str()) {
                ("hnefatafl", "attacker") => "/hnefatafl_redirect_online",
                ("hnefatafl", "defender") => "/hnefatafl_redirect_online",
                ("tablut", "attacker") => "/tablut_redirect_online",
                ("tablut", "defender") => "/tablut_redirect_online",
                ("brandubh", "attacker") => "/brandubh_redirect_online",
                ("brandubh", "defender") => "/brandubh_redirect_online",
                _ => return Ok::<_, warp::Rejection>(
                    warp::http::Response::builder()
                        .status(400)
                        .body("Invalid game mode or role".into())
                        .unwrap(),
                ),
            };

            // Redirect response
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", redirect_url)
                .body("Redirecting...".into())
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    
    // Endpoint: Create a new hnefataflgame and redirect to it
    let hnefatafl_redirect_local = warp::path("hnefatafl_redirect_local")
        .and(warp::post())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Hnefatafl(HnefataflGameState::new(id), HnefataflGameState::new(id), GameMode::Local);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    let hnefatafl_redirect_online = warp::path("hnefatafl_redirect_online")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Hnefatafl(HnefataflGameState::new(id), HnefataflGameState::new(id), GameMode::Online);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    // Endpoint: Create a new tablut game and redirect to it
    let tablut_redirect_local = warp::path("tablut_redirect_local")
        .and(warp::post())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Tablut(TablutGameState::new(id), TablutGameState::new(id), GameMode::Local);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    let tablut_redirect_online = warp::path("tablut_redirect_online")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Tablut(TablutGameState::new(id), TablutGameState::new(id), GameMode::Online);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    // Endpoint: Create a new brandubh game and redirect to it
    let brandubh_redirect_local = warp::path("brandubh_redirect_local")
        .and(warp::post())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Brandubh(BrandubhGameState::new(id), BrandubhGameState::new(id), GameMode::Local);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });

    let brandubh_redirect_online = warp::path("brandubh_redirect_online")
        .and(warp::get())
        .and(state_filter.clone())
        .and_then(|state: AppState| async move {
            let mut games = state.games.write().await;
            let id = generate_random_id();
            let game = GameVariant::Brandubh(BrandubhGameState::new(id), BrandubhGameState::new(id), GameMode::Online);
            games.push(Some(game)); // Store the new game

            // Redirect to the new game page
            let response = warp::http::Response::builder()
                .status(302)
                .header("Location", format!("/game/{}", id))
                .body("Redirecting to new game...")
                .unwrap();

            Ok::<_, warp::Rejection>(response)
        });


    // Dictionary to store broadcast channels for each game
    let channels: Arc<RwLock<HashMap<usize, HashMap<String, broadcast::Sender<String>>>>> = Arc::new(RwLock::new(HashMap::new()));

    // Endpoint to create a new game and its broadcast channel
    let new_game = warp::path!("game" / usize)
        .and(warp::get())
        .and(state_filter.clone())
        .and({
            let channels = channels.clone();
            warp::any().map(move || channels.clone())
        })
        .and(warp::cookie::optional("session_id")) // Retrieve session_id from cookies
        .and_then(
            |id: usize,
            state: AppState,
            channels: Arc<RwLock<HashMap<usize, HashMap<String, broadcast::Sender<String>>>>>,
            session_id: Option<String>| async move {
                let games = state.games.write().await;
                let players = state.players.read().await;
                let mut mapping = state.player_game_map.write().await;

                let mut board_html = String::new();
                let mut board_message = String::new();
                let mut game_title = String::new();
                let mut players_html = String::new();
                let player_username = String::new();

                // Locate the game and populate its data
                let found_game = games.iter().any(|game_option| {
                    game_option.as_ref().map_or(false, |game_variant| match game_variant {
                        GameVariant::Tablut(game_at, _game_def, _) => {
                            if game_at.id == id {
                                board_html = render_tablut_board_as_html(&game_at.board);
                                board_message = game_at.board_message.clone();
                                game_title = game_at.game_title.clone();
                                mapping.insert(session_id.clone().unwrap(), id);
                                true
                            } else {
                                false
                            }
                        }
                        GameVariant::Hnefatafl(game_at, _game_def, _) => {
                            if game_at.id == id {
                                board_html = render_hnefatafl_board_as_html(&game_at.board);
                                board_message = game_at.board_message.clone();
                                game_title = game_at.game_title.clone();
                                mapping.insert(session_id.clone().unwrap(), id);
                                true
                            } else {
                                false
                            }
                        }
                        GameVariant::Brandubh(game_at, _game_def, _) => {
                            if game_at.id == id {
                                board_html = render_brandubh_board_as_html(&game_at.board);
                                board_message = game_at.board_message.clone();
                                game_title = game_at.game_title.clone();
                                mapping.insert(session_id.clone().unwrap(), id);
                                true
                            } else {
                                false
                            }
                        }
                    })
                });

                // If the game is not found, return an error
                if !found_game {
                    let error_response = warp::http::Response::builder()
                        .status(404) // Not Found
                        .body("Game not found.".into())
                        .unwrap();
                    return Ok::<_, warp::Rejection>(error_response);
                }

                let mut current_game_players: Vec<(String, String)> = Vec::new();

                for (session_id, game_id) in mapping.iter() {
                    if game_id == &id {
                        if let Some((username, role)) = players.get(session_id) {
                            current_game_players.push((username.clone(), role.clone()));
                        }
                    }
                }

                // Enforce player limit
                if current_game_players.len() > 2 {
                    return Ok::<_, warp::Rejection>(
                        warp::http::Response::builder()
                            .status(403)
                            .body("Game is already full. Maximum of two players allowed.".into())
                            .unwrap(),
                    );
                }

                // Update broadcast channels
                {
                    let mut channels = channels.write().await;
                    let game_channels = channels.entry(id).or_insert_with(HashMap::new);

                    for (player_session_id, _) in players.iter() {
                        game_channels.entry(player_session_id.clone()).or_insert_with(|| {
                            broadcast::channel::<String>(100).0
                        });
                    }
                }

                // Add all other players in the game
                players_html.push_str(&current_game_players
                    .iter()
                    .map(|(username, role)| format!("<p>{} ({})</p>", username, role))
                    .collect::<String>());

                let template_path = "templates/game.html";
                let template = read_html_template(template_path).unwrap();

                // Embed the session_id in a <script> tag in the response
                let session_script = if let Some(session_id) = session_id {
                    format!(r#"<script>const session_id = "{}";</script>"#, session_id)
                } else {
                    "<script>const session_id = null;</script>".to_string()
                };

                let response = template
                    .replace("{game_title}", &game_title)
                    .replace("{board_message}", &board_message)
                    .replace("{board_html}", &board_html)
                    .replace("{id}", &id.to_string())
                    .replace("{player_username}", &player_username)
                    .replace("{players_html}", &players_html)
                    .replace("</head>", &format!("{}\n</head>", session_script)); // Add session script to the head

                Ok::<_, warp::Rejection>(Response::builder().body(response).unwrap())
            },
        );


    // Endpoint: Join a game by ID
    let join_game_by_id = warp::path("join")
        .and(warp::post())
        .map( || {
            // Read the HTML template from file
            let template_path = "templates/join_game.html";
            let template = read_html_template(template_path);
            // If no matching game is found, return the join game page
            html(template.unwrap())
        });

    // Endpoint: Redirect to a game by ID
    let redirect_to_game = warp::path!("redirect" / usize)
        .and(state_filter.clone())
        .and(warp::header::headers_cloned()) 
        .and_then(|game_id: usize, state: AppState, headers: warp::http::HeaderMap| async move {
            let games = state.games.read().await;
            let mut players = state.players.write().await;
            let mapping = state.player_game_map.write().await;
            let player_id: String;
            if let Some(session_id) = get_session_id_from_cookie(&headers) {
                player_id = session_id;
                println!("Player ID: {}", player_id);
            } else {
                return Ok::<_, warp::Rejection>(
                    warp::http::Response::builder()
                        .status(400)
                        .body("Missing session ID".to_string())
                        .unwrap(),
                );
            }            

            // Check if there's a game with the given ID and if it's in online mode
            let game_exists_and_online = games.iter().any(|game_option| {
                game_option.as_ref().map_or(false, |game_variant| match game_variant {
                    GameVariant::Tablut(game_at, _game_def, GameMode::Online) => game_at.id == game_id,
                    GameVariant::Hnefatafl(game_at, _game_def, GameMode::Online) => game_at.id == game_id,
                    GameVariant::Brandubh(game_at, _game_def, GameMode::Online) => game_at.id == game_id,
                    _ => false,
                })
            });

            if game_exists_and_online {
                // Find the rival's role and assign opposite
                let rival_id = mapping.iter().find_map(|(key, &val)| {
                    if val == game_id && key != &player_id {
                        Some(key)
                    } else {
                        None
                    }
                }).unwrap();

                let rival_role = players.get(rival_id).unwrap().1.clone();
                let own_role: String;

                if rival_role == "attacker" {
                    own_role = "defender".to_string();
                } else {
                    own_role = "attacker".to_string();
                }

                // Update the player's role
                for (session_id, (_username, role)) in players.iter_mut() {
                    if session_id == &player_id {
                        // Update the second string
                        *role = own_role.clone();
                        let response = warp::http::Response::builder()
                            .status(302)
                            .header("Location", format!("/game/{}", game_id))
                            .body("Redirecting to game...".to_string())
                            .unwrap();
                        return Ok::<_, warp::Rejection>(response); // Exit after updating
                    }
                }

                // Redirect to the game
                let response = warp::http::Response::builder()
                    .status(302)
                    .header("Location", format!("/game/{}", game_id))
                    .body("Redirecting to game...".to_string())
                    .unwrap();
                Ok::<_, warp::Rejection>(response)
            } else {
                // Return error message if the game is not online or doesn't exist
                let error_response = warp::http::Response::builder()
                    .status(400) // Bad Request
                    .body("Cannot connect to game. Either the game does not exist or is not online.".to_string())
                    .unwrap();
                Ok::<_, warp::Rejection>(error_response)
            }
        });



    // Endpoint for board updates
    let board_updates = warp::path!("board-updates" / usize)
        .and(warp::get())
        .and({
            let channels = channels.clone();
            warp::any().map(move || channels.clone())
        })
        .and(warp::cookie::optional("session_id")) // Capture the session ID from cookies
        .and_then(
            |id: usize, channels: Arc<RwLock<HashMap<usize, HashMap<String, broadcast::Sender<String>>>>>, session_id: Option<String>| async move {
                if let Some(session_id) = session_id {
                    let channels = channels.read().await;

                    if let Some(game_channels) = channels.get(&id) {
                        if let Some(channel) = game_channels.get(&session_id) {
                            let rx = channel.subscribe();
                            return Ok::<_, warp::Rejection>(warp::sse::reply(warp::sse::keep_alive().stream(async_stream::stream! {
                                let mut rx = rx;
                                while let Ok(message) = rx.recv().await {
                                    yield Ok::<_, warp::Error>(warp::sse::Event::default().data(message));
                                }
                            })));
                        }
                    }
                    Err(warp::reject::not_found())
                } else {
                    Err(warp::reject::not_found())
                }
            },
        );


    // Endpoint to handle cell clicks
    let cell_click = warp::path!("cell-click" / usize)
    .and(warp::post())
    .and(warp::body::json())
    .and(state_filter.clone())
    .and({
        let channels = channels.clone();
        warp::any().map(move || channels.clone())
    })
    .and_then(
        |game_id: usize, click: CellClick, state: AppState, channels: Arc<RwLock<HashMap<usize, HashMap<String, broadcast::Sender<String>>>>>| async move {

            let players = state.players.read().await;

            let (_username, click_role) = match players.get(&click.session_id) {
                Some((username, click_role)) => (username, click_role),
                None => return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                    "success": false,
                    "error": "Session ID not found",
                }))),
            };

            let mut games = state.games.write().await;

            let move_made: bool;
            let current_turn: String;

            // Check if the game exists and process the click
            if let Some(game_option) = games.iter_mut().find(|game_option| {
                if let Some(game_variant) = game_option {
                    match game_variant {
                        GameVariant::Tablut(game_at, _game_def, _) => game_at.id == game_id,
                        GameVariant::Hnefatafl(game_at, _game_def, _) => game_at.id == game_id,
                        GameVariant::Brandubh(game_at, _game_def, _) => game_at.id == game_id,
                    }
                } else {
                    false
                }
            }) {
                if let Some(game_variant) = game_option {
                    let (board_unupdated, board_html, board_message, process_result, game_mode) = match game_variant {
                        GameVariant::Tablut(game_at, game_def, mode) => {

                            if game_at.current_turn.cell_type == TablutCellType::Defender {
                                current_turn = "defender".to_string();
                            } else {
                                current_turn = "attacker".to_string();
                            }

                            if click_role == &current_turn || click_role == "local" {
                                if click_role == "defender" {
                                    let board_unupdated = render_tablut_board_as_html(&game_def.board.clone());
                                    let process_result = game_def.process_click(click.row, click.col);
                                    let _unproccessed_result = game_at.process_click(click.row, click.col);
                                    let board_html = render_tablut_board_as_html(&game_def.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_def.board_message.clone(), process_result, mode.clone())
                                } else {
                                    let board_unupdated = render_tablut_board_as_html(&game_at.board.clone());
                                    let process_result = game_at.process_click(click.row, click.col);
                                    let _unproccessed_result = game_def.process_click(click.row, click.col);
                                    let board_html = render_tablut_board_as_html(&game_at.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_at.board_message.clone(), process_result, mode.clone())
                                }
                            } else {
                                return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                    "success": false,
                                    "error": "Not your turn",
                                })));
                            }                            
                        }
                        GameVariant::Hnefatafl(game_at, game_def, mode) => {
                            if game_at.current_turn.cell_type == HnefataflCellType::Defender {
                                current_turn = "defender".to_string();
                            } else {
                                current_turn = "attacker".to_string();
                            }

                            if click_role == &current_turn || click_role == "local"  {
                                if click_role == "defender" {
                                    let board_unupdated = render_hnefatafl_board_as_html(&game_def.board.clone());
                                    let process_result = game_def.process_click(click.row, click.col);
                                    let _unproccessed_result = game_at.process_click(click.row, click.col);
                                    let board_html = render_hnefatafl_board_as_html(&game_def.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_def.board_message.clone(), process_result, mode.clone())
                                } else {
                                    let board_unupdated = render_hnefatafl_board_as_html(&game_at.board.clone());
                                    let process_result = game_at.process_click(click.row, click.col);
                                    let _unproccessed_result = game_def.process_click(click.row, click.col);
                                    let board_html = render_hnefatafl_board_as_html(&game_at.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_at.board_message.clone(), process_result, mode.clone())
                                }
                            } else {
                                return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                    "success": false,
                                    "error": "Not your turn",
                                })));
                            }
                        }
                        GameVariant::Brandubh(game_at, game_def, mode) => {
                            if game_at.current_turn.cell_type == BrandubhCellType::Defender {
                                current_turn = "defender".to_string();
                            } else {
                                current_turn = "attacker".to_string();
                            }

                            if click_role == &current_turn || click_role == "local"  {
                                if click_role == "defender" {
                                    let board_unupdated = render_brandubh_board_as_html(&game_def.board.clone());
                                    let process_result = game_def.process_click(click.row, click.col);
                                    let _unproccessed_result = game_at.process_click(click.row, click.col);
                                    let board_html = render_brandubh_board_as_html(&game_def.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_def.board_message.clone(), process_result, mode.clone())
                                } else {
                                    let board_unupdated = render_brandubh_board_as_html(&game_at.board.clone());
                                    let process_result = game_at.process_click(click.row, click.col);
                                    let _unproccessed_result = game_def.process_click(click.row, click.col);
                                    let board_html = render_brandubh_board_as_html(&game_at.board);
                                    move_made = game_at.move_done;
                                    (board_unupdated, board_html, game_at.board_message.clone(), process_result, mode.clone())
                                }
                            } else {
                                return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                    "success": false,
                                    "error": "Not your turn",
                                })));
                            }
                        }
                    };


                    match process_result {
                        Ok(_) => {
                            // Check if session_id exists in players
                            let session_id = &click.session_id;
                                if let Some((username, _role)) = players.get(session_id) {
                                    // Prepare the update message
                                    let update = serde_json::to_string(&serde_json::json!({
                                        "board_html": board_html,
                                        "board_message": board_message,
                                        "username": username,
                                    }))
                                    .unwrap();
                                    
                                    let update_unupdated = serde_json::to_string(&serde_json::json!({
                                        "board_html": board_unupdated,
                                        "board_message": board_message,
                                        "username": username,
                                    }))
                                    .unwrap();                        

                                    // Access the channels map
                                    let channels = channels.read().await;

                                    if let Some(game_channels) = channels.get(&game_id) {
                                        match game_mode {
                                            GameMode::Local => {
                                                // Broadcast the update to all players in the game
                                                for channel in game_channels.values() {
                                                    let _ = channel.send(update.clone());
                                                }
                                            }
                                            GameMode::Online => {
                                                if move_made {
                                                    for channel in game_channels.values() {
                                                        let _ = channel.send(update.clone());
                                                    }
                                                } else {
                                                    for (sessions_id, channel) in game_channels.iter() {
                                                        if sessions_id == session_id {
                                                            let _ = channel.send(update.clone());
                                                        }
                                                        else {
                                                            let _ = channel.send(update_unupdated.clone());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                } else {
                                    // If the session_id isn't found in players, log it
                                    println!("Session ID not found in players: {:?}", session_id);
                                }

                            return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "success": true,
                                "board_html": board_html,
                                "board_message": board_message,
                            })));
                        }
                        Err(error_message) => {
                            return Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                                "success": false,
                                "error": error_message,
                                "board_html": board_html,
                                "board_message": board_message,
                            })));
                        }
                    }
                }
            }

            // If no game could process the click
            Ok::<_, warp::Rejection>(warp::reply::json(&serde_json::json!({
                "success": false,
                "error": "Game not found or inactive",
            })))
        },
    );


    // Combine all routes
    let routes = static_files
        .or(username_form)
        .or(main_page_get)
        .or(main_page_post)
        .or(sign_out_post)
        .or(rules)
        .or(new_game)
        .or(cell_click)
        .or(board_updates)
        .or(join_game_by_id)
        .or(redirect_to_game)
        .or(redirect_endpoint)
        .or(hnefatafl_redirect_local)
        .or(hnefatafl_redirect_online)
        .or(tablut_redirect_local)
        .or(tablut_redirect_online)
        .or(brandubh_redirect_local)
        .or(brandubh_redirect_online)
        .or(game_mode_local)
        .or(game_mode_online)
        .with(cors().allow_any_origin().allow_methods(vec![Method::GET, Method::POST]));


    // Start the server
    warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
}


/// Helper function to render the board as an HTML table
fn render_tablut_board_as_html(board: &Vec<Vec<TablutCell>>) -> String {
    let mut html = String::from("<table>");

    // Add rows with board cells and right-side coordinates
    for (row_idx, row) in board.iter().enumerate() {
        html.push_str("<tr>"); // Start a new row

        let mut col_idx = 0;
        for cell in row {
            // Determine the class and content based on the cell type
            let (class, content) = match cell.cell_type {
                TablutCellType::Empty => ("empty", ""),
                TablutCellType::Attacker => (
                    "attacker",
                    r#"<img src="/images/attacker.png" alt="Attacker" class="piece" />"#,
                ),
                TablutCellType::Defender => (
                    "defender",
                    r#"<img src="/images/defender.png" alt="Defender" class="piece" />"#,
                ),
                TablutCellType::King => (
                    "king",
                    r#"<img src="/images/queen.png" alt="King" class="piece" />"#,
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

/// Helper function to render the board as an HTML table
fn render_hnefatafl_board_as_html(board: &Vec<Vec<HnefataflCell>>) -> String {
    let mut html = String::from("<table>");

    // Add rows with board cells and right-side coordinates
    for (row_idx, row) in board.iter().enumerate() {
        html.push_str("<tr>"); // Start a new row

        let mut col_idx = 0;
        for cell in row {
            // Determine the class and content based on the cell type
            let (class, content) = match cell.cell_type {
                HnefataflCellType::Empty => ("empty", ""),
                HnefataflCellType::Attacker => (
                    "attacker",
                    r#"<img src="/images/attacker.png" alt="Attacker" class="piece" />"#,
                ),
                HnefataflCellType::Defender => (
                    "defender",
                    r#"<img src="/images/defender.png" alt="Defender" class="piece" />"#,
                ),
                HnefataflCellType::King => (
                    "king",
                    r#"<img src="/images/queen.png" alt="King" class="piece" />"#,
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

fn render_brandubh_board_as_html(board: &Vec<Vec<BrandubhCell>>) -> String {
    let mut html = String::from("<table>");

    // Add rows with board cells and right-side coordinates
    for (row_idx, row) in board.iter().enumerate() {
        html.push_str("<tr>"); // Start a new row

        let mut col_idx = 0;
        for cell in row {
            // Determine the class and content based on the cell type
            let (class, content) = match cell.cell_type {
                BrandubhCellType::Empty => ("empty", ""),
                BrandubhCellType::Attacker => (
                    "attacker",
                    r#"<img src="/images/attacker.png" alt="Attacker" class="piece" />"#,
                ),
                BrandubhCellType::Defender => (
                    "defender",
                    r#"<img src="/images/defender.png" alt="Defender" class="piece" />"#,
                ),
                BrandubhCellType::King => (
                    "king",
                    r#"<img src="/images/queen.png" alt="King" class="piece" />"#,
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

// Helper to get the session ID from the cookie
fn get_session_id_from_cookie(headers: &warp::http::HeaderMap) -> Option<String> {
    headers
        .get("cookie")
        .and_then(|cookie| cookie.to_str().ok())
        .and_then(|cookie_str| {
            cookie_str
                .split(';')
                .find_map(|cookie| {
                    let cookie = cookie.trim();
                    if cookie.starts_with("session_id=") {
                        // Safe extraction after the "=" symbol
                        Some(cookie["session_id=".len()..].to_string()) // Extract session ID
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

/// Helper function to generate a random ID of 8 digits
fn generate_random_id() -> usize {
    let mut rng = rand::thread_rng();
    let id: usize = rng.gen_range(10000000..100000000); // Generate a random number between 10000000 and 99999999
    id
}


