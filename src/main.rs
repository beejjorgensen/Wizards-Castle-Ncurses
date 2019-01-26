use ncurses::*;
use std::char;
use std::collections::HashMap;
use std::env;

use rand::rngs::ThreadRng;
use rand::thread_rng;
use rand::Rng;

use wizardscastle::error::Error;
use wizardscastle::game::{
    BookEvent, ChestEvent, CombatEvent, Direction, DrinkEvent, Event, Game, GameState, HitResult,
    OrbEvent, Stairs,
};
use wizardscastle::monster::MonsterType;
use wizardscastle::room::RoomType;

mod gen;
mod help;
mod inv;
mod log;
mod map;
mod names;
mod quit;
mod stat;
mod vendor;
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

    rng: ThreadRng,

    currently_fighting: Option<MonsterType>,

    discover_all: bool, // map cheat
}

impl G {
    /// Build a new global game struct
    fn new(discover_all: bool) -> G {
        // Build out the known color schemes for color and non-color terminals
        let mut color = HashMap::new();

        if has_colors() {
            init_pair(1, COLOR_YELLOW, COLOR_BLACK);
            init_pair(2, COLOR_RED, COLOR_BLACK);
            init_pair(3, COLOR_GREEN, COLOR_BLACK);

            color.insert("bold-yellow", COLOR_PAIR(1) | A_BOLD());
            color.insert("bold-red", COLOR_PAIR(2) | A_BOLD());
            color.insert("red", COLOR_PAIR(2));
            color.insert("dim-yellow", COLOR_PAIR(1) | A_DIM());
            color.insert("dim-green", COLOR_PAIR(3) | A_DIM());
            color.insert("dim-red", COLOR_PAIR(2) | A_DIM());
        } else {
            color.insert("bold-yellow", A_BOLD());
            color.insert("bold-red", 0);
            color.insert("red", 0);
            color.insert("dim-yellow", A_DIM());
            color.insert("dim-green", A_DIM());
            color.insert("dim-red", A_DIM());
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

            rng: thread_rng(),

            currently_fighting: None,

            discover_all,
        }
    }

    #[allow(non_snake_case)]
    fn A_TITLE() -> &'static str {
        "bold-yellow"
    }

    #[allow(non_snake_case)]
    fn A_WARN_TITLE() -> &'static str {
        "bold-red"
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
        // TODO check for retreating
        self.currently_fighting = None;

        // Ask if player is sure they want to leave
        let roomtype = self.game.room_at_player().roomtype.clone();

        if roomtype == RoomType::Entrance
            && dir == Direction::North
            && !self.game.player_has_orb_of_zot()
            && !self.verify_quit(true)
        {
            return;
        }

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

    /// Print a no-stairs error
    fn print_no_stairs_error(&self, s: Stairs) {
        self.update_log_error(&format!(
            "** Oh {}, no stairs going {} in here.",
            self.race_name(),
            G::stair_name(s)
        ));
    }

    /// Take some stairs
    fn move_stairs(&mut self, stairs: Stairs) {
        if self.game.move_stairs(stairs).is_err() {
            self.print_no_stairs_error(stairs);
        }
    }

    /// Drink or Down
    fn drink_down(&mut self) {
        match self.game.room_at_player().room_type() {
            RoomType::StairsDown => self.move_stairs(Stairs::Down),
            RoomType::Pool => self.drink(),
            _ => {
                self.print_no_stairs_error(Stairs::Down);
                self.update_log_error("** If you want a drink, find a pool.");
            }
        }
    }

    /// Gaze into an Orb
    pub fn gaze(&mut self) {
        let s;
        let mut error = false;
        let mut logf: fn(&G, &str) = G::update_log;

        match self.game.gaze() {
            Ok(event) => match event {
                OrbEvent::BloodyHeap => {
                    s = String::from("yourself in a bloody heap!");
                    logf = G::update_log_bad;
                }
                OrbEvent::Polymorph(m) => {
                    let mon_str = G::monster_name(m);
                    s = format!(
                        "yourself drinking from a pool and becoming {} {}.",
                        G::get_article(&mon_str),
                        mon_str
                    );
                }
                OrbEvent::GazeBack(m) => {
                    let mon_str = G::monster_name(m);
                    s = format!(
                        "{} {} gazing back at you.",
                        G::get_article(&mon_str),
                        mon_str
                    );
                }
                OrbEvent::Item(room_type, x, y, z) => {
                    s = format!(
                        "{} at ({},{}) level {}",
                        G::room_name(&room_type),
                        x + 1,
                        y + 1,
                        z + 1
                    );
                }
                OrbEvent::OrbOfZot(x, y, z) => {
                    s = format!("THE ORB OF ZOT at ({},{}) level {}!", x + 1, y + 1, z + 1);
                    logf = G::update_log_good;
                }
                OrbEvent::SoapOpera => {
                    s = String::from("a soap opera rerun.");
                }
            },
            Err(Error::Blind) => {
                s = format!("** You can't see anything, dumb {}", self.race_name());
                error = true;
            }
            Err(Error::CantGo) => {
                s = String::from("** No orb--no gaze!");
                error = true;
            }
            _ => panic!("SNH"),
        }

        if error {
            self.update_log_error(&s);
        } else {
            logf(&self, &format!("You see {}", s));
        }
    }

    /// Open a chest
    fn open_chest(&mut self) {
        match self.game.open_chest() {
            Ok(event) => match event {
                ChestEvent::Explode => self.update_log_bad("KABOOM! It explodes!"),
                ChestEvent::Gas => self.update_log_bad("Gas! You stagger from the room."),
                ChestEvent::Treasure(amount) => {
                    self.update_log(&format!("You find {} gold pieces!", amount))
                }
            },

            Err(err) => panic!(err),
        }

        println!();
    }

    /// Open a book
    fn open_book(&mut self) {
        match self.game.open_book() {
            Ok(event) => match event {
                BookEvent::Blind => self.update_log_bad(&format!(
                    "FLASH! Oh no! You are now a blind {}!",
                    self.race_name()
                )),
                BookEvent::Poetry => {
                    self.update_log("it's another volume of Zot's poetry! - Yeech!")
                }
                BookEvent::PlayMonster(m) => {
                    self.update_log(&format!("It's an old copy of play{}.", G::monster_name(m)))
                }
                BookEvent::Dexterity => self.update_log_good("It's a manual of dexterity!"),
                BookEvent::Strength => self.update_log_good("It's a manual of strength!"),
                BookEvent::Sticky => self.update_log_bad(
                    "The book sticks to your hands--now you can't draw your weapon!",
                ),
            },
            Err(err) => panic!(err),
        }
    }

    /// Open a book or chest
    fn open(&mut self) {
        let room_type = self.game.room_at_player().room_type().clone();

        match room_type {
            RoomType::Chest => self.open_chest(),
            RoomType::Book => self.open_book(),
            _ => {
                self.update_log_error("** The only thing you opened was your big mouth!");
            }
        }
    }

    /// Things to do at the start of the turn
    fn at_turn_start(&mut self) {
        self.game.add_turn(1);

        self.game.curse_effects();

        if self.game.curse_check() {
            self.update_log_bad("You feel a chill in your bones.");
        }

        //self.rand_message();

        // Cure blindness
        if self.game.cure_blindness() {
            self.update_log_good("The Opal Eye cures your blindness!");
        }

        // Cure book stuck to hands
        if self.game.cure_book() {
            self.update_log_good("The Blue Flame dissolves the book!");
        }
    }

    /// Set the status window display based on the room
    fn set_statmode_display(&mut self) {
        match self.game.room_at_player().room_type() {
            RoomType::Pool => self.set_statmode(StatMode::Pool),
            RoomType::Book => self.set_statmode(StatMode::Book),
            RoomType::Chest => self.set_statmode(StatMode::Chest),
            RoomType::StairsUp => self.set_statmode(StatMode::StairsUp),
            RoomType::StairsDown => self.set_statmode(StatMode::StairsDown),
            RoomType::CrystalOrb => self.set_statmode(StatMode::CrystalOrb),
            _ => self.set_statmode(StatMode::None),
        }
    }

    fn trade(&mut self) {
        let find_vendor = || {
            self.update_log_error("** If you want to trade, find a vendor.");
        };

        match self.game.room_at_player().room_type() {
            RoomType::Monster(m) => {
                if m.monster_type() == MonsterType::Vendor {
                    if self.game.vendors_angry() {
                        self.update_log_error("** The vendor's in no mood to trade!");
                    } else {
                        self.vendor_trade();
                    }
                } else {
                    find_vendor();
                }
            }
            _ => find_vendor(),
        }
    }

    /// Print messaging when monster defeated by melee or magic
    fn monster_defeated_message(&mut self, result: HitResult, mon_art: &str, m_str: &str) {
        if result.defeated {
            self.update_log_good(&format!(
                "{} {} lies dead at your feet!",
                G::initial_upper(mon_art),
                m_str
            ));

            if self.game.rand_recipe() {
                let suffix = [
                    "wich", " stew", " soup", " burger", " roast", " munchy", " taco", " pie",
                ];

                let i = self.rng.gen_range(0, suffix.len());

                self.update_log(&format!("You spend an hour eating {}{}", m_str, suffix[i]));
            }

            if result.killed_vendor {
                self.update_log_good(
                    "You get all his wares: plate armor, a sword, a strength potion,",
                );

                let mut s = String::from("an intelligence potion, ");

                if result.got_lamp {
                    s.push_str("a dexterity potion, and a lamp.");
                } else {
                    s.push_str("and a dexterity potion.");
                }

                self.update_log_good(&s);
            } else {
                if result.got_runestaff {
                    self.update_log_good("** GREAT ZOT! YOU'VE FOUND THE RUNESTAFF! **");
                }

                self.update_log(&format!("You now get his hoard of {} GPs", result.treasure));
            }
        }
    }

    /// Attack a thing
    fn attack(&mut self) {
        let weapon_type = self.game.player_weapon_type();

        match self.game.state() {
            GameState::Vendor => {
                self.update_log_bad("** You'll be sorry you did that!");
                self.game.vendor_attack();
            }

            GameState::PlayerAttack => {
                let monster_type = self.currently_fighting.unwrap();
                let mon_str = G::monster_name(monster_type);
                let mon_art = G::get_article(&mon_str);

                match self.game.attack() {
                    Ok(CombatEvent::NoWeapon) => {
                        self.update_log_bad(&format!(
                            "** Pounding on {} {} won't hurt it!",
                            mon_art, mon_str
                        ));
                    }

                    Ok(CombatEvent::BookHands) => {
                        self.update_log_bad("** You can't beat it to death with a book!");
                    }

                    Ok(CombatEvent::Hit(result)) => {
                        self.update_log_good(&format!("You hit the lousy {}!", mon_str));

                        if result.broke_weapon {
                            self.update_log_bad(&format!(
                                "Oh no! Your {} broke!",
                                G::weapon_name(weapon_type)
                            ));
                        }

                        self.monster_defeated_message(result, &mon_art, &mon_str);
                    }

                    Ok(CombatEvent::Miss) => {
                        println!("\n  DRAT! MISSED");
                    }

                    Ok(any) => panic!("unexpected combat event {:#?}", any),

                    Err(err) => panic!("error in combat {:#?}", err),
                }
            }
            _ => self.update_log_error("** There's nothing to attack here!"),
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
                self.at_turn_start();

                self.update_map(self.discover_all);
                self.update_stat();

                if !automove {
                    let key = getch();

                    match G::norm_key(key) {
                        'A' => self.attack(),
                        'N' => self.move_dir(Direction::North),
                        'S' => self.move_dir(Direction::South),
                        'W' => self.move_dir(Direction::West),
                        'E' => self.move_dir(Direction::East),
                        'D' => self.drink_down(),
                        'U' => self.move_stairs(Stairs::Up),
                        'F' => self.flare(),
                        'L' => self.lamp(),
                        'G' => self.gaze(),
                        'O' => self.open(),
                        'I' => self.show_inventory(),
                        'T' => self.trade(),
                        'H' | '?' => self.help(),
                        'Q' => {
                            if self.verify_quit(false) {
                                alive = false;
                                // TODO play again?
                                playing = false;
                            }
                        }
                        _ => (),
                    }
                } else {
                    automove = false;
                }

                let ox = self.game.player_x();
                let oy = self.game.player_y();
                let oz = self.game.player_z();

                self.set_statmode_display();

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
                    Event::Combat(monster_type) => {
                        self.set_statmode(StatMode::Combat);
                        self.currently_fighting = Some(monster_type);
                    }
                    Event::Vendor => {
                        self.set_statmode(StatMode::Vendor);
                    }
                    _ => (),
                }
            }
        }

        nocbreak();
        echo();
    }

    /// Redraw the main screen windows
    fn redraw_underwins(&self) {
        redrawwin(self.logwin);
        wrefresh(self.logwin);
        redrawwin(self.statwin);
        wrefresh(self.statwin);
        redrawwin(self.mapwin);
        wrefresh(self.mapwin);
    }
}

/// Main
fn main() {
    let all_args: Vec<String> = env::args().collect();
    let args = &all_args[1..];

    let mut discover_all = false;
    let mut force_bw = false;

    for a in args {
        match a.as_ref() {
            "-d" => discover_all = true,
            "-b" => force_bw = true,
            any => panic!("unknown command arg: {}", any),
        }
    }

    initscr();

    if !force_bw && has_colors() {
        start_color();
    }

    keypad(stdscr(), true);

    set_escdelay(100);

    refresh(); // If we don't do this first, windows don't show up

    let mut g = G::new(discover_all);

    g.run();

    endwin();
}
