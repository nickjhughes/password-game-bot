use lazy_static::lazy_static;

/// A chess puzzle.
#[derive(Debug, Clone)]
pub struct ChessPuzzle {
    /// Board in Forsyth-Edwards Notation (FEN).
    pub fen: String,
    /// The correct optimal move in Standard Algebraic Notation (SAN).
    pub solution: String,
}

/// A GeoGuessr-like game.
#[derive(Debug, Clone)]
pub struct GeoGame {
    /// The coordinates (lat, long) of the start location.
    pub coordindates: (f64, f64),
    /// The solution country.
    pub country: String,
}

lazy_static! {
    pub static ref CAPTCHAS: Vec<&'static str> = {
        let mut v = Vec::new();
        let captchas_raw = include_str!("data/captchas.txt");
        for line in captchas_raw.lines().filter(|l| !l.is_empty()) {
            v.push(line);
        }
        v
    };
    pub static ref GEO_GAMES: Vec<GeoGame> = {
        let mut v = Vec::new();
        let coordinates_raw = include_str!("data/coordinates.txt");
        let countries_raw = include_str!("data/countries.txt");
        for (coordinates, country) in coordinates_raw
            .lines()
            .filter(|l| !l.is_empty())
            .zip(countries_raw.lines().filter(|l| !l.is_empty()))
        {
            let mut parts = coordinates.split(',');
            let lat = parts.next().unwrap().parse::<f64>().unwrap();
            let long = parts.next().unwrap().parse::<f64>().unwrap();

            v.push(GeoGame {
                coordindates: (lat, long),
                country: country.to_string(),
            });
        }
        v
    };
    pub static ref CHESS_PUZZLES: Vec<ChessPuzzle> = {
        let mut v = Vec::new();
        let puzzles_raw = include_str!("data/chess_puzzles.txt");
        let moves_raw = include_str!("data/chess_moves.txt");
        for (fen, solution) in puzzles_raw
            .lines()
            .filter(|l| !l.is_empty())
            .zip(moves_raw.lines().filter(|l| !l.is_empty()))
        {
            v.push(ChessPuzzle {
                fen: fen.to_string(),
                solution: solution.to_string(),
            });
        }
        v
    };
}

#[cfg(test)]
mod tests {
    use ordered_float::NotNan;

    #[test]
    fn load_captchas() {
        use super::CAPTCHAS;

        assert_eq!(CAPTCHAS.len(), 149);
        assert!(CAPTCHAS.iter().all(|c| c.len() == 5));
    }

    #[test]
    #[ignore]
    fn load_geo_games() {
        use super::GEO_GAMES;
        use crate::game::helpers::get_country_from_coordinates;

        assert_eq!(GEO_GAMES.len(), 63);

        for geo_game in GEO_GAMES.iter() {
            let country = get_country_from_coordinates(
                NotNan::new(geo_game.coordindates.0).unwrap(),
                NotNan::new(geo_game.coordindates.1).unwrap(),
            );
            assert_eq!(country, geo_game.country.to_ascii_lowercase());
        }
    }

    #[test]
    #[ignore]
    fn load_chess_puzzles() {
        use super::CHESS_PUZZLES;
        use crate::game::helpers::get_optimal_move;

        assert_eq!(CHESS_PUZZLES.len(), 193);

        for puzzle in CHESS_PUZZLES.iter() {
            let solution_move = get_optimal_move(puzzle.fen.clone());
            assert_eq!(solution_move, puzzle.solution);
        }
    }
}
