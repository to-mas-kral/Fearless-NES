use std::convert::TryFrom;

use sha1::{Digest, Sha1};

use crate::{
    cartridge::{ConsoleType, Header, Region},
    ppu::Mirroring,
    NesError,
};

use super::HeaderSource;

const GAMEDB: &str = include_str!("nes20db.xml");

impl Header {
    pub fn from_prg_chr(prg: &[u8], chr: Option<&[u8]>) -> Result<Option<Self>, NesError> {
        let prg_sha1 = Self::arr_to_hex(Sha1::digest(prg).as_slice());
        let chr_sha1 = chr.and_then(|chr| Some(Self::arr_to_hex(Sha1::digest(chr).as_slice())));

        Self::find_game(&prg_sha1, chr_sha1.as_deref())
    }

    fn arr_to_hex(arr: &[u8]) -> String {
        let mut res = String::new();

        let nibble_to_hex = |nibble| {
            return match nibble {
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
                .split_once("\\")
                .ok_or(NesError::GameDbFormat)?
                .1
                .trim_end_matches(".nes"),
        );

        let pcb = game
            .children()
            .find(|n| n.tag_name().name() == "pcb")
            .ok_or(NesError::GameDbFormat)?;

        let prg_rom_size = game
            .children()
            .find(|n| n.tag_name().name() == "prgrom")
            .ok_or(NesError::GameDbFormat)?
            .attribute("size")
            .ok_or(NesError::GameDbFormat)?
            .parse::<u32>()
            .map_err(|_| NesError::GameDbFormat)?;

        let chr_rom_size = game
            .children()
            .find(|n| n.tag_name().name() == "chrrom")
            .and_then(|chr| chr.attribute("size"))
            .and_then(|size| Some(size.parse::<u32>()))
            .transpose()
            .map_err(|_| NesError::GameDbFormat)?;

        let chr_ram_size = game
            .children()
            .find(|n| n.tag_name().name() == "chrram")
            .and_then(|chr| chr.attribute("size"))
            .and_then(|size| Some(size.parse::<u32>()))
            .transpose()
            .map_err(|_| NesError::GameDbFormat)?;

        let prg_ram_size = game
            .children()
            .find(|n| n.tag_name().name() == "prgram")
            .and_then(|chr| chr.attribute("size"))
            .and_then(|size| Some(size.parse::<u32>()))
            .transpose()
            .map_err(|_| NesError::GameDbFormat)?;

        let prg_nvram_size = game
            .children()
            .find(|n| n.tag_name().name() == "prgnvram")
            .and_then(|chr| chr.attribute("size"))
            .and_then(|size| Some(size.parse::<u32>()))
            .transpose()
            .map_err(|_| NesError::GameDbFormat)?;

        let mapper = pcb
            .attribute("mapper")
            .ok_or(NesError::GameDbFormat)?
            .parse::<u32>()
            .map_err(|_| NesError::GameDbFormat)?;

        let submapper = pcb
            .attribute("submapper")
            .ok_or(NesError::GameDbFormat)?
            .parse::<u32>()
            .map_err(|_| NesError::GameDbFormat)?;

        let mirroring =
            Mirroring::try_from(pcb.attribute("mirroring").ok_or(NesError::GameDbFormat)?)?;

        let battery = pcb
            .attribute("battery")
            .ok_or(NesError::GameDbFormat)?
            .parse::<u32>()
            .map_err(|_| NesError::GameDbFormat)?
            != 0;

        let console = game
            .children()
            .find(|n| n.tag_name().name() == "console")
            .ok_or(NesError::GameDbFormat)?;

        let console_typ =
            ConsoleType::try_from(console.attribute("type").ok_or(NesError::GameDbFormat)?)?;

        let region = Region::try_from(console.attribute("region").ok_or(NesError::GameDbFormat)?)?;

        let expansion = game
            .children()
            .find(|n| n.tag_name().name() == "expansion")
            .ok_or(NesError::GameDbFormat)?
            .attribute("type")
            .ok_or(NesError::GameDbFormat)?
            .parse::<u32>()
            .map_err(|_| NesError::GameDbFormat)?;

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
}
