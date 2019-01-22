use crate::G;
use ncurses::*;

use wizardscastle::armor::{Armor, ArmorType};
use wizardscastle::player::{Gender, Race, Stat};
use wizardscastle::weapon::{Weapon, WeaponType};

impl G {
    /// Do the intro
    pub fn intro(&self) {
        let w = G::popup(15, 64);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "* * * THE WIZARD'S CASTLE * * *\n\n");
        self.wcoff(w, G::A_TITLE());

        wattron(w, A_BOLD());
        self.wprintw_center(w, "Many cycles ago, in the Kingdom of N'DIC, the gnomic\n");
        self.wprintw_center(w, "wizard ZOT forged his great *ORB OF POWER*. He soon\n");
        self.wprintw_center(w, "vanished, leaving behind his vast subterranean castle\n");
        self.wprintw_center(
            w,
            "filled with esurient monsters, fabulous treasures, and\n",
        );
        self.wprintw_center(
            w,
            "the incredible *ORB OF ZOT*. From that time hence, many\n",
        );
        self.wprintw_center(
            w,
            "a bold youth has ventured into the wizard's castle. As\n",
        );
        self.wprintw_center(
            w,
            "of now, *NONE* has ever emerged victoriously! BEWARE!!\n\n",
        );
        wattroff(w, A_BOLD());

        wattron(w, A_REVERSE());
        self.wprintw_center(w, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);

        wrefresh(w);

        getch();

        G::popup_close(w);
    }

    /// Choose class
    pub fn choose_class(&mut self) {
        let w = G::popup(7, 48);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "All right, Bold One. You may be an:");
        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_BOLD());
        self.mvwprintw_center(w, 4, "|[E]|lf  |[D]|warf  Hu|[m]|an  |[H]|obbit");
        wattr_off(w, A_BOLD());

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
    pub fn choose_gender(&mut self) {
        let w = G::popup(7, 36);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "Which sex do you prefer?");
        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_BOLD());
        self.mvwprintw_center(w, 4, "|[F]|emale  |[M]|ale");
        wattr_off(w, A_BOLD());

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
    pub fn choose_stats(&mut self) {
        let w = G::popup(15, 50);

        let stats = [Stat::Strength, Stat::Intelligence, Stat::Dexterity];

        for stat in stats.iter() {
            let additional_points = self.game.player_additional_points();

            if additional_points == 0 {
                break;
            }

            wclear(w);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(
                w,
                2,
                &format!(
                    "Ok, {}, you have these statistics:",
                    self.player_race_name()
                ),
            );
            self.wcoff(w, G::A_TITLE());

            self.mvwprintw_center_notrim(
                w,
                4,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Strength),
                    self.game.player_stat(Stat::Strength)
                ),
            );
            self.mvwprintw_center_notrim(
                w,
                5,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Intelligence),
                    self.game.player_stat(Stat::Intelligence)
                ),
            );
            self.mvwprintw_center_notrim(
                w,
                6,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Dexterity),
                    self.game.player_stat(Stat::Dexterity)
                ),
            );

            self.mvwprintw_center(
                w,
                8,
                &format!(
                    "And {} other points to allocate as you wish.",
                    additional_points
                ),
            );

            wattr_on(w, A_BOLD());
            self.mvwprintw_center(
                w,
                10,
                &format!(
                    "How many points do you add to {}?",
                    G::stat_name(*stat).to_uppercase()
                ),
            );

            self.mvwprintw_center(w, 12, &format!("Press |[0]| to |[{}]|", additional_points));
            wattr_off(w, A_BOLD());

            box_(w, 0, 0);

            wrefresh(w);

            loop {
                let key = getch();

                let nkey = G::norm_key(key);

                if let '0'...'9' = nkey {
                    let v = nkey.to_digit(10).unwrap();

                    if v > additional_points {
                        continue;
                    }

                    if let Err(err) = self.game.player_allocate_points(*stat, v) {
                        panic!(err);
                    }

                    break;
                }
            }
        }

        G::popup_close(w);
    }

    /// Return the number of types of armor the player can afford.
    ///
    /// This function counts "nothing" as a type of armor, so will always return
    /// at least 1.
    fn armor_purchase_type_count(&self, is_vendor: bool) -> u32 {
        let armor = [
            ArmorType::Plate,
            ArmorType::Chainmail,
            ArmorType::Leather,
            ArmorType::None,
        ];

        let mut count = 0;

        for a in armor.iter() {
            if self.armor_can_afford(*a, is_vendor) {
                count += 1;
            }
        }

        count
    }

    /// True if the player can afford an armor
    fn armor_can_afford(&self, armor: ArmorType, is_vendor: bool) -> bool {
        Armor::cost(armor, is_vendor) <= self.game.player_gp()
    }

    /// Return the number of types of weapon the player can afford.
    ///
    /// This function counts "nothing" as a type of weapon, so will always return
    /// at least 1.
    fn weapon_purchase_type_count(&self, is_vendor: bool) -> u32 {
        let weapon = [
            WeaponType::Sword,
            WeaponType::Mace,
            WeaponType::Dagger,
            WeaponType::None,
        ];

        let mut count = 0;

        for a in weapon.iter() {
            if self.weapon_can_afford(*a, is_vendor) {
                count += 1;
            }
        }

        count
    }

    /// True if the player can afford an weapon
    fn weapon_can_afford(&self, weapon: WeaponType, is_vendor: bool) -> bool {
        Weapon::cost(weapon, is_vendor) <= self.game.player_gp()
    }

    /// Convert a string from the form "Foo" to the form "|[F]|oo"
    fn name_to_menuitem(s: &str) -> String {
        format!("|[{}]|{}", s.get(0..1).unwrap(), s.get(1..).unwrap())
    }

    /// Buy armor
    pub fn choose_armor(&mut self) {
        let armor = [
            ArmorType::Plate,
            ArmorType::Chainmail,
            ArmorType::Leather,
            ArmorType::None,
        ];

        let armor_type_count = self.armor_purchase_type_count(false);

        if armor_type_count < 2 {
            return;
        }

        let w = G::popup(8 + armor_type_count as i32, 46);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(
            w,
            2,
            &format!(
                "Ok, {}, you have {} gold pieces.",
                self.player_race_name(),
                self.game.player_gp()
            ),
        );
        self.wcoff(w, G::A_TITLE());

        self.mvwprintw_center(w, 4, &"Here is a list of armor you can buy.".to_string());

        wattr_on(w, A_BOLD());

        let mut row_count = 0;

        for armor_type in armor.iter() {
            if self.armor_can_afford(*armor_type, false) {
                let cost_str;

                if *armor_type == ArmorType::None {
                    cost_str = String::from("     ");
                } else {
                    cost_str = format!("{:2>} GP", Armor::cost(*armor_type, false));
                };

                self.mvwprintw_center_notrim(
                    w,
                    6 + row_count,
                    &format!(
                        "{:<14} {}",
                        G::name_to_menuitem(&G::armor_name(*armor_type)),
                        cost_str
                    ),
                );

                row_count += 1;
            }
        }

        wattr_off(w, A_BOLD());

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            let r = loop {
                let key = getch();

                match G::norm_key(key) {
                    'P' => break self.game.player_purchase_armor(ArmorType::Plate, false),
                    'C' => break self.game.player_purchase_armor(ArmorType::Chainmail, false),
                    'L' => break self.game.player_purchase_armor(ArmorType::Leather, false),
                    'N' => break self.game.player_purchase_armor(ArmorType::None, false),
                    _ => (),
                };
            };

            if r.is_ok() {
                break;
            }
        }

        G::popup_close(w);
    }

    /// Buy weapon
    pub fn choose_weapon(&mut self) {
        let weapon = [
            WeaponType::Sword,
            WeaponType::Mace,
            WeaponType::Dagger,
            WeaponType::None,
        ];

        let weapon_type_count = self.weapon_purchase_type_count(false);

        if weapon_type_count < 2 {
            return;
        }

        let w = G::popup(8 + weapon_type_count as i32, 56);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(
            w,
            2,
            &format!(
                "Ok, Bold {}, you have {} gold pieces left.",
                self.player_race_name(),
                self.game.player_gp()
            ),
        );
        self.wcoff(w, G::A_TITLE());

        self.mvwprintw_center(w, 4, "Here is a list of weapons you can buy.");

        wattr_on(w, A_BOLD());

        let mut row_count = 0;

        for weapon_type in weapon.iter() {
            if self.weapon_can_afford(*weapon_type, false) {
                let cost_str;

                if *weapon_type == WeaponType::None {
                    cost_str = String::from("     ");
                } else {
                    cost_str = format!("{:2>} GP", Weapon::cost(*weapon_type, false));
                };

                self.mvwprintw_center_notrim(
                    w,
                    6 + row_count,
                    &format!(
                        "{:<14} {}",
                        G::name_to_menuitem(&G::weapon_name(*weapon_type)),
                        cost_str
                    ),
                );

                row_count += 1;
            }
        }

        wattr_off(w, A_BOLD());

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            let r = loop {
                let key = getch();

                match G::norm_key(key) {
                    'S' => break self.game.player_purchase_weapon(WeaponType::Sword, false),
                    'M' => break self.game.player_purchase_weapon(WeaponType::Mace, false),
                    'D' => break self.game.player_purchase_weapon(WeaponType::Dagger, false),
                    'N' => break self.game.player_purchase_weapon(WeaponType::None, false),
                    _ => (),
                };
            };

            if r.is_ok() {
                break;
            }
        }

        G::popup_close(w);
    }

    /// Buy lamp
    pub fn choose_lamp(&mut self) {
        if !self.game.player_can_purchase_lamp() {
            return;
        }

        let w = G::popup(7, 40);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "Want to buy a lamp for 20 GPs?");
        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_BOLD());
        self.mvwprintw_center(w, 4, "|[Y]|es  |[N]|o");
        wattr_off(w, A_BOLD());

        box_(w, 0, 0);

        wrefresh(w);

        let r = loop {
            let key = getch();

            match G::norm_key(key) {
                'Y' => break self.game.player_purchase_lamp(true),
                'N' => break self.game.player_purchase_lamp(false),
                _ => (),
            }
        };

        if let Err(err) = r {
            panic!(err);
        }

        G::popup_close(w);
    }

    /// Buy flares
    pub fn choose_flares(&mut self) {
        let gps = self.game.player_gp();

        if gps == 0 {
            return;
        }

        let w = G::popup(9, 54);

        let mut input = String::from("invalid");

        let mut success = false;

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(
            w,
            2,
            &format!(
                "Ok, {}, you have {} GPs left.",
                self.player_race_name(),
                gps
            ),
        );
        self.wcoff(w, G::A_TITLE());

        self.mvwprintw_center(w, 4, "Flares cost 1 GP each.");

        box_(w, 0, 0);

        while !success {
            mvwprintw(w, 6, 15 + 22, "    "); // erase old input
            mvwprintw(w, 6, 15, "How many do you want? ");

            wrefresh(w);
            redrawwin(w);

            nocbreak();
            echo();
            G::show_cursor(true);

            wgetnstr(w, &mut input, 2);

            input = input.trim().to_string();

            G::show_cursor(false);
            noecho();
            cbreak();

            if let Ok(num) = input.parse::<u32>() {
                if num > gps {
                    self.popup_error(&format!("You can only afford {}!", gps));
                } else {
                    if let Err(err) = self.game.player_purchase_flares(num) {
                        panic!(err);
                    }
                    success = true;
                }
            } else {
                self.popup_error("If you don't wany any, just type 0.");
            }
        }

        G::popup_close(w);
    }
}
