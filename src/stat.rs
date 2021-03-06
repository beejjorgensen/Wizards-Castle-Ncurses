use crate::G;
use ncurses::*;

use wizardscastle::monster::MonsterType;
use wizardscastle::player::Stat;
use wizardscastle::room::RoomType;

#[derive(Debug, Clone, Copy)]
pub enum StatMode {
    None,
    Lamp,
    Vendor,
    Pool,
    Book,
    Chest,
    StairsUp,
    StairsDown,
    CrystalOrb,
    Combat,
    Spell,
}

impl G {
    pub fn set_statmode(&mut self, mode: StatMode) {
        self.statmode = mode;

        self.update_stat();
    }

    pub fn update_stat(&self) {
        wclear(self.statwin);
        let player_has_runestaff = self.game.player_has_runestaff();
        let player_has_orb_of_zot = self.game.player_has_orb_of_zot();
        let player_has_magic_item = player_has_runestaff || player_has_orb_of_zot;
        let spacing = if player_has_magic_item { "  " } else { "   " };

        // Draw stats
        let mut stat_str = format!(
            "ST:{:<}{}IQ:{:<}{}DX:{:<}",
            self.game.player_stat(Stat::Strength),
            spacing,
            self.game.player_stat(Stat::Intelligence),
            spacing,
            self.game.player_stat(Stat::Dexterity)
        );

        if self.game.player_has_runestaff() {
            stat_str.push_str("  %YR%Y");
        }
        if self.game.player_has_orb_of_zot() {
            stat_str.push_str("  %YZ%Y");
        }

        self.mvwprintw_center(self.statwin, 2, &stat_str);

        // Gold and flares
        self.mvwprintw_center(
            self.statwin,
            3,
            &format!(
                "GP:{:<}  FL:{:<}  T:{:<}",
                self.game.player_gp(),
                self.game.player_flares(),
                self.game.turn()
            ),
        );

        // Inventory
        let mut inv = format!(
            "{}, {}",
            G::armor_name(self.game.player_armor_type()),
            G::weapon_name(self.game.player_weapon_type())
        );

        if self.game.player_has_lamp() {
            inv.push_str(", Lamp");
        }

        self.mvwprintw_center(self.statwin, 4, &inv);

        // Print the room location
        if self.game.player_is_blind() {
            self.wcon(self.statwin, "red");
            self.mvwprintw_center(self.statwin, 6, "** BLIND **");
            self.wcoff(self.statwin, "red");
        } else {
            self.mvwprintw_center(
                self.statwin,
                6,
                &format!(
                    "({},{}) level {}",
                    self.game.player_x() + 1,
                    self.game.player_y() + 1,
                    self.game.player_z() + 1
                ),
            );
        }

        // Print the room contents
        let room = self.game.room_at_player();

        let combat_room_desc = if let RoomType::Monster(m) = room.room_type() {
            // "You're facing A MONSTER"
            (m.monster_type() == MonsterType::Vendor && self.game.vendors_angry())
                || m.monster_type() != MonsterType::Vendor
        } else {
            // Just a regular room
            false
        };

        if combat_room_desc && !self.game.player_bribed_this_monster() {
            self.mvwprintw_center(
                self.statwin,
                8,
                &format!(
                    "You're facing {}!",
                    G::room_name(room.room_type()).to_uppercase()
                ),
            );
        } else {
            self.mvwprintw_center(
                self.statwin,
                8,
                //&G::initial_upper(&G::room_name(room.room_type())).to_string(),
                &format!("You find {}", G::room_name(room.room_type())),
            );
        }

        // Additional status info
        self.update_stat_additional();

        box_(self.statwin, 0, 0);

        wrefresh(self.statwin);
    }

    /// Update additional status info
    fn update_stat_additional(&self) {
        match self.statmode {
            StatMode::None => (),
            StatMode::Lamp => {
                self.mvwprintw_center(self.statwin, 10, "Shine lamp which way?");
                self.mvwprintw_center(self.statwin, 12, "|[N]|");
                self.mvwprintw_center(self.statwin, 13, "|[W]|   |[E]|");
                self.mvwprintw_center(self.statwin, 14, "|[S]|");
            }
            StatMode::Vendor => {
                self.mvwprintw_center(self.statwin, 10, "|[T]|rade\n");
                self.wprintw_center(self.statwin, "|[A]|ttack");
            }
            StatMode::Combat => {
                self.mvwprintw_center(self.statwin, 10, "|[A]|ttack\n");
                if self.game.bribe_possible() && self.game.player_has_any_treasure() {
                    self.wprintw_center(self.statwin, "|[B]|ribe\n");
                }
                if self.game.spell_possible() {
                    self.wprintw_center(self.statwin, "|[C]|ast spell\n");
                }
                self.wprintw_center(self.statwin, "|[N]||[S]||[W]||[E]| to retreat\n");
            }
            StatMode::Spell => {
                self.mvwprintw_center(self.statwin, 10, "|[W]|eb\n");
                self.wprintw_center(self.statwin, "|[F]|ireball\n");
                self.wprintw_center(self.statwin, "|[D]|eathspell\n\n");
                self.wprintw_center(self.statwin, "|[N]|othing\n\n");
            }
            StatMode::Pool => {
                self.mvwprintw_center(self.statwin, 10, "|[D]|rink");
            }
            StatMode::StairsUp => {
                self.mvwprintw_center(self.statwin, 10, "|[U]|p");
            }
            StatMode::StairsDown => {
                self.mvwprintw_center(self.statwin, 10, "|[D]|own");
            }
            StatMode::Book | StatMode::Chest => {
                self.mvwprintw_center(self.statwin, 10, "|[O]|pen");
            }
            StatMode::CrystalOrb => {
                self.mvwprintw_center(self.statwin, 10, "|[G]|aze");
            }
        }
    }
}
