use crate::G;
use ncurses::*;

impl G {
    /// Verify the player wants to quit
    ///
    /// Returns true if they really do
    pub fn help(&self) {
        let w = G::popup(14, 44);

        let strs = [
            "[N]orth     [T]rade       [R]ead ",
            "[S]outh     [A]ttack      [D]rink",
            "[W]est      [L]amp        [O]pen ",
            "[E]ast      [F]lare       [G]aze ",
            "[U]p        [I]nventory   [H]elp ",
            "[D]own      [T]ele[p]ort  [Q]uit ",
        ];

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "You can use the following commands");
        self.wcoff(w, G::A_TITLE());

        for (i, s) in strs.iter().enumerate() {
            self.mvwprintw_center_notrim(w, i as i32 + 4, s);
        }

        wattron(w, A_REVERSE());
        self.mvwprintw_center_notrim(w, 11, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);
        wrefresh(w);

        getch();

        G::popup_close(w);

        self.redraw_underwins();
    }
}
