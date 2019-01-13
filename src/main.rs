use ncurses::*;
use std::char;
use std::collections::HashMap;

use wizardscastle::game::Game;
use wizardscastle::player::Race;

mod win;

struct G {
    color: HashMap<&'static str, attr_t>,
    game: Game,
}

const A_TITLE: &str = "bold-yellow";

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

    /// Do the intro
    fn intro(&self) {
        let w = G::popup(15, 64);

        self.wcon(w, A_TITLE);
        G::mvwprintw_center(w, 2, "* * * THE WIZARD'S CASTLE * * *\n\n");
        self.wcoff(w, A_TITLE);

        wattron(w, A_BOLD());
        G::wprintw_center(w, "Many cycles ago, in the Kingdom of N'DIC, the gnomic\n");
        G::wprintw_center(w, "wizard ZOT forged his great *ORB OF POWER*. He soon\n");
        G::wprintw_center(w, "vanished, leaving behind his vast subterranean castle\n");
        G::wprintw_center(
            w,
            "filled with esurient monsters, fabulous treasures, and\n",
        );
        G::wprintw_center(
            w,
            "the incredible *ORB OF ZOT*. From that time hence, many\n",
        );
        G::wprintw_center(
            w,
            "a bold youth has ventured into the wizard's castle. As\n",
        );
        G::wprintw_center(
            w,
            "of now, *NONE* has ever emerged victoriously! BEWARE!!\n\n",
        );
        wattroff(w, A_BOLD());

        wattron(w, A_REVERSE());
        G::wprintw_center(w, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);

        wrefresh(w);

        getch();

        G::popup_close(w);
    }

    /// Choose class
    fn choose_class(&mut self) {
        let w = G::popup(7, 45);

        self.wcon(w, A_TITLE);
        G::mvwprintw_center(w, 2, "All right, Bold One. You may be an:");
        self.wcoff(w, A_TITLE);

        G::mvwprintw_center(w, 4, "|[E]|lf  |[D]|warf  Hu|[m]|an  |[H]|obbit");

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            let key = getch();

            match G::norm_key(key) {
                'E' => break self.game.player_init(Race::Elf),
                'D' => break self.game.player_init(Race::Dwarf),
                'M' => break self.game.player_init(Race::Human),
                'H' => break self.game.player_init(Race::Hobbit),
                _ => (),
            }
        }

        G::popup_close(w);
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
            //self.choose_gender();

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
