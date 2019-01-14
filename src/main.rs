use ncurses::*;
use std::char;
use std::collections::HashMap;

use wizardscastle::game::Game;

mod gen;
mod names;
mod win;
mod map;

struct G {
    color: HashMap<&'static str, attr_t>,
    game: Game,
    mapwin: WINDOW,
}

impl G {
    /// Build a new global game struct
    fn new() -> G {
        // Build out the known color schemes for color and non-color terminals
        let mut color = HashMap::new();

        if has_colors() {
            init_pair(1, COLOR_YELLOW, COLOR_BLACK);
            init_pair(2, COLOR_RED, COLOR_BLACK);

            color.insert("bold-yellow", COLOR_PAIR(1) | A_BOLD());
            color.insert("bold-red", COLOR_PAIR(2) | A_BOLD());
            color.insert("red", COLOR_PAIR(2));
        } else {
            color.insert("bold-yellow", A_BOLD());
            color.insert("bold-red", 0);
            color.insert("red", 0);
        }

        // Return the new struct
        G {
            color,
            mapwin: newwin(17, 47, 0, 0),
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

        let playing = true;

        while playing {
            clear();
            refresh();

            self.choose_class();
            self.choose_gender();
            self.choose_stats();
            self.choose_armor();
            self.choose_weapon();
            self.choose_lamp();
            self.choose_flares();

            let alive = true;

            while alive {
                self.update_map(false);
            }
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
