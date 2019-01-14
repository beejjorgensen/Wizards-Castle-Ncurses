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
        G::mvwprintw_center(w, 2, "* * * THE WIZARD'S CASTLE * * *\n\n");
        self.wcoff(w, G::A_TITLE());

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
    pub fn choose_class(&mut self) {
        let w = G::popup(7, 48);

        self.wcon(w, G::A_TITLE());
        G::mvwprintw_center(w, 2, "All right, Bold One. You may be an:");
        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_BOLD());
        G::mvwprintw_center(w, 4, "|[E]|lf  |[D]|warf  Hu|[m]|an  |[H]|obbit");
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
        G::mvwprintw_center(w, 2, "Which sex do you prefer?");
        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_BOLD());
        G::mvwprintw_center(w, 4, "|[F]|emale  |[M]|ale");
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
            G::mvwprintw_center(
                w,
                2,
                &format!(
                    "Ok, {}, you have these statistics:",
                    self.player_race_name()
                ),
            );
            self.wcoff(w, G::A_TITLE());

            G::mvwprintw_center_notrim(
                w,
                4,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Strength),
                    self.game.player_stat(Stat::Strength)
                ),
            );
            G::mvwprintw_center_notrim(
                w,
                5,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Intelligence),
                    self.game.player_stat(Stat::Intelligence)
                ),
            );
            G::mvwprintw_center_notrim(
                w,
                6,
                &format!(
                    "{:>12}: {:>2}",
                    G::stat_name(Stat::Dexterity),
                    self.game.player_stat(Stat::Dexterity)
                ),
            );

            G::mvwprintw_center(
                w,
                8,
                &format!(
                    "And {} other points to allocate as you wish.",
                    additional_points
                ),
            );

            wattr_on(w, A_BOLD());
            G::mvwprintw_center(
                w,
                10,
                &format!(
                    "How many points do you add to {}?",
                    G::stat_name(*stat).to_uppercase()
                ),
            );

            G::mvwprintw_center(w, 12, &format!("Press |[0]| to |[{}]|", additional_points));
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

    /// Buy armor
    pub fn choose_armor(&mut self) {
        let armor = [
            ArmorType::Plate,
            ArmorType::Chainmail,
            ArmorType::Leather,
            ArmorType::None,
        ];

        let armor_names = ["|[P]|late", "|[C]|hainmail", "|[L]|eather", "|[N]|othing"];

        let armor_type_count = self.armor_purchase_type_count(false);

        if armor_type_count < 2 {
            return;
        }

        let w = G::popup(8 + armor_type_count as i32, 46);

        self.wcon(w, G::A_TITLE());
        G::mvwprintw_center(
            w,
            2,
            &format!("Choose your armor, {}.", self.player_race_name()),
        );
        self.wcoff(w, G::A_TITLE());

        G::mvwprintw_center(
            w,
            4,
            &format!("You have {} gold pieces.", self.game.player_gp()),
        );

        wattr_on(w, A_BOLD());

        let mut row_count = 0;

        for (i, armor_type) in armor.iter().enumerate() {
            if self.armor_can_afford(*armor_type, false) {
                let cost_str;

                if *armor_type == ArmorType::None {
                    cost_str = String::from("     ");
                } else {
                    cost_str = format!("{:2>} GP", Armor::cost(*armor_type, false));
                };

                G::mvwprintw_center_notrim(
                    w,
                    6 + row_count,
                    &format!("{:<14} {}", armor_names[i], cost_str),
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
                    'L' => break self.game.player_purchase_armor(ArmorType::Plate, false),
                    'N' => break self.game.player_purchase_armor(ArmorType::Plate, false),
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

        let weapon_names = ["|[S]|word", "|[M]|ace", "|[D]|agger", "|[N]|othing"];

        let weapon_type_count = self.weapon_purchase_type_count(false);

        if weapon_type_count < 2 {
            return;
        }

        let w = G::popup(8 + weapon_type_count as i32, 56);

        self.wcon(w, G::A_TITLE());
        G::mvwprintw_center(w, 2, "Now select your weapon of choice.");
        self.wcoff(w, G::A_TITLE());

        G::mvwprintw_center(
            w,
            4,
            &format!(
                "Ok, Bold {}, you have {} gold pieces left.",
                self.player_race_name(),
                self.game.player_gp()
            ),
        );

        wattr_on(w, A_BOLD());

        let mut row_count = 0;

        for (i, weapon_type) in weapon.iter().enumerate() {
            if self.weapon_can_afford(*weapon_type, false) {
                let cost_str;

                if *weapon_type == WeaponType::None {
                    cost_str = String::from("     ");
                } else {
                    cost_str = format!("{:2>} GP", Weapon::cost(*weapon_type, false));
                };

                G::mvwprintw_center_notrim(
                    w,
                    6 + row_count,
                    &format!("{:<14} {}", weapon_names[i], cost_str),
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
}
