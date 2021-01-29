use crate::G;
use ncurses::*;
use std::cmp::Ordering;

impl G {
    /// Print a teleport error message
    pub fn teleport_error(&self) {
        self.update_log_error("** You can't teleport without the Runestaff!");
    }

    /// Teleport
    pub fn teleport(&mut self) {
        if !self.game.can_teleport() {
            self.teleport_error();
            return;
        }

        let mut done = false;
        let mut state = 0;
        let mut coord = [0, 0, 0];

        let w = G::popup(12, 28);

        while !done {
            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "Teleport to where?");
            self.wcoff(w, G::A_TITLE());

            mvwprintw(w, 4, 9, "X coord:");
            mvwprintw(w, 5, 9, "Y coord:");
            mvwprintw(w, 6, 9, "Z coord:");

            for i in 0..3 {
                match i.cmp(&state) {
                    Ordering::Less => {
                        wattroff(w, A_REVERSE());
                        mvwprintw(w, 4 + i, 18, &format!("{}", coord[i as usize] + 1));
                    },
                    Ordering::Equal => {
                        wattron(w, A_REVERSE());
                        mvwprintw(w, 4 + i, 18, " ");
                        wattroff(w, A_REVERSE());
                    }
                    _ => (),
                }
            }

            self.mvwprintw_center(w, 8, "Choose |[1]|-|[8]|\n");
            self.wprintw_center(w, "or |[N]|evermind");

            box_(w, 0, 0);

            wrefresh(w);

            let ch = G::norm_key(getch());
            match ch {
                '1'..='8' => {
                    let v = ch.to_digit(10).unwrap();

                    coord[state as usize] = v - 1;
                    state += 1;

                    if state == 3 {
                        match self.game.teleport(coord[0], coord[1], coord[2]) {
                            Ok(found_orb_of_zot) => {
                                if found_orb_of_zot {
                                    self.update_log_good("GREAT UNMITIGATED ZOT!");
                                    self.update_log_good("** YOU JUST FOUND THE ORB OF ZOT! **");
                                    self.update_log("The Runestaff is gone.");
                                }
                            }
                            Err(err) => panic!("{:#?}", err),
                        }

                        done = true;
                    }
                }
                'N' => done = true,
                _ => (),
            }
        }

        G::popup_close(w);

        self.redraw_underwins();
    }
}
