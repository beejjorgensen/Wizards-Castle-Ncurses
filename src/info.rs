use crate::G;
use ncurses::*;

impl G {
    /// Show info
    pub fn info(&self) {
        let w = G::popup(15, 62);

        self.wcon(w, G::A_TITLE());
        self.mvwprintw_center(w, 2, "** WIZARD'S CASTLE **\n\n");
        self.wcoff(w, G::A_TITLE());

        self.wprintw_center(w, "Originally written by Joseph R. Power\n");
        self.wprintw_center(w, "Recreational Computing Magazine, July 1980\n\n");

        self.wprintw_center(w, "Rust/ncurses port released under the MIT license\n");
        self.wprintw_center(w, "Brian \"Beej Jorgensen\" Hall <beej@beej.us>\n\n");

        self.wprintw_center(
            w,
            "https://github.com/beejjorgensen/Wizards-Castle-Info\n\n",
        );

        wattron(w, A_REVERSE());
        self.wprintw_center_notrim(w, " Press any key ");
        wattroff(w, A_REVERSE());

        box_(w, 0, 0);
        wrefresh(w);

        getch();

        G::popup_close(w);

        self.redraw_underwins();
    }
}
