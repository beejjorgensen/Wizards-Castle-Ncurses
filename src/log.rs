use crate::G;
use ncurses::*;

impl G {
    #[allow(non_snake_case)]
    fn A_LOG_GOOD() -> &'static str {
        "bold-yellow"
    }

    #[allow(non_snake_case)]
    fn A_LOG_ERROR() -> &'static str {
        "bold-red"
    }

    pub fn update_log(&self, s: &str) {
        self.update_log_attr(s, 0)
    }

    pub fn update_log_good(&self, s: &str) {
        self.update_log_attr(&s, self.wcget(G::A_LOG_GOOD()));
    }

    pub fn update_log_bad(&self, s: &str) {
        self.update_log_error(s)
    }

    pub fn update_log_error(&self, s: &str) {
        self.update_log_attr(&s, self.wcget(G::A_LOG_ERROR()));
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
