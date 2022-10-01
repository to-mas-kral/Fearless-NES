use common::blargg_test;

mod common;

#[test]
fn apu_length_counter() {
    blargg_test(
        "apu/apu_test/rom_singles/1-len_ctr.nes",
        "\n1-len_ctr\n\nPassed\n",
    );
}

#[test]
fn apu_length_counter_table() {
    blargg_test(
        "apu/apu_test/rom_singles/2-len_table.nes",
        "\n2-len_table\n\nPassed\n",
    );
}

#[test]
fn apu_irq_flag() {
    blargg_test(
        "apu/apu_test/rom_singles/3-irq_flag.nes",
        "\n3-irq_flag\n\nPassed\n",
    );
}

// TODO: APU all blargg test and group
/* #[test]
fn apu_blargg_all() {
    blargg_test("apu/apu_test/apu_test.nes", "passed");
} */
