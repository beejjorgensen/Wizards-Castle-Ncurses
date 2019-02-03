use crate::G;
use ncurses::*;

impl G {
    pub fn show_inventory(&self) {
        let mut inv = Vec::new();

        for i in self.game.player_get_treasures() {
            inv.push(G::treasure_name(i));
        }

        inv.sort_unstable();

        let player_has_runestaff = self.game.player_has_runestaff();
        let player_has_orb_of_zot = self.game.player_has_orb_of_zot();
        let player_has_magic_item = player_has_runestaff || player_has_orb_of_zot;

        let mut magic_item = "";

        if player_has_runestaff {
            magic_item = "Runestaff";
        } else if player_has_orb_of_zot {
            magic_item = "Orb of Zot!";
        };

        let count = inv.len() + if player_has_magic_item { 1 } else { 0 };

        let width = if count == 0 { 42 } else { 39 };
        let height = if count == 0 { 7 } else { (8 + count) as i32 };

        let title = if count == 0 && !player_has_magic_item {
            "You don't yet have any treasure."
        } else {
            "You have the following items:"
        };

        let w = G::popup(height, width);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, &format!("{}\n\n", title));
        self.wcoff(w, G::A_TITLE());

        for i in inv {
            self.wprintw_center_notrim(w, &format!("The {:<11}", i));
            wprintw(w, "\n");
        }

        if player_has_magic_item {
            wattr_on(w, A_BOLD());
            self.wprintw_center_notrim(w, &format!("The {:<11}", magic_item));
            wprintw(w, "\n");
            wattr_off(w, A_BOLD());
        }

        wattron(w, A_REVERSE());
        self.mvwprintw_center_notrim(w, height - 3, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);
        wrefresh(w);

        getch();

        G::popup_close(w);
    }
}
