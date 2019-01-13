/// Windows-related functions

use crate::G;
use ncurses::*;

impl G {
    /// Set or clear output attributes on a window.
    /// 
    /// See `new()` for a list of attributes.
    /// 
    /// You probably want `wcon()` or `wcoff()` instead.
    pub fn wcset(&self, w: WINDOW, c: &str, on: bool) {
        if let Some(attr) = self.color.get(c) {
            if on {
                wattr_on(w, *attr);
            } else {
                wattr_off(w, *attr);
            }
        } else {
            panic!("undeclared color {}", c);
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
    pub fn wprintw_center(w: WINDOW, s: &str) -> i32{
        G::mvwprintw_center(w, getcury(w), s)
    }

    /// Print some text in the center of a window, ignoring leading and trailing
    /// whitespace, given a `y` position.
    pub fn mvwprintw_center(w: WINDOW, y: i32, s: &str) -> i32 {
        let x = (getmaxx(w) - s.trim().len() as i32) / 2;

        mvwprintw(w, y, x, s)
    }

    /// Pop-up a new window of a given size with a border.
    pub fn newwin_popup(lines: i32, cols: i32) -> WINDOW {
        
        let mut y = LINES() / 4 - lines / 2;
        let x = (COLS() - cols) / 2;

        if y < 0 {
            y = 0;
        }

        let w = newwin(lines, cols, y, x);
        box_(w, 0, 0);

        w
    }

    /// Show or hide the cursor.
    pub fn show_cursor(show: bool) {
        if show {
            curs_set(CURSOR_VISIBILITY::CURSOR_VISIBLE);
        } else {
            curs_set(CURSOR_VISIBILITY::CURSOR_INVISIBLE);
        }
    }
}