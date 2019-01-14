use crate::G;
use ncurses::*;

impl G {
    pub fn update_stat(&self) {
        wclear(self.statwin); // If we don't clear first, nothing redraws...??

        box_(self.statwin, 0, 0);

        wrefresh(self.statwin);
    }
}
