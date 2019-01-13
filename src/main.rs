use ncurses::*;
use std::char;
use std::collections::HashMap;

use wizardscastle::game::Game;
use wizardscastle::player::{Stat, Race, Gender};

mod win;
mod names;

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
        let w = G::popup(7, 48);

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

    /// Choose gender
    fn choose_gender(&mut self) {
        let w = G::popup(7, 36);

        self.wcon(w, A_TITLE);
        G::mvwprintw_center(w, 2, "Which sex do you prefer?");
        self.wcoff(w, A_TITLE);

        G::mvwprintw_center(w, 4, "|[F]|emale  |[M]|ale");

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            let key = getch();

            match G::norm_key(key) {
                'F' => break self.game.player_set_gender(Gender::Female),
                'M' => break self.game.player_set_gender(Gender::Male),
                _ => (),
            }
        }

        G::popup_close(w);
    }

    /// Choose stats
    fn choose_stats(&mut self) {
        let w = G::popup(15, 50);


        let stats = [
            Stat::Strength,
            Stat::Intelligence,
            Stat::Dexterity,
        ];

        for stat in stats.iter() {
            let additional_points = self.game.player_additional_points();

            if additional_points == 0 {
                break;
            }

            wclear(w);

            self.wcon(w, A_TITLE);
            G::mvwprintw_center(w, 2, &format!("Ok, {}, you have these statistics:", self.player_race_name()));
            self.wcoff(w, A_TITLE);

            G::mvwprintw_center_notrim(w, 4, &format!("{:>12}: {:>2}", G::stat_name(Stat::Strength), self.game.player_stat(Stat::Strength)));
            G::mvwprintw_center_notrim(w, 5, &format!("{:>12}: {:>2}", G::stat_name(Stat::Intelligence), self.game.player_stat(Stat::Intelligence)));
            G::mvwprintw_center_notrim(w, 6, &format!("{:>12}: {:>2}", G::stat_name(Stat::Dexterity), self.game.player_stat(Stat::Dexterity)));

            G::mvwprintw_center(w, 8, &format!("And {} other points to allocate as you wish.", additional_points));

            G::mvwprintw_center(w, 10, &format!("How many points do you add to {}?", G::stat_name(*stat).to_uppercase()));

            G::mvwprintw_center(w, 12, &format!("Press |[0]| to |[{}]|", additional_points));

            box_(w, 0, 0);

            wrefresh(w);

            loop {
                let key = getch();

                let nkey = G::norm_key(key);

                match nkey {
                    '0'...'9' => {
                        let v = nkey.to_digit(10).unwrap();

                        if v > additional_points {
                            continue;
                        }

                        if let Err(err) = self.game.player_allocate_points(*stat, v) {
                            panic!(err);
                        }

                        break;
                    }
                    _ => (),
                }
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
            self.choose_gender();
            self.choose_stats();

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
