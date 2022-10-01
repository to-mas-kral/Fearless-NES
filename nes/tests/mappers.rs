mod common;

use common::blargg_test;

#[test]
fn mmc3_test_2_1_clocking() {
    blargg_test(
        "mappers/mmc3_test_2/1-clocking.nes",
        "\n1-clocking\n\nPassed\n",
    );
}

#[test]
fn mmc3_test_2_2_details() {
    blargg_test(
        "mappers/mmc3_test_2/2-details.nes",
        "\n2-details\n\nPassed\n",
    );
}

#[test]
fn mmc3_test_2_3_a12_clocking() {
    blargg_test(
        "mappers/mmc3_test_2/3-A12_clocking.nes",
        "\n3-A12_clocking\n\nPassed\n",
    );
}

#[ignore = "doesn't work yet, maybe the dot skipping is wrong or 8x16 sprites are wrong"]
#[test]
fn mmc3_test_2_4_scanline_timing() {
    blargg_test("mappers/mmc3_test_2/4-scanline_timing.nes", "");
}

#[test]
fn mmc3_test_2_5_mmc3() {
    blargg_test("mappers/mmc3_test_2/5-MMC3.nes", "\n5-MMC3\n\nPassed\n");
}
