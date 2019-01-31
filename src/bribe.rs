use crate::G;
use ncurses::*;

impl G {
    /// Bribe a monster
    pub fn combat_bribe(&mut self) -> bool {
        let mut bribed = false;

        let w = G::popup(9, 33);

        if let Ok(Some(t_type)) = self.game.bribe_proposition() {
            let tname = G::treasure_name(t_type);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, &format!("I want The {}\n\n", &tname));
            self.wcoff(w, G::A_TITLE());

            self.wprintw_center(w, "Will you give it to me?\n\n");

            self.wprintw_center(w, "|[Y]|es   |[N]|o");

            box_(w, 0, 0);

            wrefresh(w);

            let mut done = false;

            while !done {
                match G::norm_key(getch()) {
                    'Y' => {
                        match self.game.bribe_accept() {
                            Ok(_) => {
                                bribed = true;
                            }
                            Err(err) => {
                                panic!("agree to bribe: {:#?}", err);
                            }
                        };
                        done = true;
                    }

                    'N' => {
                        match self.game.bribe_decline() {
                            Ok(_) => {
                                bribed = false;
                            }
                            Err(err) => {
                                panic!("disagree to bribe: {:#?}", err);
                            }
                        };
                        done = true;
                    }

                    _ => (),
                }
            }
        } else {
            panic!("We should have been able to bribe at this point");
        }

        G::popup_close(w);

        self.redraw_underwins();

        if bribed {
            let w = G::popup(7, 37);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "Ok, just don't tell anyone.\n\n");
            self.wcoff(w, G::A_TITLE());

            wattr_on(w, A_REVERSE());
            self.wprintw_center_notrim(w, " Press any key ");
            wattr_off(w, A_REVERSE());

            box_(w, 0, 0);

            wrefresh(w);

            getch();

            G::popup_close(w);

            self.redraw_underwins();
        }

        bribed
    }
}
