//mod crate::tablut;
use crate::tablut::{Cell as TablutCell, CellType as TablutCellType};

//mod crate::hnefatafl;
use crate::hnefatafl::{Cell as HnefataflCell, CellType as HnefataflCellType};

//mod crate::brandubh;
use crate::brandubh::{Cell as BrandubhCell, CellType as BrandubhCellType};

//mod crate::koch;
use crate::koch::{Cell as KochCell, CellType as KochCellType};

use rand::Rng;
use std::fs;


// Helper function to render the board as an HTML table
pub fn render_tablut_board_as_html(board: &Vec<Vec<TablutCell>>) -> String {
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
pub fn render_hnefatafl_board_as_html(board: &Vec<Vec<HnefataflCell>>) -> String {
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

pub fn render_brandubh_board_as_html(board: &Vec<Vec<BrandubhCell>>) -> String {
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

pub fn render_koch_board_as_html(board: &Vec<Vec<KochCell>>) -> String {
    let mut html = String::from("<table>");

    // Add rows with board cells and right-side coordinates
    for (row_idx, row) in board.iter().enumerate() {
        html.push_str("<tr>"); // Start a new row

        let mut col_idx = 0;
        for cell in row {
            // Determine the class and content based on the cell type
            let (class, content) = match cell.cell_type {
                KochCellType::Empty => ("empty", ""),
                KochCellType::Attacker => (
                    "attacker",
                    r#"<img src="/images/attacker.png" alt="Attacker" class="piece" />"#,
                ),
                KochCellType::Defender => (
                    "defender",
                    r#"<img src="/images/defender.png" alt="Defender" class="piece" />"#,
                ),
                KochCellType::King => (
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
pub fn get_session_id_from_cookie(headers: &warp::http::HeaderMap) -> Option<String> {
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
pub fn read_html_template(path: &str) -> Result<String, std::io::Error> {
    fs::read_to_string(path)
}

/// Helper function to generate a random ID of 8 digits
pub fn generate_random_id() -> usize {
    let mut rng = rand::thread_rng();
    let id: usize = rng.gen_range(10000000..100000000); // Generate a random number between 10000000 and 99999999
    id
}