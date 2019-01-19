use crate::G;
use ncurses::*;

impl G {
    /// Verify the player wants to quit
    ///
    /// Returns true if they really do
    pub fn help(&self) {
        let w = G::popup(13, 44);

        let strs = [
            "[N]orth      [D]own        [G]aze ",
            "[S]outh      [T]eleport    [D]rink",
            "[W]est       [M]ap         [O]pen ",
            "[E]ast       [L]amp        [H]elp ",
            "[U]p         [F]lare       [Q]uit ",
        ];

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "You can use the following commands");
        self.wcoff(w, G::A_TITLE());

        for (i, s) in strs.iter().enumerate() {
            self.mvwprintw_center(w, i as i32 + 4, s);
        }

        wattron(w, A_REVERSE());
        self.mvwprintw_center(w, 10, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);
        wrefresh(w);

        getch();

        G::popup_close(w);
    }
}
