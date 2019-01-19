use crate::G;
use ncurses::*;

impl G {
    /// Verify the player wants to quit
    ///
    /// Returns true if they really do
    pub fn verify_quit(&self, exiting: bool) -> bool {
        let s = if exiting {
            "Do you really want to exit the dungeon?"
        } else {
            "Do you really want to quit?"
        };

        let width = s.len() + 10;

        let w = G::popup(7, width as i32);

        self.wcon(w, "bold-yellow");
        self.mvwprintw_center(w, 2, s);
        self.wcoff(w, "bold-yellow");

        wattr_on(w, A_BOLD());
        self.mvwprintw_center(w, 4, "|[Y]|es or |[N]|o");
        wattr_off(w, A_BOLD());

        box_(w, 0, 0);
        wrefresh(w);

        let key = getch();

        G::popup_close(w);

        G::norm_key(key) == 'Y'
    }
}
