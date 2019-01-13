use std::collections::HashMap;

use wizardscastle::game::Game;
use ncurses::*;

mod win;

struct G {
    color: HashMap<&'static str, attr_t>,
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
        }
    }

    /// Do the intro
    fn intro(&self) {
        let w = G::newwin_popup(15, 64);

        self.wcon(w, "bold-yellow");
        G::mvwprintw_center(w, 2,  "* * * THE WIZARD'S CASTLE * * *\n\n");
        self.wcoff(w, "bold-yellow");

        wattron(w, A_BOLD());
        G::wprintw_center(w, "Many cycles ago, in the Kingdom of N'DIC, the gnomic\n");
        G::wprintw_center(w, "wizard ZOT forged his great *ORB OF POWER*. He soon\n");
        G::wprintw_center(w, "vanished, leaving behind his vast subterranean castle\n");
        G::wprintw_center(w, "filled with esurient monsters, fabulous treasures, and\n");
        G::wprintw_center(w, "the incredible *ORB OF ZOT*. From that time hence, many\n");
        G::wprintw_center(w, "a bold youth has ventured into the wizard's castle. As\n");
        G::wprintw_center(w, "of now, *NONE* has ever emerged victoriously! BEWARE!!\n\n");
        wattroff(w, A_BOLD());

        wattron(w, A_REVERSE());
        G::wprintw_center(w, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);

        wrefresh(w);

        getch();
    }

    /// Main game loop
    fn run(&mut self) {
        let _game = Game::new(8, 8, 8);

        G::show_cursor(false);

        self.intro();
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
