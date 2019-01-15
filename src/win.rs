/// Windows-related functions
use crate::G;
use ncurses::*;

impl G {
    /// Look up a color attribute by the given name
    pub fn wcget(&self, c: &str) -> u32 {
        if let Some(attr) = self.color.get(c) {
            *attr
        } else {
            panic!("undeclared color {}", c);
        }
    }

    /// Set or clear output attributes on a window.
    ///
    /// See `new()` for a list of attributes.
    ///
    /// You probably want `wcon()` or `wcoff()` instead.
    pub fn wcset(&self, w: WINDOW, c: &str, on: bool) {
        let attr = self.wcget(c);

        if on {
            wattr_on(w, attr);
        } else {
            wattr_off(w, attr);
        }
    }

    /// Turn on an attribute by name.
    pub fn wcon(&self, w: WINDOW, c: &str) {
        self.wcset(w, c, true);
    }

    /// Turn off an attribute by name.
    pub fn wcoff(&self, w: WINDOW, c: &str) {
        self.wcset(w, c, false);
    }

    /// Print some text in the center of a window, ignoring leading and trailing
    /// whitespace.
    pub fn wprintw_center(&self, w: WINDOW, s: &str) {
        self.mvwprintw_center(w, getcury(w), s)
    }

    /// Print some text in the center of a window, ignoring leading and trailing
    /// whitespace, given a `y` position.
    ///
    /// If the string contains pipes like so:
    ///
    ///    "|Foo| bar baz |E|tc"
    ///
    /// The contents between the pipes will be shown in A_REVERSE.
    pub fn mvwprintw_center(&self, w: WINDOW, y: i32, s: &str) {
        self.mvwprintw_center_core(w, y, s, true)
    }

    /// Variant doesn't trim before centering
    pub fn mvwprintw_center_notrim(&self, w: WINDOW, y: i32, s: &str) {
        self.mvwprintw_center_core(w, y, s, false)
    }

    /// Base functionality
    fn mvwprintw_center_core(&self, w: WINDOW, y: i32, s: &str, trim: bool) {
        let ts = if trim { s.trim() } else { s };

        let mut len = 0;
        let mut has_markup = false;

        for c in ts.chars() {
            if c != '|' && c != '^' {
                len += 1;
            } else {
                has_markup = true;
            }
        }

        let x = (getmaxx(w) - len) / 2;

        wmove(w, y, x);

        let mut reversed = false;
        let mut red = false;

        if has_markup {
            wattr_off(w, A_REVERSE());
        }

        for c in s.chars() {
            if c == '|' {
                reversed = !reversed;

                if reversed {
                    wattr_on(w, A_REVERSE());
                } else {
                    wattr_off(w, A_REVERSE());
                }
                continue;
            }

            if c == '^' {
                red = !red;

                if red {
                    self.wcon(w, "bold-red");
                } else {
                    self.wcoff(w, "bold-red");
                }
                continue;
            }

            wprintw(w, &format!("{}", c));
        }
    }

    /// Pop-up a new window of a given size with a border.
    pub fn popup(lines: i32, cols: i32) -> WINDOW {
        let mut y;
        let x = (COLS() - cols) / 2;

        // Aesthetic positioning logic
        if LINES() <= 25 && lines >= 12 {
            y = (LINES() as f32 / 2.5) as i32 - lines / 2;
        } else {
            y = LINES() / 4 - lines / 2;
        }

        if y < 0 {
            y = 0;
        }

        newwin(lines, cols, y, x)
    }

    /// Close a popup
    pub fn popup_close(w: WINDOW) {
        wclear(w);
        wrefresh(w);
        delwin(w);
    }

    /// Show or hide the cursor.
    pub fn show_cursor(show: bool) {
        if show {
            curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        } else {
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        }
    }

    /*
    /// Clear the border
    pub fn clear_border(w: WINDOW) {
        let ch = ' ' as chtype;
        wborder(w, ch, ch, ch, ch, ch, ch, ch, ch);
    }
    */

    pub fn popup_error(&self, s: &str) {
        let mut width = s.len() as i32 + 10;

        width += width % 2; // Force to even width

        let w = G::popup(9, width);

        self.wcon(w, "bold-red");
        self.mvwprintw_center(w, 2, &format!("** SILLY {} **", self.player_race_name()));
        self.wcoff(w, "bold-red");

        self.mvwprintw_center(w, 4, s);

        wattron(w, A_REVERSE());
        self.mvwprintw_center(w, 6, " Press any key ");
        wattroff(w, A_REVERSE());

        self.wcon(w, "bold-red");
        wattr_on(w, A_REVERSE());
        box_(w, 0, 0);
        wattr_off(w, A_REVERSE());
        self.wcoff(w, "bold-red");

        wrefresh(w);

        getch();

        G::popup_close(w);
    }
}
