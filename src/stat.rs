use crate::G;
use ncurses::*;

use wizardscastle::player::Stat;

impl G {
    pub fn update_stat(&self) {
        //wclear(self.statwin); // If we don't clear first, nothing redraws...??

        // Draw stats
        G::mvwprintw_center(
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
        G::mvwprintw_center(
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

        G::mvwprintw_center(self.statwin, 4, &inv);

        box_(self.statwin, 0, 0);

        wrefresh(self.statwin);
    }
}
