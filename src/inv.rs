use crate::G;
use ncurses::*;

impl G {
    pub fn show_inventory(&self) {
        let mut inv = Vec::new();

        for i in self.game.player_get_treasures() {
            inv.push(G::treasure_name(i));
        }

        inv.sort_unstable();

        let count = inv.len();

        let width = if count == 0 { 42 } else { 43 };
        let height = if count == 0 { 7 } else { (8 + count) as i32 };
        let title = if count == 0 {
            "You don't yet have any treasure."
        } else {
            "You have the following treasures:"
        };

        let w = G::popup(height, width);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, &format!("{}\n\n", title));
        self.wcoff(w, G::A_TITLE());

        for i in inv {
            self.wprintw_center_notrim(w, &format!("The {:<10}\n", i));
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
