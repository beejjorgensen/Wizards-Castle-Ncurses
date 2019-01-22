use crate::G;
use ncurses::*;

use wizardscastle::armor::{Armor, ArmorType};
use wizardscastle::game::Game;
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
            Err(err) => panic!(err),
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
                let price = match pricemap.get(&t) {
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
                '1'...'9' => {
                    let index = (nch.to_digit(10).unwrap() - 1) as usize;
                    if index < treasures.len() {
                        let tt = treasures[index];
                        if !sold.contains(&tt) {
                            if let Err(err) = self.game.vendor_treasure_accept(tt) {
                                panic!(err);
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

    /// Get a list of weapons you can purchase
    fn purchasable_weapons(&self) -> Option<Vec<WeaponType>> {
        let weapon_list = [WeaponType::Dagger, WeaponType::Mace, WeaponType::Sword];

        let mut weapons = Vec::new();

        for w in weapon_list.iter() {
            if self.game.vendor_can_afford_weapon_type(*w)
                && !self.game.player_has_at_least_weapon(*w)
            {
                weapons.push(*w);
            }
        }

        if weapons.is_empty() {
            None
        } else {
            Some(weapons)
        }
    }

    /// Get a list of armor you can purchase
    fn purchasable_armor(&self) -> Option<Vec<ArmorType>> {
        let armor_list = [ArmorType::Leather, ArmorType::Chainmail, ArmorType::Plate];

        let mut armor = Vec::new();

        for a in armor_list.iter() {
            if self.game.vendor_can_afford_armor_type(*a)
                && !self.game.player_has_at_least_armor(*a)
            {
                armor.push(*a);
            }
        }

        if armor.is_empty() {
            None
        } else {
            Some(armor)
        }
    }

    /// Trade equipment
    pub fn trade_equipment(&mut self) -> bool {
        let /*mut*/ bailout = false;

        let weapon_count;
        let armor_count;

        let pur_weapons = self.purchasable_weapons();
        let pur_armor = self.purchasable_armor();

        if let Some(ref pw) = pur_weapons {
            weapon_count = pw.len() as i32;
        } else {
            weapon_count = 0i32;
        }

        if let Some(ref pa) = pur_armor {
            armor_count = pa.len() as i32;
        } else {
            armor_count = 0i32;
        }

        let lamp_count = if self.game.vendor_can_afford_lamp() && !self.game.player_has_lamp() {
            1
        } else {
            0
        };

        // Check and see if we can buy anything
        if lamp_count == 0 && armor_count == 0 && weapon_count == 0 {
            return false;
        }

        let mut height = 7 + armor_count + weapon_count + lamp_count;

        if armor_count > 0 {
            height += 1; // room for newline
        }

        if weapon_count > 0 {
            height += 1; // room for newline
        }

        if lamp_count > 0 {
            height += 1; // room for newline
        }

        let w = G::popup(height, 47);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "What equipment would you like to buy?\n");
        self.wcoff(w, G::A_TITLE());

        if let Some(ref pa) = pur_armor {
            wprintw(w, "\n");
            for a in pa {
                let name = match a {
                    ArmorType::Leather => "|[L]|eather",
                    ArmorType::Chainmail => "|[C]|hainmail",
                    ArmorType::Plate => "|[P]|late",
                    _ => panic!("armor shouldn't be None here"),
                };
                let cost = format!("{} GP", Armor::cost(*a, true));

                self.wprintw_center(w, &format!("{:<16}{:>8}\n", name, cost));
            }
        }

        if let Some(ref pw) = pur_weapons {
            wprintw(w, "\n");
            for weap in pw {
                let name = match weap {
                    WeaponType::Dagger => "|[D]|agger",
                    WeaponType::Mace => "|[M]|ace",
                    WeaponType::Sword => "|[S]|word",
                    _ => panic!("weapon shouldn't be None here"),
                };
                let cost = format!("{} GP", Weapon::cost(*weap, true));

                self.wprintw_center(w, &format!("{:<16}{:>8}\n", name, cost));
            }
        }

        if lamp_count > 0 {
            let name = "L|[a]|mp";
            let cost = format!("{} GP", Game::vendor_lamp_cost());

            wprintw(w, "\n");

            self.wprintw_center(w, &format!("{:<16}{:>8}\n", name, cost));
        }

        self.mvwprintw_center(w, height - 3, "|[N]|othing");

        box_(w, 0, 0);
        wrefresh(w);

        getch();

        G::popup_close(w);

        self.redraw_underwins();

        bailout
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

        //self.trade_potions();
    }
}
