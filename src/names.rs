use crate::G;
use wizardscastle::player::{Race, Stat};

impl G {
    pub fn player_race_name(&self) -> &str {
        match self.game.player_race() {
            Race::Hobbit => "Hobbit",
            Race::Elf => "Elf",
            Race::Human => "Human",
            Race::Dwarf => "Dwarf",
        }
    }

    pub fn stat_name(s: Stat) -> String {
        match s {
            Stat::Strength => String::from("Strength"),
            Stat::Intelligence => String::from("Intelligence"),
            Stat::Dexterity => String::from("Dexterity"),
        }
    }

    /*
    pub fn gender_name(g: Gender) -> String {
        match g {
            Gender::Female => String::from("FEMALE"),
            Gender::Male => String::from("MALE"),
        }
    }
    */
}

