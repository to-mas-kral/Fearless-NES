use super::*;

blargg_test!(
    ppu_vbl_nmi_basics,
    "ppu/ppu_vbl_nmi/rom_singles/01-vbl_basics.nes",
    "\n01-vbl_basics\n\nPassed\n"
);
blargg_test!(
    ppu_vbl_nmi_set_time,
    "ppu/ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes",
    "T+ 1 2\n00 - V\n01 - V\n02 - V\n03 - V\n04 - -\n05 V -\n06 V -\n07 V -\n08 V -\n\n02-vbl_set_time\n\nPassed\n"
);
blargg_test!(
    ppu_vbl_nmi_clear_time,
    "ppu/ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes",
    "00 V\n01 V\n02 V\n03 V\n04 V\n05 V\n06 -\n07 -\n08 -\n\n03-vbl_clear_time\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_control,
    "ppu/ppu_vbl_nmi/rom_singles/04-nmi_control.nes",
    "\n04-nmi_control\n\nPassed\n"
);

//TODO: get nmi_timing test working
/*blargg_test!(
    ppu_vbl_nmi_timing,
    "ppu/ppu_vbl_nmi/rom_singles/05-nmi_timing.nes",
    "wrong"
);*/

//"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/06-suppression.nes"
//"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes"
//"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes"
//"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes"
//"/home/tomas/Documents/Programovani/fearless-nes/nes/src/tests/ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes"

//TODO: automatize blargg_ppu_tests_2005
//blargg_test!(
//    blargg_ppu_tests_2005_palette_ram,
//    "ppu/blargg_ppu_tests_2005.09.15b/palette_ram.nes",
//    ""
//);
//
//
//blargg_test!(
//    blargg_ppu_tests_2005_power_up_palette,
//    "ppu/blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
//    ""
//);
//
//
//blargg_test!(
//    blargg_ppu_tests_2005_sprite_ram,
//    "ppu/blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
//    ""
//);
//
//
//blargg_test!(
//    blargg_ppu_tests_2005_vbl_clear_time,
//    "ppu/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
//    ""
//);
//
//
//blargg_test!(
//    blargg_ppu_tests_2005_vram_access,
//    "ppu/blargg_ppu_tests_2005.09.15b/vram_access.nes",
//    ""
//);

blargg_test!(oam_read, "ppu/oam_read/oam_read.nes","----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_read\n\nPassed\n");
blargg_test!(oam_stress, "ppu/oam_stress/oam_stress.nes", "----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_stress\n\nPassed\n");
