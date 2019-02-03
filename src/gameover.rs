use crate::G;
use ncurses::*;

use wizardscastle::armor::ArmorType;
use wizardscastle::game::GameState;
use wizardscastle::player::Stat;
use wizardscastle::weapon::WeaponType;

use std::cmp;

impl G {
    /// Initial screen if dead
    fn dead1(&self) {
        let w = G::popup(9, 52);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(
            w,
            2,
            &format!(
                "A noble effort, oh formerly-living {}\n\n",
                self.race_name()
            ),
        );
        self.wcoff(w, G::A_TITLE());

        let mut cod = String::from("You died due to lack of ");

        if self.game.player_stat(Stat::Strength) == 0 {
            cod.push_str("strength.");
        } else if self.game.player_stat(Stat::Intelligence) == 0 {
            cod.push_str("intelligence.");
        } else if self.game.player_stat(Stat::Dexterity) == 0 {
            cod.push_str("dexterity.");
        }

        self.wprintw_center(w, &cod);

        wprintw(w, "\n\n");

        self.wprintw_center(w, "|[C]|ontinue");

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            if G::norm_key(getch()) == 'C' {
                break;
            }
        }

        G::popup_close(w);

        self.redraw_underwins();
    }

    /// Initial screen if exited
    fn exit1(&self, win: bool) {
        let width = if win { 50 } else { 53 };

        let w = G::popup(9, width);

        let title = if win {
            "A glorious victory!"
        } else {
            "A less than awe-inspiring defeat."
        };

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, title);
        self.wcoff(w, G::A_TITLE());

        let out = if win { "" } else { "out" };
        let punc = if win { "!" } else { "." };
        let copy = format!("You left the castle with{} the Orb of Zot{}\n\n", out, punc);

        wprintw(w, "\n\n");

        self.wprintw_center(w, &copy);

        self.wprintw_center(w, "|[C]|ontinue");

        box_(w, 0, 0);

        wrefresh(w);

        loop {
            if G::norm_key(getch()) == 'C' {
                break;
            }
        }

        G::popup_close(w);

        self.redraw_underwins();
    }

    /// Build final inventory equipment string
    fn final_inventory_equip(weapon: WeaponType, armor: ArmorType, lamp: bool) -> String {
        let mut strs = Vec::new();
        let mut first = true;
        let mut final_str = String::new();

        if weapon != WeaponType::None {
            strs.push(format!("a {}", G::weapon_name(weapon)));
        }

        if armor != ArmorType::None {
            let mut armor_str = G::armor_name(armor);

            if armor != ArmorType::Chainmail {
                armor_str.push_str(" armor");
            }

            strs.push(armor_str);
        }

        if lamp {
            strs.push(String::from("a lamp"));
        }

        for (i, s) in strs.iter().enumerate() {
            let last = i == strs.len() - 1;

            if first {
                final_str.push_str(&G::initial_upper(&s.to_lowercase()));
                first = false;
            } else {
                let and = if last { "and " } else { "" };
                let comma = if strs.len() > 2 { "," } else { "" };
                final_str.push_str(&format!("{} {}{}", comma, and, s.to_lowercase()));
            }
        }

        final_str
    }

    /// Show final inventory
    fn final_inventory(&self, dead: bool) -> bool {
        let weapon = self.game.player_weapon_type();
        let armor = self.game.player_armor_type();
        let lamp = self.game.player_has_lamp();

        let weapon_str = G::final_inventory_equip(weapon, armor, lamp);

        let has_outfit = weapon_str != ""; //weapon != WeaponType::None || armor != ArmorType::None || lamp;

        let treasures = self.game.player_get_treasures();

        let turns = self.game.turn();
        let turns_s = if *turns != 1 { "s" } else { "" };

        let turn_str = format!("And it took you {} turn{}!", turns, turns_s);

        let title_str = if dead {
            "When you died you had:"
        } else {
            "When you left the castle, you had:"
        };

        // width is max len of all weapon_str, title_str, and turn_str + 10
        let width = cmp::max(weapon_str.len(), cmp::max(turn_str.len(), title_str.len())) + 10;

        let mut height = 10;
        let mut add_height = 0;

        if !dead {
            add_height += 1;
        }
        if has_outfit {
            add_height += 1;
        }

        if self.game.player_has_runestaff() || self.game.player_has_orb_of_zot() {
            add_height += 1;
        }

        add_height += treasures.len();

        let flare_count = self.game.player_flares();

        if flare_count > 0 {
            add_height += 1;
        }

        let gp_count = self.game.player_gp();

        if gp_count > 0 {
            add_height += 1;
        }

        let has_nothing = add_height == 0;

        if has_nothing {
            add_height += 1;
        }

        height += add_height;

        let w = G::popup(height as i32, width as i32);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, title_str);
        self.wcoff(w, G::A_TITLE());

        wmove(w, 4, 0);

        if !dead {
            self.wprintw_center(w, "Your miserable life!\n");
        }

        for t in treasures {
            self.wprintw_center(w, &format!("The {}\n", G::treasure_name(t)));
        }

        if has_outfit {
            self.wprintw_center(w, &format!("{}\n", weapon_str));
        }

        if flare_count > 0 {
            self.wprintw_center(w, &format!("{} flares\n", flare_count));
        }

        if gp_count > 0 {
            self.wprintw_center(w, &format!("{} gold pieces\n", gp_count));
        }

        if self.game.player_has_runestaff() {
            self.wprintw_center(w, "The Runestaff\n");
        }

        if self.game.player_has_orb_of_zot() {
            self.wprintw_center(w, "The Orb of Zot!\n");
        }

        if has_nothing {
            self.wprintw_center(w, "Nothing!\n");
        }

        wprintw(w, "\n");

        self.wprintw_center(w, &format!("{}\n\n", turn_str));

        self.wprintw_center(w, "|[P]|lay again or |[Q]|uit");

        box_(w, 0, 0);

        wrefresh(w);

        let play_again = loop {
            match G::norm_key(getch()) {
                'P' => break true,
                'Q' => break false,
                _ => (),
            }
        };

        G::popup_close(w);

        self.redraw_underwins();

        play_again
    }

    /// Show the restart screen
    fn restart_screen(&self, play_again: bool) {
        let width = if play_again { 35 } else { 50 };

        let w = G::popup(7, width);

        self.wcon(w, G::A_TITLE());

        if play_again {
            self.mvwprintw_center(w, 2, &format!("Some {}s never learn!", self.race_name()));
        } else {
            self.mvwprintw_center(
                w,
                2,
                &format!("Maybe dumb {} not so dumb after all!", self.race_name()),
            );
        }

        self.wcoff(w, G::A_TITLE());

        wattr_on(w, A_REVERSE());

        if play_again {
            self.mvwprintw_center_notrim(w, 4, " Press any key to start ");
        } else {
            self.mvwprintw_center_notrim(w, 4, " Press any key to exit ");
        }

        wattr_off(w, A_REVERSE());

        box_(w, 0, 0);

        wrefresh(w);

        getch();

        G::popup_close(w);

        self.redraw_underwins();
    }

    /// Show gameover summary
    pub fn game_summary(&self) -> bool {
        let dead;

        match self.game.state() {
            GameState::Dead => {
                self.dead1();
                dead = true;
            }

            GameState::Exit => {
                let win = self.game.player_has_orb_of_zot();
                dead = false;

                self.exit1(win);
            }

            GameState::Quit => {
                dead = false;
                self.exit1(false);
            }

            any => panic!("unexpected game state at end {:#?}", any),
        }

        let play_again = self.final_inventory(dead);

        self.restart_screen(play_again);

        play_again
    }
}
