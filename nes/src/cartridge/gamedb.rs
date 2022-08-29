use std::str::FromStr;

use roxmltree::Node;
use sha1::{Digest, Sha1};

use crate::{cartridge::Header, NesError};

use super::HeaderSource;

const GAMEDB: &str = include_str!("nes20db.xml");

impl Header {
    pub fn from_prg_chr(prg: &[u8], chr: Option<&[u8]>) -> Result<Option<Self>, NesError> {
        let prg_sha1 = Self::arr_to_hex(Sha1::digest(prg).as_slice());
        let chr_sha1 = chr.map(|chr| Self::arr_to_hex(Sha1::digest(chr).as_slice()));

        Self::find_game(&prg_sha1, chr_sha1.as_deref())
    }

    fn arr_to_hex(arr: &[u8]) -> String {
        let mut res = String::new();

        let nibble_to_hex = |nibble| match nibble {
            0 => '0',
            1 => '1',
            2 => '2',
            3 => '3',
            4 => '4',
            5 => '5',
            6 => '6',
            7 => '7',
            8 => '8',
            9 => '9',
            10 => 'A',
            11 => 'B',
            12 => 'C',
            13 => 'D',
            14 => 'E',
            15 => 'F',
            _ => unreachable!(),
        };

        for num in arr.iter() {
            res.push(nibble_to_hex((num & 0xF0) >> 4));
            res.push(nibble_to_hex(num & 0xF));
        }

        res
    }

    // TODO: maybe verify crc32 too ?
    /** Find a game based on its PRG ROM sha1 hash, and optionally the CHR ROM sha1 hash.
    We need to check the console type too, since two roms with the same hashes can exist for
    both the standard NES and the VS system / PlayChoice **/
    pub fn find_game(prg_sha1: &str, chr_sha1: Option<&str>) -> Result<Option<Self>, NesError> {
        let doc = roxmltree::Document::parse(GAMEDB).map_err(|_| NesError::GameDbFormat)?;

        for game in doc.descendants().filter(|n| n.tag_name().name() == "game") {
            let chr_mmatches = match chr_sha1 {
                Some(chr_sha1) => game.children().any(|n| {
                    n.tag_name().name() == "chrrom" && n.attribute("sha1") == Some(chr_sha1)
                }),
                None => true,
            };

            let is_normal_nes = game
                .children()
                .any(|n| n.tag_name().name() == "console" && n.attribute("type") == Some("0"));

            if game
                .children()
                .any(|n| n.tag_name().name() == "prgrom" && n.attribute("sha1") == Some(prg_sha1))
                && chr_mmatches
                && is_normal_nes
            {
                return Self::parse_game_info(game).map(|gi| Some(gi));
            }
        }

        Ok(None)
    }

    pub fn parse_game_info(game: roxmltree::Node) -> Result<Header, NesError> {
        let name_comment = game
            .children()
            .find(|n| n.is_comment())
            .ok_or(NesError::GameDbFormat)?
            .text()
            .ok_or(NesError::GameDbFormat)?;

        let name = String::from(
            name_comment
                .split_once('\\')
                .ok_or(NesError::GameDbFormat)?
                .1
                .trim_end_matches(".nes"),
        );

        let pcb = Self::find_child_tag(game, "pcb").ok_or(NesError::GameDbFormat)?;

        let prg_rom_size = Self::find_child_tag(game, "prgrom").ok_or(NesError::GameDbFormat)?;
        let prg_rom_size = Self::parse_attr_required::<u32>(prg_rom_size, "size")?;
        let chr_rom_size = Self::find_child_tag(game, "chrrom");
        let chr_rom_size = Self::parse_attr(chr_rom_size, "size")?;
        let chr_ram_size = Self::find_child_tag(game, "chrram");
        let chr_ram_size = Self::parse_attr(chr_ram_size, "size")?;
        let prg_ram_size = Self::find_child_tag(game, "prgram");
        let prg_ram_size = Self::parse_attr(prg_ram_size, "size")?;
        let prg_nvram_size = Self::find_child_tag(game, "prgnvram");
        let prg_nvram_size = Self::parse_attr(prg_nvram_size, "size")?;

        let mapper = Self::parse_attr_required(pcb, "mapper")?;
        let submapper = Self::parse_attr_required(pcb, "submapper")?;
        let mirroring = Self::parse_attr_required(pcb, "mirroring")?;
        let battery = Self::parse_attr_required::<u32>(pcb, "battery")? != 0;

        let console = Self::find_child_tag(game, "console").ok_or(NesError::GameDbFormat)?;
        let console_typ = Self::parse_attr_required(console, "type")?;
        let region = Self::parse_attr_required::<super::Region>(console, "region")?;

        let expansion = Self::find_child_tag(game, "expansion").ok_or(NesError::GameDbFormat)?;
        let expansion = Self::parse_attr_required(expansion, "type")?;

        Ok(Header {
            source: HeaderSource::GameDb,

            name,

            prg_rom_size,
            chr_rom_size,

            chr_ram_size,
            prg_ram_size,
            prg_nvram_size,

            mapper,
            submapper,
            mirroring,
            battery,

            console_typ,
            region,
            expansion,
        })
    }

    fn find_child_tag<'n>(node: Node<'n, '_>, name: &str) -> Option<Node<'n, 'n>> {
        node.children().find(|n| n.tag_name().name() == name)
    }

    fn parse_attr_required<T: FromStr>(node: Node, attr: &str) -> Result<T, NesError> {
        node.attribute(attr)
            .ok_or(NesError::GameDbFormat)?
            .parse::<T>()
            .map_err(|_| NesError::GameDbFormat)
    }

    fn parse_attr<T: FromStr>(node: Option<Node>, attr: &str) -> Result<Option<T>, NesError> {
        node.and_then(|chr| chr.attribute(attr))
            .map(|size| size.parse::<T>())
            .transpose()
            .map_err(|_| NesError::GameDbFormat)
    }
}

// TODO: write tests for iNES 2.0 parsing
