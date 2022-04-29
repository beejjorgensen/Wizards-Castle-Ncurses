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
    OrbEvent, RandomMessage, Stairs,
};
use wizardscastle::monster::MonsterType;
use wizardscastle::room::RoomType;

use crate::stat::StatMode;

mod bribe;
mod gameover;
mod gen;
mod help;
mod info;
mod inv;
mod log;
mod map;
mod names;
mod quit;
mod stat;
mod teleport;
mod vendor;
mod win;

struct Opts {
    discover_all: bool,
    force_bw: bool,
    give_orb_of_zot: bool,
    give_runestaff: bool,
    locations: bool,
}

struct G {
    color: HashMap<&'static str, attr_t>,
    game: Game,
    mapwin: WINDOW,
    statwin: WINDOW,
    logwin: WINDOW,
    loginner: WINDOW,
    statmode: StatMode,

    rng: ThreadRng,

    retreat_direction: Option<Direction>,

    options: Opts,
}

impl G {
    /// Build a new global game struct
    fn new(options: Opts) -> G {
        // Build out the known color schemes for color and non-color terminals
        let mut color = HashMap::new();

        if has_colors() {
            init_pair(1, COLOR_YELLOW, COLOR_BLACK);
            init_pair(2, COLOR_RED, COLOR_BLACK);
            init_pair(3, COLOR_GREEN, COLOR_BLACK);
            init_pair(4, COLOR_BLUE, COLOR_BLACK);
            init_pair(5, COLOR_MAGENTA, COLOR_BLACK);
            init_pair(6, COLOR_CYAN, COLOR_BLACK);

            color.insert("bold-yellow", COLOR_PAIR(1) | A_BOLD());
            color.insert("bold-red", COLOR_PAIR(2) | A_BOLD());
            color.insert("red", COLOR_PAIR(2));
            color.insert("dim-yellow", COLOR_PAIR(1) | A_DIM());
            color.insert("dim-green", COLOR_PAIR(3) | A_DIM());
            color.insert("dim-red", COLOR_PAIR(2) | A_DIM());

            color.insert("room-.", A_DIM()); // Empis [sic]
            color.insert("room-E", 0);
            color.insert("room-D", 0);
            color.insert("room-U", 0);
            color.insert("room-G", COLOR_PAIR(1) | A_BOLD());
            color.insert("room-P", COLOR_PAIR(4) | A_BOLD());
            color.insert("room-C", 0);
            color.insert("room-F", A_BOLD());
            color.insert("room-W", 0);
            color.insert("room-S", 0);
            color.insert("room-O", COLOR_PAIR(6));
            color.insert("room-B", 0);
            color.insert("room-V", 0);
            color.insert("room-M", COLOR_PAIR(2) | A_BOLD());
            color.insert("room-T", COLOR_PAIR(1) | A_BOLD());
        } else {
            color.insert("bold-yellow", A_BOLD());
            color.insert("bold-red", 0);
            color.insert("red", 0);
            color.insert("dim-yellow", A_DIM());
            color.insert("dim-green", A_DIM());
            color.insert("dim-red", A_DIM());

            color.insert("room-.", A_DIM()); // Empis [sic]
            color.insert("room-E", 0);
            color.insert("room-D", 0);
            color.insert("room-U", 0);
            color.insert("room-G", A_BOLD());
            color.insert("room-P", 0);
            color.insert("room-C", 0);
            color.insert("room-F", A_BOLD());
            color.insert("room-W", 0);
            color.insert("room-S", 0);
            color.insert("room-O", 0);
            color.insert("room-B", 0);
            color.insert("room-V", 0);
            color.insert("room-M", 0);
            color.insert("room-T", A_BOLD());
        }

        let logwin = newwin(8, 80, 17, 0);
        let loginner = derwin(logwin, 7, 78, 0, 1);

        scrollok(loginner, true);

        let game = Game::new(8, 8, 8);

        // Return the new struct
        let mut g = G {
            color,
            mapwin: newwin(17, 47, 0, 0),
            statwin: newwin(17, 32, 0, 48),
            logwin,
            loginner,
            game,
            statmode: StatMode::None,

            retreat_direction: None,

            rng: thread_rng(),

            options,
        };

        g.restart(false);

        g
    }

    /// Restore the game to a clean slate for restarting
    fn restart(&mut self, new_game: bool) {
        if new_game {
            self.game = Game::new(8, 8, 8);
        }

        if self.options.give_orb_of_zot {
            self.game.debug_give_orb_of_zot();
        }

        if self.options.give_runestaff {
            self.game.debug_give_runestaff();
        }

        wclear(self.loginner);
        wmove(self.loginner, 0, 0);
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
            format!("{}{}", prefix, blind_suffix)
        } else {
            format!(
                "{} at ({},{}) level {}{}",
                prefix,
                x + 1,
                y + 1,
                z + 1,
                suffix
            )
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
            return;
        }

        if self.game.player_is_blind() {
            self.update_log_error(&format!(
                "** You can see anything, dumb {}",
                self.race_name()
            ));
            return;
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
                Err(err) => panic!("{:?}", err),
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

            Err(err) => panic!("{:?}", err),
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
            Err(err) => panic!("{:?}", err),
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

    /// Read a book
    fn read(&mut self) {
        let room_type = self.game.room_at_player().room_type().clone();

        if room_type == RoomType::Book {
            self.open_book();
        } else {
            self.update_log_error("** If you want to read, find a book!");
        }
    }

    /// Display a random message
    fn rand_message(&mut self) {
        match self.game.rand_message() {
            RandomMessage::SeeBat => self.update_log("You see a bat fly by."),
            RandomMessage::HearSound => {
                let sounds = ["a scream!", "footsteps.", "a wumpus.", "thunder."];

                let i = self.rng.gen_range(0..sounds.len());

                self.update_log(&format!("You hear {}", sounds[i]));
            }
            RandomMessage::Sneeze => self.update_log("You sneezed!"),
            RandomMessage::StepFrog => self.update_log("You stepped on a frog."),
            RandomMessage::MonsterFrying => {
                let mon_name = self.rand_monster_name();
                self.update_log(&format!("You smell {} frying.", mon_name));
            }
            RandomMessage::Watched => self.update_log("You feel like you're being watched."),
            RandomMessage::Playing => self.update_log("You are playing Wizard's Castle."),
            RandomMessage::None => (),
        }
    }

    /// Things to do at the start of the turn
    fn at_turn_start(&mut self) {
        self.game.add_turn(1);

        self.game.curse_effects();

        if self.game.curse_check() {
            self.update_log_bad("You feel a chill in your bones.");
        }

        self.rand_message();

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

    /// Print a message to find a vendor
    fn find_vendor_error(&self) {
        self.update_log_error("** If you want to trade, find a vendor.");
    }

    /// Trade with a vendor
    fn trade(&mut self) {
        match self.game.room_at_player().room_type() {
            RoomType::Monster(m) => {
                if m.monster_type() == MonsterType::Vendor {
                    if self.game.vendors_angry() {
                        self.update_log_error("** The vendor's in no mood to trade!");
                    } else {
                        self.vendor_trade();
                    }
                } else {
                    self.find_vendor_error();
                }
            }
            _ => self.find_vendor_error(),
        }
    }

    /// Trade or teleport
    fn trade_teleport(&mut self) {
        let mut is_vendor = false;
        let can_teleport = self.game.player_has_runestaff();

        if let RoomType::Monster(m) = self.game.room_at_player().room_type() {
            if m.monster_type() == MonsterType::Vendor {
                is_vendor = true;
            }
        }

        if is_vendor {
            self.trade();
        } else if can_teleport {
            self.teleport();
        } else {
            self.find_vendor_error();
            self.teleport_error();
        }
    }

    /// Print messaging when monster defeated by melee or magic
    fn monster_defeated_message(&mut self, result: HitResult, mon_art: &str, m_str: &str) {
        self.update_log_good(&format!(
            "{} {} lies dead at your feet!",
            G::initial_upper(mon_art),
            m_str
        ));

        if self.game.rand_recipe() {
            let suffix = [
                "wich", " stew", " soup", " burger", " roast", " munchy", " taco", " pie",
            ];

            let i = self.rng.gen_range(0..suffix.len());

            self.update_log(&format!("You spend an hour eating {}{}.", m_str, suffix[i]));
        }

        if result.killed_vendor {
            self.update_log("You get all his wares: plate armor, a sword, a strength potion,");

            let mut s = String::from("an intelligence potion, ");

            if result.got_lamp {
                s.push_str("a dexterity potion, and a lamp.");
            } else {
                s.push_str("and a dexterity potion.");
            }

            self.update_log(&s);
        } else {
            if result.got_runestaff {
                self.update_log_good("** GREAT ZOT! YOU'VE FOUND THE RUNESTAFF! **");
            }

            self.update_log(&format!("You now get his hoard of {} GPs", result.treasure));
        }
    }

    /// Attack a thing
    fn attack(&mut self) {
        if self.game.state() == GameState::Vendor {
            self.update_log_bad("** You'll be sorry you did that!");
            self.game.vendor_attack();
        }
    }

    /// Combat: player attack melee
    fn combat_player_attack_melee(&mut self, mon_str: &str, mon_art: &str) -> bool {
        let weapon_type = self.game.player_weapon_type();

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

                if result.defeated {
                    self.monster_defeated_message(result, &mon_art, &mon_str);
                    return true;
                }
            }

            Ok(CombatEvent::Miss) => {
                self.update_log_bad("Drat! Missed!");
            }

            Ok(any) => panic!("unexpected combat event {:#?}", any),

            Err(err) => panic!("error in combat {:#?}", err),
        }

        false
    }

    /// Start a retreat
    fn combat_retreat(&mut self, dir: Direction) {
        self.retreat_direction = Some(dir);

        match self.game.retreat() {
            Ok(_) => (),
            Err(err) => panic!("error retreating {:#?}", err),
        };
    }

    /// Combat: player spell
    fn combat_player_attack_spell(&mut self, mon_str: &str, mon_art: &str) -> bool {
        let mut valid = false;
        let mut done = false;

        self.set_statmode(StatMode::Spell);

        while !valid {
            let ch = G::norm_key(getch());

            match ch {
                'W' => match self.game.spell_web() {
                    Ok(CombatEvent::Hit(_)) => (),
                    Ok(CombatEvent::Died) => (),
                    Ok(any) => panic!("Unexpected: {:#?}", any),
                    Err(err) => panic!("{:#?}", err),
                },

                'F' => match self.game.spell_fireball() {
                    Ok(CombatEvent::Hit(hr)) => {
                        self.update_log_good(&format!(
                            "The fireball does {} points of damage!",
                            hr.damage
                        ));
                        if hr.defeated {
                            self.monster_defeated_message(hr, mon_art, mon_str);
                            done = true;
                        }
                    }
                    Ok(CombatEvent::Died) => (),
                    Ok(any) => panic!("Unexpected: {:#?}", any),
                    Err(err) => panic!("{:#?}", err),
                },

                'D' => match self.game.spell_deathspell() {
                    Ok(CombatEvent::Hit(hr)) => {
                        self.update_log_good("Death... his!");
                        self.monster_defeated_message(hr, mon_art, mon_str);
                        done = true;
                    }
                    Ok(CombatEvent::Died) => {
                        self.update_log_bad("Death... YOURS!");
                    }
                    Ok(any) => panic!("Unexpected: {:#?}", any),
                    Err(err) => panic!("{:#?}", err),
                },

                _ => (),
            }

            if ch == 'W' || ch == 'F' || ch == 'D' || ch == 'N' {
                valid = true;
            }
        }

        self.set_statmode(StatMode::Combat);

        done
    }

    /// Combat: player attacks
    fn combat_player_attack(&mut self, monster_type: MonsterType, bribed: &mut bool) -> bool {
        let mon_str = G::monster_name(monster_type);
        let mon_art = G::get_article(&mon_str);

        let mut done = false;
        *bribed = false;

        self.update_stat(); // might not be able to bribe or cast spells anymore

        match G::norm_key(getch()) {
            'A' => done = self.combat_player_attack_melee(&mon_str, &mon_art),
            'B' => {
                if self.game.bribe_possible() && self.game.player_has_any_treasure() {
                    done = self.combat_bribe();
                    *bribed = done;
                }
            }
            'C' => done = self.combat_player_attack_spell(&mon_str, &mon_art),
            'N' => self.combat_retreat(Direction::North),
            'S' => self.combat_retreat(Direction::South),
            'W' => self.combat_retreat(Direction::West),
            'E' => self.combat_retreat(Direction::East),
            _ => (),
        }

        done
    }

    /// Be attacked by a monster
    fn combat_monster_attack(&mut self, monster_type: MonsterType) {
        let mon_str = G::monster_name(monster_type);

        match self.game.be_attacked() {
            Ok(CombatEvent::MonsterWebbed) => {
                self.update_log_good(&format!("The {} is stuck and can't attack!", mon_str));
            }

            Ok(CombatEvent::MonsterHit(_damage, _defeated, armor_destroyed, web_broke)) => {
                if web_broke {
                    self.update_log_bad("The web just broke!");
                }

                self.update_log(&format!("The {} attacks!", mon_str));

                self.update_log_bad("OUCH! He hit you!");

                if armor_destroyed {
                    self.update_log_bad("Your armor is destroyed--good luck!");
                }
            }

            Ok(CombatEvent::MonsterMiss) => {
                self.update_log(&format!("The {} attacks!", mon_str));

                self.update_log("Hah! He missed you!");
            }

            Ok(any) => panic!("unexpected event while being attacked {:#?}", any),

            Err(err) => panic!("error in combat being attacked {:#?}", err),
        }
    }

    /// Combat
    fn combat(&mut self, monster_type: MonsterType) -> bool {
        let mut done = false;
        let mut retreated = false;
        let mut bribed = false;

        self.retreat_direction = None;

        self.update_map(self.options.discover_all); // update player position
        self.set_statmode(StatMode::Combat);

        let mon_str = G::monster_name(monster_type);
        let mon = format!("{} {}", G::get_article(&mon_str), mon_str).to_uppercase();

        self.update_log_bad(&format!("You're facing {}!", mon));

        while !done {
            //self.update_log(&format!(">> {:#?}", self.game.state()));
            match self.game.state() {
                GameState::PlayerAttack => {
                    done = self.combat_player_attack(monster_type, &mut bribed)
                }
                GameState::MonsterAttack => self.combat_monster_attack(monster_type),
                GameState::Dead => done = true,
                GameState::Retreat => {
                    self.game.retreat_dir(self.retreat_direction.unwrap());
                    self.update_log("You have escaped.");
                    retreated = true;
                    done = true;
                }
                any => panic!("Unexpected game state: {:#?}", any),
            }
        } // while !done

        self.set_statmode(StatMode::None);

        if bribed {
            self.update_log(&format!(
                "The {} is happy with your bribe... for now.",
                &mon_str
            ));
        }

        retreated
    }

    /// Death message on log
    fn death_message(&self) {
        self.update_log_bad("** You have died! **");
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

            self.restart(true);

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

            if self.options.locations {
                self.update_log(&format!(
                    ">>> Orb of Zot: {}",
                    self.game.debug_orb_of_zot_location()
                ));
                self.update_log(&format!(
                    ">>> Runestaff: {}",
                    self.game.debug_runestaff_location()
                ));
            }

            while alive {
                if self.game.state() == GameState::Dead {
                    self.death_message();
                    alive = false;
                    continue;
                }

                self.at_turn_start();

                self.update_map(self.options.discover_all);
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
                        'R' => self.read(),
                        'I' => self.show_inventory(),
                        'T' => self.trade_teleport(),
                        'P' => self.teleport(),
                        'H' | '?' => self.help(),
                        'C' => self.info(),
                        'Q' => {
                            if self.verify_quit(false) {
                                alive = false;
                                // TODO play again?
                                playing = false;
                                continue;
                            }
                        }
                        _ => (),
                    }
                } else {
                    automove = false;
                }

                if self.game.state() == GameState::Dead {
                    self.death_message();
                    alive = false;
                    continue;
                }

                if self.game.state() == GameState::Exit {
                    self.update_log("You have exited the castle.");
                    alive = false;
                    continue;
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
                        automove = true;
                    }
                    Event::Warp => {
                        let msg =
                            self.make_loc_event_msg("You entered a warp", "!", "!", ox, oy, oz);
                        self.update_log(&msg);
                        automove = true;
                    }
                    Event::Treasure(t) => {
                        let msg = &format!(
                            "Here you find the {}! It's now yours!",
                            G::treasure_name(*t.treasure_type())
                        );
                        self.update_log_good(&msg);
                    }
                    Event::Combat(monster_type) => automove = self.combat(monster_type),
                    Event::Vendor => {
                        self.set_statmode(StatMode::Vendor);
                    }
                    _ => (),
                }
            } // while alive

            if playing {
                playing = self.game_summary();
            }
        } // while playing

        nocbreak();
        echo();
        G::show_cursor(true);
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

/// Gather command line options
fn gather_options() -> Opts {
    let all_args: Vec<String> = env::args().collect();
    let args = &all_args[1..];

    let mut discover_all = false;
    let mut force_bw = false;
    let mut give_orb_of_zot = false;
    let mut give_runestaff = false;
    let mut locations = false;

    if cfg!(feature = "cheat") {
        for a in args {
            match a.as_ref() {
                "-d" => discover_all = true,
                "-b" => force_bw = true,
                "-z" => give_orb_of_zot = true,
                "-r" => give_runestaff = true,
                "-l" => locations = true,
                any => panic!("unknown command arg: {}", any),
            }
        }
    }

    Opts {
        discover_all,
        force_bw,
        give_orb_of_zot,
        give_runestaff,
        locations,
    }
}

/// Main
fn main() {
    let options = gather_options();

    initscr();

    if !options.force_bw && has_colors() {
        start_color();
    }

    keypad(stdscr(), true);

    set_escdelay(100);

    refresh(); // If we don't do this first, windows don't show up

    let mut g = G::new(options);

    g.run();

    endwin();
}
