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

blargg_test!(
    ppu_vbl_nmi_timing,
    "ppu/ppu_vbl_nmi/rom_singles/05-nmi_timing.nes",
    "00 4\n01 4\n02 4\n03 3\n04 3\n05 3\n06 3\n07 3\n08 3\n09 2\n\n05-nmi_timing\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_suppression,
    "ppu/ppu_vbl_nmi/rom_singles/06-suppression.nes",
    "00 - N\n01 - N\n02 - N\n03 - N\n04 - -\n05 V -\n06 V -\n07 V N\n08 V N\n09 V N\n\n06-suppression\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_on_timing,
    "ppu/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes",
    ""
);

blargg_test!(
    ppu_vbl_nmi_off_timing,
    "ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes",
    "03 -\n04 -\n05 -\n06 -\n07 N\n08 N\n09 N\n0A N\n0B N\n0C N\n\n08-nmi_off_timing\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_odd_frames,
    "ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes",
    ""
);

blargg_test!(
    ppu_vbl_nmi_odd_timing,
    "ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes",
    ""
);

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

//TODO: automatize sprite_overflow tests
//blargg_test!(
//    sprite_overflow_tests_basics,
//    "ppu/sprite_overflow_tests/1.Basics.nes",
//    ""
//);
//blargg_test!(
//    sprite_overflow_tests_details,
//    "ppu/sprite_overflow_tests/2.Details.nes",
//    ""
//);
//blargg_test!(
//    sprite_overflow_tests_timing,
//    "ppu/sprite_overflow_tests/3.Timing.nes",
//    ""
//);
//blargg_test!(
//    sprite_overflow_tests_obscure,
//    "ppu/sprite_overflow_tests/4.Obscure.nes",
//    ""
//);
//blargg_test!(
//    sprite_overflow_tests_emulator,
//    "ppu/sprite_overflow_tests/5.Emulator.nes",
//    ""
//);

blargg_test!(
    ppu_sprite_hit_basics,
    "ppu/ppu_sprite_hit/rom_singles/01-basics.nes",
    "\n01-basics\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_alignment,
    "ppu/ppu_sprite_hit/rom_singles/02-alignment.nes",
    "\n02-alignment\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_corners,
    "ppu/ppu_sprite_hit/rom_singles/03-corners.nes",
    "\n03-corners\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_flip,
    "ppu/ppu_sprite_hit/rom_singles/04-flip.nes",
    "\n04-flip\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_left_clip,
    "ppu/ppu_sprite_hit/rom_singles/05-left_clip.nes",
    "\n05-left_clip\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_right_edge,
    "ppu/ppu_sprite_hit/rom_singles/06-right_edge.nes",
    "\n06-right_edge\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_screen_bottom,
    "ppu/ppu_sprite_hit/rom_singles/07-screen_bottom.nes",
    "\n07-screen_bottom\n\nPassed\n"
);

blargg_test!(
    ppu_sprite_hit_double_height,
    "ppu/ppu_sprite_hit/rom_singles/08-double_height.nes",
    "\n08-double_height\n\nPassed\n"
);
