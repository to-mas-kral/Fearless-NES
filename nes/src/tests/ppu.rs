use super::*;

//TODO: get read_buffer test working
//TODO: implement oamtest3 - iNES 2.0 needed

//TODO: group vbl_nmi tests after passing them
#[test]
fn ppu_vbl_nmi_basics() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/01-vbl_basics.nes",
        "\n01-vbl_basics\n\nPassed\n",
    );
}
#[test]
fn ppu_vbl_nmi_set_time() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/02-vbl_set_time.nes",
        "T+ 1 2\n00 - V\n01 - V\n02 - V\n03 - V\n04 - -\n05 V -\n06 V -\n07 V -\n08 V -\n\n02-vbl_set_time\n\nPassed\n"
    );
}
#[test]
fn ppu_vbl_nmi_clear_time() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/03-vbl_clear_time.nes",
        "00 V\n01 V\n02 V\n03 V\n04 V\n05 V\n06 -\n07 -\n08 -\n\n03-vbl_clear_time\n\nPassed\n",
    );
}

#[test]
fn ppu_vbl_nmi_control() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/04-nmi_control.nes",
        "\n04-nmi_control\n\nPassed\n",
    );
}

#[test]
#[ignore = "ppu_vbl advanced test don't work yet"]
fn ppu_vbl_nmi_timing() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/05-nmi_timing.nes",
        "00 4\n01 4\n02 4\n03 3\n04 3\n05 3\n06 3\n07 3\n08 3\n09 2\n\n05-nmi_timing\n\nPassed\n",
    );
}

#[test]
fn ppu_vbl_nmi_suppression() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/06-suppression.nes",
        "00 - N\n01 - N\n02 - N\n03 - N\n04 - -\n05 V -\n06 V -\n07 V N\n08 V N\n09 V N\n\n06-suppression\n\nPassed\n"
    );
}

#[test]
#[ignore = "ppu_vbl advanced test don't work yet"]
fn ppu_vbl_nmi_on_timing() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/07-nmi_on_timing.nes",
        "00 N\n01 N\n02 N\n03 N\n04 N\n05 -\n06 -\n07 -\n08 -\n\n07-nmi_on_timing\n\nPassed\n",
    );
}

#[test]
#[ignore = "ppu_vbl advanced test don't work yet"]
fn ppu_vbl_nmi_off_timing() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/08-nmi_off_timing.nes",
        "03 -\n04 -\n05 -\n06 -\n07 N\n08 N\n09 N\n0A N\n0B N\n0C N\n\n08-nmi_off_timing\n\nPassed\n"
    );
}

#[test]
#[ignore = "ppu_vbl advanced test don't work yet"]
fn ppu_vbl_nmi_even_odd_frames() {
    blargg_test(
        "ppu/ppu_vbl_nmi/rom_singles/09-even_odd_frames.nes",
        "00 01 01 02 \n09-even_odd_frames\n\nPassed\n",
    );
}

#[test]
#[ignore = "ppu_vbl advanced test don't work yet"]
fn ppu_vbl_nmi_even_odd_timing() {
    blargg_test("ppu/ppu_vbl_nmi/rom_singles/10-even_odd_timing.nes", "");
}

#[test]
fn blargg_ppu_tests_2005_palette_ram() {
    hash_test(
        "ppu/blargg_ppu_tests_2005.09.15b/palette_ram.nes",
        49,
        1479301556045212574,
    );
}

#[test]
fn blargg_ppu_tests_2005_power_up_palette() {
    hash_test(
        "ppu/blargg_ppu_tests_2005.09.15b/power_up_palette.nes",
        49,
        1479301556045212574,
    );
}

#[test]
fn blargg_ppu_tests_2005_sprite_ram() {
    hash_test(
        "ppu/blargg_ppu_tests_2005.09.15b/sprite_ram.nes",
        49,
        1479301556045212574,
    );
}

#[test]
fn blargg_ppu_tests_2005_vbl_clear_time() {
    hash_test(
        "ppu/blargg_ppu_tests_2005.09.15b/vbl_clear_time.nes",
        49,
        1479301556045212574,
    );
}

#[test]
fn blargg_ppu_tests_2005_vram_access() {
    hash_test(
        "ppu/blargg_ppu_tests_2005.09.15b/vram_access.nes",
        49,
        1479301556045212574,
    );
}

#[test]
fn oam_read() {
    blargg_test(
    "ppu/oam_read/oam_read.nes",
    "----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_read\n\nPassed\n"
);
}

#[test]
fn oam_stress() {
    blargg_test(
    "ppu/oam_stress/oam_stress.nes",
    "----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n----------------\n\noam_stress\n\nPassed\n"
);
}

#[test]
fn sprite_overflow_tests_basics() {
    hash_test(
        "ppu/sprite_overflow_tests/1.Basics.nes",
        33,
        13692212044290598699,
    );
}

#[test]
fn sprite_overflow_tests_details() {
    hash_test(
        "ppu/sprite_overflow_tests/2.Details.nes",
        40,
        12646558507142779566,
    );
}

#[test]
fn sprite_overflow_tests_timing() {
    hash_test(
        "ppu/sprite_overflow_tests/3.Timing.nes",
        122,
        16317615199587539327,
    );
}

#[test]
fn sprite_overflow_tests_obscure() {
    hash_test(
        "ppu/sprite_overflow_tests/4.Obscure.nes",
        43,
        8959535434800245388,
    );
}

#[test]
fn sprite_overflow_tests_emulator() {
    hash_test(
        "ppu/sprite_overflow_tests/5.Emulator.nes",
        37,
        17585350519035883224,
    );
}

//TODO: group sprite hit tests after passing the last one
#[test]
fn ppu_sprite_hit_basics() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/01-basics.nes",
        "\n01-basics\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_alignment() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/02-alignment.nes",
        "\n02-alignment\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_corners() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/03-corners.nes",
        "\n03-corners\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_flip() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/04-flip.nes",
        "\n04-flip\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_left_clip() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/05-left_clip.nes",
        "\n05-left_clip\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_right_edge() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/06-right_edge.nes",
        "\n06-right_edge\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_screen_bottom() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/07-screen_bottom.nes",
        "\n07-screen_bottom\n\nPassed\n",
    );
}

#[test]
fn ppu_sprite_hit_double_height() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/08-double_height.nes",
        "\n08-double_height\n\nPassed\n",
    );
}

#[test]
#[ignore = "ppu_sprite_hit_timing advanced test don't work yet"]
fn ppu_sprite_hit_timing() {
    blargg_test("ppu/ppu_sprite_hit/rom_singles/09-timing.nes", "");
}

#[test]
fn ppu_sprite_hit_timing_order() {
    blargg_test(
        "ppu/ppu_sprite_hit/rom_singles/10-timing_order.nes",
        "\n10-timing_order\n\nPassed\n",
    );
}

#[test]
fn vbl_nmi_timing_frame_basics() {
    hash_test(
        "ppu/vbl_nmi_timing/1.frame_basics.nes",
        180,
        9868293553859411772,
    );
}

#[test]
fn vbl_nmi_timing_vbl_timing() {
    hash_test(
        "ppu/vbl_nmi_timing/2.vbl_timing.nes",
        180,
        7027774011718828111,
    );
}

#[test]
fn vbl_nmi_timing_even_odd_frames() {
    hash_test(
        "ppu/vbl_nmi_timing/3.even_odd_frames.nes",
        122,
        4298393885108462395,
    );
}

#[test]
fn vbl_nmi_timing_vbl_clear_timing() {
    hash_test(
        "ppu/vbl_nmi_timing/4.vbl_clear_timing.nes",
        140,
        4691931363517236132,
    );
}

#[test]
fn vbl_nmi_timing_nmi_suppression() {
    hash_test(
        "ppu/vbl_nmi_timing/5.nmi_suppression.nes",
        180,
        10661010577938478738,
    );
}

#[test]
#[ignore = "vbl_nmi_timing advanced tests don't work yet"]
fn vbl_nmi_timing_nmi_disable() {
    hash_test("ppu/vbl_nmi_timing/6.nmi_disable.nes", 180, 0);
}

#[test]
#[ignore = "vbl_nmi_timing advanced tests don't work yet"]
fn vbl_nmi_timing_nmi_timing() {
    hash_test("ppu/vbl_nmi_timing/7.nmi_timing.nes", 180, 0);
}
