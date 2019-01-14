use crate::G;
use ncurses::*;

impl G {
    pub fn update_map(&self, show_all: bool) {
        let z = self.game.player_z();

        wprintw(self.mapwin, "\n ");

        for y in 0..self.game.dungeon_ysize() {
            for x in 0..self.game.dungeon_xsize() {
                if x >= 1 {
                    wprintw(self.mapwin, "   ");
                }

                let r = self.game.dungeon_room_at(x, y, z);

                let bracket = x == self.game.player_x() && y == self.game.player_y();

                if self.game.player_is_blind() {
                } else {
                    if bracket { 
                        wattr_on(self.mapwin, A_BOLD());
                        wprintw(self.mapwin, "<");
                        wattr_off(self.mapwin, A_BOLD());
                    } else {
                        wprintw(self.mapwin, " ");
                    }

                    if r.discovered || show_all {
                        wprintw(self.mapwin, &format!("{}", G::room_char(&r.roomtype)));
                    } else {
                        wprintw(self.mapwin, "?");
                    }

                    if bracket {
                        wattr_on(self.mapwin, A_BOLD());
                        wprintw(self.mapwin, ">");
                        wattr_off(self.mapwin, A_BOLD());
                    } else {
                        wprintw(self.mapwin, " ");
                    }
                }
            }

            wprintw(self.mapwin, "\n\n ");
        }

        box_(self.mapwin, 0, 0);
        wrefresh(self.mapwin);
    }
}
