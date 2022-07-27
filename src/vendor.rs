use crate::G;
use ncurses::*;

use wizardscastle::armor::{Armor, ArmorType};
use wizardscastle::game::Game;
use wizardscastle::player::Stat;
use wizardscastle::treasure::TreasureType;
use wizardscastle::weapon::{Weapon, WeaponType};

impl G {
    /// Helper function to print gold pieces
    fn print_gps(&self, w: WINDOW) {
        let gps = self.game.player_gp();

        let unit = if gps == 1 { "GP" } else { "GPs" };

        self.wprintw_center(w, &format!("You have {} {}.\n\n", gps, unit));
    }

    /// Sell treasures
    ///
    /// Return true if the player hit ESC and wants to bail from the vendor
    fn vendor_trade_treasure(&mut self) -> bool {
        let treasures = self.game.player_get_treasures();

        if treasures.is_empty() {
            return false;
        }

        let pricemap = match self.game.vendor_treasure_offer() {
            Ok(p) => p,
            Err(err) => panic!("{:?}", err),
        };

        let height = (10 + treasures.len()) as i32;
        let w = G::popup(height, 40);

        let mut sold: Vec<TreasureType> = Vec::new();

        let mut done = false;
        let mut bailout = false;

        while !done {
            wclear(w);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "What do you want to sell?\n\n");
            self.wcoff(w, G::A_TITLE());

            self.print_gps(w);

            for (i, t) in treasures.iter().enumerate() {
                let price = match pricemap.get(t) {
                    Some(p) => p,
                    None => panic!("price missing"),
                };

                let gps = format!("{} GP", price);

                let tname = if sold.contains(t) {
                    self.wcon(w, "dim-green");
                    format!("||    The {}", G::treasure_name(*t))
                } else {
                    format!("|[{}]| The {}", i + 1, G::treasure_name(*t))
                };

                self.wprintw_center_notrim(w, &format!("{:<22}{:>8}\n", tname, gps));

                wattr_set(w, 0, 0);
            }

            self.mvwprintw_center(w, height - 3, "|[N]|othing");

            box_(w, 0, 0);
            wrefresh(w);

            let ch = getch();

            if ch == 27 {
                done = true;
                bailout = true;
            }

            let nch = G::norm_key(ch);
            match nch {
                '1'..='9' => {
                    let index = (nch.to_digit(10).unwrap() - 1) as usize;
                    if index < treasures.len() {
                        let tt = treasures[index];
                        if !sold.contains(&tt) {
                            if let Err(err) = self.game.vendor_treasure_accept(tt) {
                                panic!("{:?}", err);
                            }
                            sold.push(tt);
                        }
                    }
                }
                'N' => done = true,
                _ => (),
            }
        }

        G::popup_close(w);

        self.redraw_underwins();

        bailout
    }

    /// Print armor name and price
    fn print_armor_prompt(&self, w: WINDOW, armor_type: ArmorType) {
        let name = match armor_type {
            ArmorType::Leather => "|[L]|eather",
            ArmorType::Chainmail => "|[C]|hainmail",
            ArmorType::Plate => "|[P]|late",
            t => panic!("invalid armor type {:#?}", t),
        };

        let costgp = Armor::cost(armor_type, true);

        let cost = format!("{} GP", costgp);

        if costgp > self.game.player_gp() {
            wattr_on(w, self.wcget("dim-red"));
        }

        self.wprintw_center(w, &format!("{:<16}{:>8}", name, cost));

        wattr_off(w, self.wcget("dim-red"));

        wprintw(w, "\n");
    }

    /// Print weapon name and price
    fn print_weapon_prompt(&self, w: WINDOW, weapon_type: WeaponType) {
        let name = match weapon_type {
            WeaponType::Dagger => "|[D]|agger",
            WeaponType::Mace => "|[M]|ace",
            WeaponType::Sword => "|[S]|word",
            t => panic!("invalid weapon type {:#?}", t),
        };

        let costgp = Weapon::cost(weapon_type, true);

        let cost = format!("{} GP", costgp);

        if costgp > self.game.player_gp() {
            wattr_on(w, self.wcget("dim-red"));
        }

        self.wprintw_center(w, &format!("{:<16}{:>8}", name, cost));

        wattr_off(w, self.wcget("dim-red"));

        wprintw(w, "\n");
    }

    /// Warn the user that they're about to purchase something they already have
    fn warn_purchase(&self, purchase_type: &str, downgrade: bool) -> bool {
        let or_better_msg = if downgrade { " or better" } else { "" };
        let title = format!("You already have {}{}!", purchase_type, or_better_msg);
        let width = if downgrade { title.len() + 10 } else { 45 };

        let w = G::popup(9, width as i32);

        self.wcon(w, G::A_WARN_TITLE());
        self.mvwprintw_center(w, 2, &title);
        self.wcoff(w, G::A_WARN_TITLE());

        wprintw(w, "\n\n");

        if downgrade {
            self.wprintw_center(w, "Are you sure you want to buy this?");
        } else {
            self.wprintw_center(w, "Do you really want to buy it again?");
        }

        wprintw(w, "\n\n");

        self.wprintw_center(w, "|[Y]|es   |[N]|o");

        box_(w, 0, 0);
        wrefresh(w);

        let yes = G::norm_key(getch()) == 'Y';

        G::popup_close(w);

        yes
    }

    /// Purchase armor
    fn purchase_armor(&mut self, armor_type: ArmorType) {
        if !self.game.vendor_can_afford_armor_type(armor_type) {
            return;
        }

        if self.game.player_has_at_least_armor(armor_type)
            && !self.warn_purchase("that armor", true)
        {
            return;
        }

        if self.game.player_purchase_armor(armor_type, true).is_err() { /* do nothing */ }
    }

    /// Purchase weapon
    fn purchase_weapon(&mut self, weapon_type: WeaponType) {
        if !self.game.vendor_can_afford_weapon_type(weapon_type) {
            return;
        }

        if self.game.player_has_at_least_weapon(weapon_type)
            && !self.warn_purchase("that weapon", true)
        {
            return;
        }

        if self.game.player_purchase_weapon(weapon_type, true).is_err() { /* do nothing */ }
    }

    /// Purchase lamp
    fn purchase_lamp(&mut self) {
        if !self.game.vendor_can_afford_lamp() {
            return;
        }

        if self.game.player_has_lamp() && !self.warn_purchase("a lamp", false) {
            return;
        }

        if self.game.vendor_buy_lamp().is_err() { /* do nothing */ }
    }

    /// Trade equipment
    fn trade_equipment(&mut self) -> bool {
        let mut bailout = false;

        let height = 20;

        let w = G::popup(height, 47);

        let mut done = false;

        while !done {
            wclear(w);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "What equipment would you like to buy?\n\n");
            self.wcoff(w, G::A_TITLE());

            let mut cur_equip = format!(
                "You have: {}, {}",
                G::armor_name(self.game.player_armor_type()),
                G::weapon_name(self.game.player_weapon_type())
            );

            if self.game.player_has_lamp() {
                cur_equip = format!("{}, Lamp", cur_equip);
            }

            self.wprintw_center(w, &cur_equip);
            wprintw(w, "\n");
            self.wprintw_center(w, &format!("and {} GPs", self.game.player_gp()));

            wprintw(w, "\n\n");

            self.print_armor_prompt(w, ArmorType::Leather);
            self.print_armor_prompt(w, ArmorType::Chainmail);
            self.print_armor_prompt(w, ArmorType::Plate);

            wprintw(w, "\n");

            self.print_weapon_prompt(w, WeaponType::Dagger);
            self.print_weapon_prompt(w, WeaponType::Mace);
            self.print_weapon_prompt(w, WeaponType::Sword);

            wprintw(w, "\n");

            {
                let name = "L|[a]|mp";
                let cost = format!("{} GP", Game::vendor_lamp_cost());

                if !self.game.vendor_can_afford_lamp() {}

                if !self.game.vendor_can_afford_lamp() {
                    wattr_on(w, self.wcget("dim-red"));
                }

                self.wprintw_center(w, &format!("{:<16}{:>8}\n", name, cost));

                wattr_off(w, self.wcget("dim-red"));
            }

            self.mvwprintw_center(w, height - 3, "|[N]|othing");

            box_(w, 0, 0);
            wrefresh(w);

            let ch = getch();

            if ch == 27 {
                done = true;
                bailout = true;
            }

            match G::norm_key(ch) {
                'N' => done = true,
                'L' => self.purchase_armor(ArmorType::Leather),
                'C' => self.purchase_armor(ArmorType::Chainmail),
                'P' => self.purchase_armor(ArmorType::Plate),
                'D' => self.purchase_weapon(WeaponType::Dagger),
                'M' => self.purchase_weapon(WeaponType::Mace),
                'S' => self.purchase_weapon(WeaponType::Sword),
                'A' => self.purchase_lamp(),
                _ => (),
            }
        }

        G::popup_close(w);

        self.redraw_underwins();

        bailout
    }

    /// Helper function to buy stat
    fn buy_stat(&mut self, stat: Stat) {
        if self.game.player_stat_maxed(stat) {
            return;
        }

        if self.game.vendor_buy_stat(stat).is_err() {
            // ignore errors
        }
    }

    /// Print out a stat
    fn print_stat_prompt(&self, w: WINDOW, stat: Stat) {
        if !self.game.vendor_can_afford_stat() {
            wattr_on(w, self.wcget("dim-red"));
        } else if self.game.player_stat_maxed(stat) {
            wattr_on(w, self.wcget("dim-green"));
        };

        let prompt = match stat {
            Stat::Strength => "|[S]|trength",
            Stat::Intelligence => "|[I]|ntelligence",
            Stat::Dexterity => "|[D]|exterity",
        };

        let cost = format!("{} GP", Game::vendor_stat_cost());

        self.wprintw_center(w, &format!("{:<17}  {}\n", prompt, cost));

        wattr_off(w, self.wcget("dim-red"));
        wattr_off(w, self.wcget("dim-green"));
    }

    /// Trade for potions
    pub fn trade_potions(&mut self) {
        if !self.game.vendor_can_afford_stat() || self.game.player_all_stats_maxed() {
            return;
        }

        let height = 14;

        let w = G::popup(height, 41);

        let mut done = false;

        while !done {
            wclear(w);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "Would you like to buy some potions?\n\n");
            self.wcoff(w, G::A_TITLE());

            self.wprintw_center(
                w,
                &format!(
                    "ST={}  IQ={}  DX={}\n",
                    self.game.player_stat(Stat::Strength),
                    self.game.player_stat(Stat::Intelligence),
                    self.game.player_stat(Stat::Dexterity)
                ),
            );

            self.wprintw_center(w, &format!("You have {} GPs\n\n", self.game.player_gp()));

            self.print_stat_prompt(w, Stat::Strength);
            self.print_stat_prompt(w, Stat::Intelligence);
            self.print_stat_prompt(w, Stat::Dexterity);

            self.mvwprintw_center(w, height - 3, "|[N]|othing");

            box_(w, 0, 0);
            wrefresh(w);

            let ch = getch();

            if ch == 27 {
                done = true;
            }

            match G::norm_key(ch) {
                'S' => self.buy_stat(Stat::Strength),
                'I' => self.buy_stat(Stat::Intelligence),
                'D' => self.buy_stat(Stat::Dexterity),
                'N' => done = true,
                _ => (),
            }
        }

        G::popup_close(w);
        self.redraw_underwins();
    }

    /// Trade with a Vendor
    pub fn vendor_trade(&mut self) {
        if self.vendor_trade_treasure() {
            return;
        }

        if !self.game.vendor_can_afford_anything() {
            self.update_log_error(&format!(
                "** You're too poor to trade, {}.",
                self.race_name()
            ));
            return;
        }

        if (self.game.vendor_can_afford_armor()
            || self.game.vendor_can_afford_weapon()
            || self.game.vendor_can_afford_lamp())
            && self.trade_equipment()
        {
            return;
        }

        self.trade_potions();
    }
}
