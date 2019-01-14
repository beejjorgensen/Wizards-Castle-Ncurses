use ncurses::*;
use std::char;
use std::collections::HashMap;

use wizardscastle::game::Game;

mod gen;
mod names;
mod win;

struct G {
    color: HashMap<&'static str, attr_t>,
    game: Game,
}

impl G {
    /// Build a new global game struct
    fn new() -> G {
        // Build out the known color schemes for color and non-color terminals
        let mut color = HashMap::new();

        if has_colors() {
            init_pair(1, COLOR_YELLOW, COLOR_BLACK);
            color.insert("bold-yellow", COLOR_PAIR(1) | A_BOLD());
        } else {
            color.insert("bold-yellow", A_BOLD());
        }

        // Return the new struct
        G {
            color,
            game: Game::new(8, 8, 8),
        }
    }

    #[allow(non_snake_case)]
    fn A_TITLE() -> &'static str {
        "bold-yellow"
    }

    /// Normalize an input character from getch(), making it uppercase
    fn norm_key(key: i32) -> char {
        let v: Vec<_> = char::from_u32(key as u32).unwrap().to_uppercase().collect();

        v[0]
    }

    /// Main game loop
    fn run(&mut self) {
        G::show_cursor(false);

        cbreak();
        noecho();

        self.intro();

        let mut playing = true;

        while playing {
            clear();
            refresh();

            self.choose_class();
            self.choose_gender();
            self.choose_stats();
            self.choose_armor();

            playing = false;
        }

        nocbreak();
        echo();
    }
}

/// Main
fn main() {
    initscr();

    if has_colors() {
        start_color();
    }

    refresh(); // If we don't do this first, windows don't show up

    let mut g = G::new();

    g.run();

    endwin();
}
