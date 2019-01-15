use crate::G;
use wizardscastle::armor::ArmorType;
use wizardscastle::monster::MonsterType;
use wizardscastle::player::{Race, Stat};
use wizardscastle::room::RoomType;
use wizardscastle::weapon::WeaponType;

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

    pub fn weapon_name(w: WeaponType) -> String {
        match w {
            WeaponType::None => String::from("No weapon"),
            WeaponType::Dagger => String::from("Dagger"),
            WeaponType::Mace => String::from("Mace"),
            WeaponType::Sword => String::from("Sword"),
        }
    }

    pub fn armor_name(a: ArmorType) -> String {
        match a {
            ArmorType::None => String::from("No armor"),
            ArmorType::Leather => String::from("Leather"),
            ArmorType::Chainmail => String::from("Chainmail"),
            ArmorType::Plate => String::from("Plate"),
        }
    }

    /// Get the printable character for a room
    pub fn room_char(room_type: &RoomType) -> char {
        match room_type {
            RoomType::Empty => '.',
            RoomType::Entrance => 'E',
            RoomType::StairsDown => 'D',
            RoomType::StairsUp => 'U',
            RoomType::Gold => 'G',
            RoomType::Pool => 'P',
            RoomType::Chest => 'C',
            RoomType::Flares => 'F',
            RoomType::Warp(_) => 'W',
            RoomType::Sinkhole => 'S',
            RoomType::CrystalOrb => 'O',
            RoomType::Book => 'B',
            RoomType::Monster(ref m) => {
                if m.monster_type() == MonsterType::Vendor {
                    'V'
                } else {
                    'M'
                }
            }
            RoomType::Treasure(_) => 'T',
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
