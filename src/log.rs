use crate::G;
use ncurses::*;

impl G {
    pub fn update_log(&self, s: &str) {
        self.update_log_attr(s, 0)
    }

    pub fn update_log_attr(&self, s: &str, attr: u32) {
        wattr_on(self.loginner, attr);
        wprintw(self.loginner, &format!("\n{}", s));
        wattr_off(self.loginner, attr);

        box_(self.logwin, 0, 0);

        wrefresh(self.loginner);
        wrefresh(self.logwin);
    }
}
