use crate::G;
use ncurses::*;

use wizardscastle::player::Stat;

impl G {
    pub fn update_stat(&self) {
        wclear(self.statwin);

        // Draw stats
        self.mvwprintw_center(
            self.statwin,
            2,
            &format!(
                "ST:{:<}   IQ:{:<}   DX:{:<}",
                self.game.player_stat(Stat::Strength),
                self.game.player_stat(Stat::Intelligence),
                self.game.player_stat(Stat::Dexterity)
            ),
        );

        // Gold and flares
        self.mvwprintw_center(
            self.statwin,
            3,
            &format!(
                "GP:{:<}   Flares:{:<}",
                self.game.player_gp(),
                self.game.player_flares()
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
        self.mvwprintw_center(self.statwin, 8, &G::room_name(room.room_type()));

        box_(self.statwin, 0, 0);

        wrefresh(self.statwin);
    }
}
