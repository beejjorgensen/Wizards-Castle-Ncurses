use ncurses::*;
use std::char;
use std::collections::HashMap;

use wizardscastle::error::Error;
use wizardscastle::game::{Direction, DrinkEvent, Event, Game};
use wizardscastle::room::RoomType;

mod gen;
mod log;
mod map;
mod names;
mod stat;
mod win;

use crate::stat::StatMode;

struct G {
    color: HashMap<&'static str, attr_t>,
    game: Game,
    mapwin: WINDOW,
    statwin: WINDOW,
    logwin: WINDOW,
    loginner: WINDOW,
    statmode: StatMode,
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

        let logwin = newwin(8, 80, 17, 0);
        let loginner = derwin(logwin, 6, 78, 1, 1);

        scrollok(loginner, true);

        // Return the new struct
        G {
            color,
            mapwin: newwin(17, 47, 0, 0),
            statwin: newwin(17, 32, 0, 48),
            logwin,
            loginner,
            game: Game::new(8, 8, 8),
            statmode: StatMode::None,
        }
    }

    #[allow(non_snake_case)]
    fn A_TITLE() -> &'static str {
        "bold-yellow"
    }

    /// Normalize an input character from getch(), making it uppercase
    fn norm_key(key: i32) -> char {
        let mut ch = ' ';

        if G::is_arrow_key(key) {
            match key {
                KEY_UP => ch = 'n',
                KEY_DOWN => ch = 's',
                KEY_LEFT => ch = 'w',
                KEY_RIGHT => ch = 'e',
                _ => (),
            }
        } else {
            ch = char::from_u32(key as u32).unwrap()
        }

        let v: Vec<_> = ch.to_uppercase().collect();

        v[0]
    }

    /// Move a direction
    fn move_dir(&mut self, dir: Direction) {
        self.game.move_dir(dir);

        // This is often redundant, but there's a case where we retreat from
        // monsters and the discover room gets overlooked
        self.game.discover_room_at_player();
    }

    /// Tell if a key was an arrow key
    pub fn is_arrow_key(k: i32) -> bool {
        k == KEY_UP || k == KEY_DOWN || k == KEY_LEFT || k == KEY_RIGHT
    }

    /// Print a location event message, hiding the location if the player is
    /// blind
    fn make_loc_event_msg(
        &self,
        prefix: &str,
        suffix: &str,
        blind_suffix: &str,
        x: u32,
        y: u32,
        z: u32,
    ) -> String {
        if self.game.player_is_blind() {
            format!("{}{}", prefix, blind_suffix).to_string()
        } else {
            format!(
                "{} at ({},{}) level {}{}",
                prefix,
                x + 1,
                y + 1,
                z + 1,
                suffix
            )
            .to_string()
        }
    }

    /// Set off a flare
    fn flare(&mut self) {
        if let Err(err) = self.game.flare() {
            match err {
                Error::CantGo => self.update_log_error("** Hey bright one, you're out of flares"),
                Error::Blind => self.update_log_error(&format!(
                    "** You can see anything, dumb {}",
                    self.race_name()
                )),
                any => panic!("{:#?}", any),
            }
        }
    }

    /// Shine the lamp
    fn lamp(&mut self) {
        if !self.game.player_has_lamp() {
            self.update_log_error("** You don't have a lamp");
        }

        if self.game.player_is_blind() {
            self.update_log_error(&format!(
                "** You can see anything, dumb {}",
                self.race_name()
            ));
        }

        self.set_statmode(StatMode::Lamp);

        let key = getch();

        let dir;

        match G::norm_key(key) {
            'N' => dir = Some(Direction::North),
            'S' => dir = Some(Direction::South),
            'W' => dir = Some(Direction::West),
            'E' => dir = Some(Direction::East),
            _ => dir = None,
        }

        if let Some(d) = dir {
            match self.game.shine_lamp(d) {
                Ok((x, y, z, room_type)) => {
                    self.update_log(&format!(
                        "The lamp shines into ({},{}) level {}. There you'll find {}.",
                        x + 1,
                        y + 1,
                        z + 1,
                        G::room_name(&room_type)
                    ));
                }
                Err(err) => panic!(err),
            }
        }

        self.set_statmode(StatMode::None);
    }

    // Drink from a pool
    fn drink(&mut self) {
        let s;

        match self.game.drink() {
            Ok(DrinkEvent::Stronger) => {
                s = String::from("feel stronger!");
            }
            Ok(DrinkEvent::Weaker) => {
                s = String::from("feel weaker.");
            }
            Ok(DrinkEvent::Smarter) => {
                s = String::from("feel smarter!");
            }
            Ok(DrinkEvent::Dumber) => {
                s = String::from("feel dumber.");
            }
            Ok(DrinkEvent::Nimbler) => {
                s = String::from("feel nimbler!");
            }
            Ok(DrinkEvent::Clumsier) => {
                s = String::from("feel clumsier.");
            }
            Ok(DrinkEvent::ChangeRace) => {
                s = format!("turn into a {}!", self.race_name());
            }
            Ok(DrinkEvent::ChangeGender) => {
                s = format!(
                    "turn into a {} {}!",
                    G::gender_name(*self.game.player_gender()),
                    self.race_name()
                );
            }
            Err(err) => panic!("{:#?}", err),
        }

        self.update_log(&format!("You take a drink and {}", s));
    }

    /// Drink or Down
    fn drink_down(&mut self) {
        match self.game.room_at_player().room_type() {
            RoomType::StairsDown => (),
            RoomType::Pool => self.drink(),
            _ => {
                self.update_log_error(&format!(
                    "** Oh {}, no stairs going down in here.",
                    self.race_name()
                ));
                self.update_log_error("** If you want a drink, find a pool.");
            }
        }
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
            self.choose_weapon();
            self.choose_lamp();
            self.choose_flares();

            let mut automove = false;

            let mut alive = true;

            self.update_log_attr(
                "You enter the castle and begin!\n",
                self.wcget("bold-yellow"),
            );

            while alive {
                self.update_map(false);
                self.update_stat();

                if !automove {
                    let key = getch();

                    match G::norm_key(key) {
                        'N' => self.move_dir(Direction::North),
                        'S' => self.move_dir(Direction::South),
                        'W' => self.move_dir(Direction::West),
                        'E' => self.move_dir(Direction::East),
                        'D' => self.drink_down(),
                        'Q' => {
                            // TODO are you sure?
                            alive = false;
                            // TODO play again?
                            playing = false;
                        }
                        'F' => self.flare(),
                        'L' => self.lamp(),
                        _ => (),
                    }
                } else {
                    automove = false;
                }

                let ox = self.game.player_x();
                let oy = self.game.player_y();
                let oz = self.game.player_z();

                match self.game.room_effect() {
                    Event::FoundGold(_) => {
                        self.update_log(&format!(
                            "You found gold! You now have {} GPs.",
                            self.game.player_gp()
                        ));
                    }
                    Event::FoundFlares(_) => {
                        self.update_log(&format!(
                            "You found flares! You now have {}.",
                            self.game.player_flares()
                        ));
                    }
                    Event::Sinkhole => {
                        let msg = self.make_loc_event_msg(
                            "You fell into a sinkhole",
                            "!",
                            "!",
                            ox,
                            oy,
                            oz,
                        );
                        self.update_log(&msg);
                        self.game.discover_room_at_player();
                        automove = true;
                    }
                    Event::Warp => {
                        let msg =
                            self.make_loc_event_msg("You entered a warp", "!", "!", ox, oy, oz);
                        self.update_log(&msg);
                        self.game.discover_room_at_player();
                        automove = true;
                    }
                    Event::Treasure(t) => {
                        let msg = &format!(
                            "Here you find the {}! It's now yours!",
                            G::treasure_name(*t.treasure_type())
                        );
                        self.update_log_good(&msg);
                    }
                    _ => (),
                }
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

    keypad(stdscr(), true);

    refresh(); // If we don't do this first, windows don't show up

    let mut g = G::new();

    g.run();

    endwin();
}
