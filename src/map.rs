use crate::G;
use ncurses::*;

impl G {
    pub fn update_map(&self, show_all: bool) {
        let z = self.game.player_z();

        werase(self.mapwin);

        let _ = waddstr(self.mapwin, "\n ");

        for y in 0..self.game.dungeon_ysize() {
            for x in 0..self.game.dungeon_xsize() {
                if x >= 1 {
                    let _ = waddstr(self.mapwin, "   ");
                }

                let r = self.game.dungeon_room_at(x, y, z);

                let bracket = x == self.game.player_x() && y == self.game.player_y();

                if self.game.player_is_blind() {
                    let _ = waddstr(self.mapwin, " - ");
                } else {
                    if bracket {
                        wattr_on(self.mapwin, A_BOLD);
                        let _ = waddstr(self.mapwin, "<");
                        wattr_off(self.mapwin, A_BOLD);
                    } else {
                        let _ = waddstr(self.mapwin, " ");
                    }

                    if r.discovered || show_all {
                        let room_ch = G::room_char(&r.roomtype);
                        let attr_str = format!("room-{}", room_ch);

                        self.wcon(self.mapwin, &attr_str);
                        let _ = waddstr(self.mapwin, &format!("{}", room_ch));
                        self.wcoff(self.mapwin, &attr_str);
                    } else {
                        wattr_on(self.mapwin, A_DIM);
                        let _ = waddstr(self.mapwin, "?");
                        wattr_off(self.mapwin, A_DIM);
                    }

                    if bracket {
                        wattr_on(self.mapwin, A_BOLD);
                        let _ = waddstr(self.mapwin, ">");
                        wattr_off(self.mapwin, A_BOLD);
                    } else {
                        let _ = waddstr(self.mapwin, " ");
                    }
                }
            }

            let _ = waddstr(self.mapwin, "\n\n ");
        }

        box_(self.mapwin, 0, 0);

        wrefresh(self.mapwin);
    }
}
