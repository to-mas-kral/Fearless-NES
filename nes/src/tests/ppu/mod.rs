use super::*;

//TODO: get read_buffer test working
//TODO: implement oamtest3 - mapper 7 needed

//TODO: refactor vbl_nmi tests
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
    "00 N\n01 N\n02 N\n03 N\n04 N\n05 -\n06 -\n07 -\n08 -\n\n07-nmi_on_timing\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_off_timing,
    "ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes",
    "03 -\n04 -\n05 -\n06 -\n07 N\n08 N\n09 N\n0A N\n0B N\n0C N\n\n08-nmi_off_timing\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_even_odd_frames,
    "ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes",
    "00 01 01 02 \n09-even_odd_frames\n\nPassed\n"
);

blargg_test!(
    ppu_vbl_nmi_even_odd_timing,
    "ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes",
    ""
);

hash_test!(
    blargg_ppu_tests_2005_palette_ram,
    "ppu/blargg_ppu_tests_2005.09.15b/palette_ram.nes",
    180,
    437891936304612670
);

hash_test!(
    blargg_ppu_tests_2005_power_up_palette,
    "ppu/blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
    180,
    437891936304612670
);

hash_test!(
    blargg_ppu_tests_2005_sprite_ram,
    "ppu/blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
    180,
    437891936304612670
);

hash_test!(
    blargg_ppu_tests_2005_vbl_clear_time,
    "ppu/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
    180,
    437891936304612670
);

hash_test!(
    blargg_ppu_tests_2005_vram_access,
    "ppu/blargg_ppu_tests_2005.09.15b/vram_access.nes",
    180,
    8270824088457806082
);

blargg_test!(oam_read, "ppu/oam_read/oam_read.nes","----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_read\n\nPassed\n");
blargg_test!(oam_stress, "ppu/oam_stress/oam_stress.nes", "----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_stress\n\nPassed\n");

hash_test!(
    sprite_overflow_tests_basics,
    "ppu/sprite_overflow_tests/1.Basics.nes",
    180,
    12903855929148818637
);

hash_test!(
    sprite_overflow_tests_details,
    "ppu/sprite_overflow_tests/2.Details.nes",
    180,
    11212288191330984064
);

hash_test!(
    sprite_overflow_tests_timing,
    "ppu/sprite_overflow_tests/3.Timing.nes",
    180,
    9851834163110547518
);

hash_test!(
    sprite_overflow_tests_obscure,
    "ppu/sprite_overflow_tests/4.Obscure.nes",
    180,
    406031556439261211
);

hash_test!(
    sprite_overflow_tests_emulator,
    "ppu/sprite_overflow_tests/5.Emulator.nes",
    180,
    4343549908322138215
);

//TODO: refactor sprite hit tests
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

blargg_test!(
    ppu_sprite_hit_timing,
    "ppu/ppu_sprite_hit/rom_singles/09-timing.nes",
    ""
);

blargg_test!(
    ppu_sprite_hit_timing_order,
    "ppu/ppu_sprite_hit/rom_singles/10-timing_order.nes",
    "\n10-timing_order\n\nPassed\n"
);

hash_test!(
    vbl_nmi_timing_frame_basics,
    "ppu/vbl_nmi_timing/1.frame_basics.nes",
    180,
    3452887670413388586
);

hash_test!(
    vbl_nmi_timing_vbl_timing,
    "ppu/vbl_nmi_timing/2.vbl_timing.nes",
    180,
    14772986747222263228
);

hash_test!(
    vbl_nmi_timing_even_odd_frames,
    "ppu/vbl_nmi_timing/3.even_odd_frames.nes",
    180,
    3063649233938891139
);

hash_test!(
    vbl_nmi_timing_vbl_clear_timing,
    "ppu/vbl_nmi_timing/4.vbl_clear_timing.nes",
    180,
    10129148082447838393
);

hash_test!(
    vbl_nmi_timing_nmi_suppression,
    "ppu/vbl_nmi_timing/5.nmi_suppression.nes",
    180,
    16599726142867019008
);

hash_test!(
    vbl_nmi_timing_nmi_disable,
    "ppu/vbl_nmi_timing/6.nmi_disable.nes",
    180,
    0
);

hash_test!(
    vbl_nmi_timing_nmi_timing,
    "ppu/vbl_nmi_timing/7.nmi_timing.nes",
    180,
    0
);

hash_test!(
    scanline,
    "ppu/scanline/scanline.nes",
    250,
    3225924796581909531
);
