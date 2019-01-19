use crate::G;
use wizardscastle::armor::ArmorType;
use wizardscastle::monster::MonsterType;
use wizardscastle::player::{Race, Stat};
use wizardscastle::room::RoomType;
use wizardscastle::treasure::TreasureType;
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

    pub fn monster_name(m: MonsterType) -> String {
        match m {
            MonsterType::Kobold => String::from("kobold"),
            MonsterType::Orc => String::from("orc"),
            MonsterType::Wolf => String::from("wolf"),
            MonsterType::Goblin => String::from("goblin"),
            MonsterType::Ogre => String::from("ogre"),
            MonsterType::Troll => String::from("troll"),
            MonsterType::Bear => String::from("bear"),
            MonsterType::Minotaur => String::from("minotaur"),
            MonsterType::Gargoyle => String::from("gargoyle"),
            MonsterType::Chimera => String::from("chimera"),
            MonsterType::Balrog => String::from("balrog"),
            MonsterType::Dragon => String::from("dragon"),
            MonsterType::Vendor => String::from("vendor"),
        }
    }

    pub fn treasure_name(t: TreasureType) -> String {
        match t {
            TreasureType::RubyRed => String::from("Ruby Red"),
            TreasureType::NornStone => String::from("Norn Stone"),
            TreasureType::PalePearl => String::from("Pale Pearl"),
            TreasureType::OpalEye => String::from("Opal Eye"),
            TreasureType::GreenGem => String::from("Green Gem"),
            TreasureType::BlueFlame => String::from("Blue Flame"),
            TreasureType::Palantir => String::from("Palantir"),
            TreasureType::Silmaril => String::from("Silmaril"),
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

    pub fn room_name(r: &RoomType) -> String {
        match r {
            RoomType::Empty => String::from("an empty room"),
            RoomType::Entrance => String::from("the entrance"),
            RoomType::StairsDown => String::from("stairs going down"),
            RoomType::StairsUp => String::from("stairs going up"),
            RoomType::Gold => String::from("gold pieces"),
            RoomType::Pool => String::from("a pool"),
            RoomType::Chest => String::from("a chest"),
            RoomType::Flares => String::from("flares"),
            RoomType::Warp(_) => String::from("a warp"),
            RoomType::Sinkhole => String::from("a sinkhole"),
            RoomType::CrystalOrb => String::from("a crystal orb"),
            RoomType::Book => String::from("a book"),
            RoomType::Monster(m) => {
                let mon_str = G::monster_name(m.monster_type());
                format!("{} {}", G::get_article(&mon_str), mon_str)
            }
            RoomType::Treasure(t) => G::treasure_name(*t.treasure_type()).to_string(),
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

    // Return player race
    pub fn race_name(&self) -> &str {
        match self.game.player_race() {
            Race::Hobbit => "Hobbit",
            Race::Elf => "Elf",
            Race::Human => "Human",
            Race::Dwarf => "Dwarf",
        }
    }

    fn starts_with_vowel(s: &str) -> bool {
        if let Some(c) = String::from(s).to_uppercase().chars().next() {
            return c == 'A' || c == 'E' || c == 'I' || c == 'O' || c == 'U';
        }

        false
    }

    pub fn get_article(s: &str) -> String {
        if G::starts_with_vowel(s) {
            return String::from("an");
        }

        String::from("a")
    }

    pub fn initial_upper(s: &str) -> String {
        format!("{}{}", s.get(0..1).unwrap().to_uppercase(), s.get(1..).unwrap())
    }
}
