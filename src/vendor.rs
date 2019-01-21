use crate::G;
use ncurses::*;

use wizardscastle::treasure::TreasureType;

impl G {
    /// Helper function to print gold pieces
    fn print_gps(&self, w: WINDOW) {
        let gps = self.game.player_gp();

        let unit = if gps == 1 { "GP" } else { "GPs" };

        self.wprintw_center(w, &format!("You have {} {}.\n\n", gps, unit));
    }

    /// Sell treasures
    ///
    /// Return true if the player hit ESC and wants to bail from the vendor
    fn vendor_trade_treasure(&mut self) -> bool {
        let treasures = self.game.player_get_treasures();

        if treasures.is_empty() {
            return false;
        }

        let pricemap = match self.game.vendor_treasure_offer() {
            Ok(p) => p,
            Err(err) => panic!(err),
        };

        let height = (10 + treasures.len()) as i32;
        let w = G::popup(height, 40);

        let mut sold: Vec<TreasureType> = Vec::new();

        let mut done = false;
        let mut bailout = false;

        while !done {
            wclear(w);

            self.wcon(w, G::A_TITLE());
            self.mvwprintw_center(w, 2, "What do you want to sell?\n\n");
            self.wcoff(w, G::A_TITLE());

            self.print_gps(w);

            for (i, t) in treasures.iter().enumerate() {
                let price = match pricemap.get(&t) {
                    Some(p) => p,
                    None => panic!("price missing"),
                };

                let gps = format!("{} GP", price);

                let tname = if sold.contains(t) {
                    self.wcon(w, "dim-green");
                    format!("||    The {}", G::treasure_name(*t))
                } else {
                    format!("|[{}]| The {}", i + 1, G::treasure_name(*t))
                };

                self.wprintw_center_notrim(w, &format!("{:<22}{:>8}\n", tname, gps));

                wattr_set(w, 0, 0);
            }

            self.mvwprintw_center(w, height - 3, "|[N]|othing");

            box_(w, 0, 0);
            wrefresh(w);

            let ch = getch();

            if ch == 27 {
                done = true;
                bailout = true;
            }

            let nch = G::norm_key(ch);
            match nch {
                '1'...'9' => {
                    let index = (nch.to_digit(10).unwrap() - 1) as usize;
                    if index < treasures.len() {
                        let tt = treasures[index];
                        if !sold.contains(&tt) {
                            if let Err(err) = self.game.vendor_treasure_accept(tt) {
                                panic!(err);
                            }
                            sold.push(tt);
                        }
                    }
                }
                'N' => done = true,
                _ => (),
            }
        }

        G::popup_close(w);

        bailout
    }

    /// Trade with a Vendor
    pub fn vendor_trade(&mut self) {
        if self.vendor_trade_treasure() {
            return;
        }
        //self.trade_equipment();
        //self.trade_potions();
    }
}
