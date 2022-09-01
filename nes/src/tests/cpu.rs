use super::*;

#[test]
fn blargg_instr_all() {
    blargg_test(
        "cpu/blargg_instr/all_instrs.nes",
        "All 16 tests passed\n\n\n",
    );
}

#[test]
fn blargg_instr_timing() {
    blargg_test(
        "cpu/instr_timing/instr_timing.nes",
        "\n2-branch_timing\n\nPassed\nAll 2 tests passed\n\n\n",
    );
}

#[test]
fn cpu_timing_test6() {
    hash_test(
        "cpu/cpu_timing_test6/cpu_timing_test.nes",
        640,
        4469663341197797724,
    );
}

#[test]
fn blargg_instr_misc_all() {
    blargg_test(
        "cpu/instr_misc/instr_misc.nes",
        "\n04-dummy_reads_apu\n\nPassed\nAll 4 tests passed\n\n\n",
    );
}

#[test]
fn cpu_dummy_writes_ppumem() {
    blargg_test(
    "cpu/cpu_dummy_writes/cpu_dummy_writes_ppumem.nes",
    "\u{1b}[0;37mTEST: cpu_dummy_writes_ppumem\n\u{1b}[0;33mThis program verifies that the\nCPU does 2x writes properly.\nAny read-modify-write opcode\nshould first write the origi-\nnal value; then the calculated\nvalue exactly 1 cycle later.\n\n\u{1b}[0;37mVerifying open bus behavior.\n\u{1b}[0;33m      W- W- WR W- W- W- W- WR\n2000+ 0  1  2  3  4  5  6  7 \n\u{1b}[0;33m  R0:\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m\n\u{1b}[0;33m  R1:\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m\n\u{1b}[0;33m  R3:\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m\n\u{1b}[0;33m  R5:\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m\n\u{1b}[0;33m  R6:\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m- 0\u{1b}[1;34m0\u{1b}[1;34m\n\u{1b}[0;37mOK; \u{1b}[0;37mVerifying opcodes...\n\u{1b}[1;34m0E\u{1b}[1;34m2E\u{1b}[1;34m4E\u{1b}[1;34m6E\u{1b}[1;34mCE\u{1b}[1;34mEE \u{1b}[1;34m1E\u{1b}[1;34m3E\u{1b}[1;34m5E\u{1b}[1;34m7E\u{1b}[1;34mDE\u{1b}[1;34mFE \n\u{1b}[1;34m0F\u{1b}[1;34m2F\u{1b}[1;34m4F\u{1b}[1;34m6F\u{1b}[1;34mCF\u{1b}[1;34mEF \u{1b}[1;34m1F\u{1b}[1;34m3F\u{1b}[1;34m5F\u{1b}[1;34m7F\u{1b}[1;34mDF\u{1b}[1;34mFF \n\u{1b}[1;34m03\u{1b}[1;34m23\u{1b}[1;34m43\u{1b}[1;34m63\u{1b}[1;34mC3\u{1b}[1;34mE3 \u{1b}[1;34m13\u{1b}[1;34m33\u{1b}[1;34m53\u{1b}[1;34m73\u{1b}[1;34mD3\u{1b}[1;34mF3 \n\u{1b}[1;34m1B\u{1b}[1;34m3B\u{1b}[1;34m5B\u{1b}[1;34m7B\u{1b}[1;34mDB\u{1b}[1;34mFB              \n\u{1b}[0;37m\nPassed\n"
);
}

#[test]
fn cpu_dummy_writes_oam() {
    blargg_test(
    "cpu/cpu_dummy_writes/cpu_dummy_writes_oam.nes",
    "\u{1b}[0;37mTEST: cpu_dummy_writes_oam\n\u{1b}[0;33mThis program verifies that the\nCPU does 2x writes properly.\nAny read-modify-write opcode\nshould first write the origi-\nnal value; then the calculated\nvalue exactly 1 cycle later.\n\nRequirement: OAM memory reads\nMUST be reliable. This is\noften the case on emulators,\nbut NOT on the real NES.\nNevertheless, this test can be\nused to see if the CPU in the\nemulator is built properly.\n\n\u{1b}[0;37m\u{1b}[0;37mTesting OAM.  The screen will go blank for a moment now.\n\u{1b}[0;37mOK; \u{1b}[0;37mVerifying opcodes...\n\u{1b}[1;34m0E\u{1b}[1;34m2E\u{1b}[1;34m4E\u{1b}[1;34m6E\u{1b}[1;34mCE\u{1b}[1;34mEE \u{1b}[1;34m1E\u{1b}[1;34m3E\u{1b}[1;34m5E\u{1b}[1;34m7E\u{1b}[1;34mDE\u{1b}[1;34mFE \n\u{1b}[1;34m0F\u{1b}[1;34m2F\u{1b}[1;34m4F\u{1b}[1;34m6F\u{1b}[1;34mCF\u{1b}[1;34mEF \u{1b}[1;34m1F\u{1b}[1;34m3F\u{1b}[1;34m5F\u{1b}[1;34m7F\u{1b}[1;34mDF\u{1b}[1;34mFF \n\u{1b}[1;34m03\u{1b}[1;34m23\u{1b}[1;34m43\u{1b}[1;34m63\u{1b}[1;34mC3\u{1b}[1;34mE3 \u{1b}[1;34m13\u{1b}[1;34m33\u{1b}[1;34m53\u{1b}[1;34m73\u{1b}[1;34mD3\u{1b}[1;34mF3 \n\u{1b}[1;34m1B\u{1b}[1;34m3B\u{1b}[1;34m5B\u{1b}[1;34m7B\u{1b}[1;34mDB\u{1b}[1;34mFB              \n\u{1b}[0;37m\nPassed\n"
);
}

#[test]
fn cpu_exec_space_ppuio() {
    blargg_test(
    "cpu/cpu_exec_space/test_cpu_exec_space_ppuio.nes",
    "\u{1b}[0;37mTEST:test_cpu_exec_space_ppuio\n\u{1b}[0;33mThis program verifies that the\nCPU can execute code from any\npossible location that it can\naddress, including I/O space.\n\nIn addition, it will be tested\nthat an RTS instruction does a\ndummy read of the byte that\nimmediately follows the\ninstructions.\n\n\u{1b}[0;37m\u{1b}[1;34mJSR+RTS TEST OK\nJMP+RTS TEST OK\nRTS+RTS TEST OK\nJMP+RTI TEST OK\nJMP+BRK TEST OK\n\u{1b}[0;37m\nPassed\n"
);
}

#[test]
fn cpu_exec_space_apu() {
    blargg_test(
    "cpu/cpu_exec_space/test_cpu_exec_space_apu.nes",
    "\u{1b}[0;37mTEST: test_cpu_exec_space_apu\n\u{1b}[0;33mThis program verifies that the\nCPU can execute code from any\npossible location that it can\naddress, including I/O space.\n\nIn this test, it is also\nverified that not only all\nwrite-only APU I/O ports\nreturn the open bus, but\nalso the unallocated I/O\nspace in $4018..$40FF.\n\n\u{1b}[0;37m\u{1b}[1;34m0022 \r4000 40 \r4001 40 \r4002 40 \r4003 40 \r4004 40 \r4005 40 \r4006 40 \r4007 40 \r4008 40 \r4009 40 \r400A 40 \r400B 40 \r400C 40 \r400D 40 \r400E 40 \r400F 40 \r4010 40 \r4011 40 \r4012 40 \r4013 40 \r4014 40 \r\r4016 40 \r4017 40 \r4018 40 \r4019 40 \r401A 40 \r401B 40 \r401C 40 \r401D 40 \r401E 40 \r401F 40 \r4020 40 \r4021 40 \r4022 40 \r4023 40 \r4024 40 \r4025 40 \r4026 40 \r4027 40 \r4028 40 \r4029 40 \r402A 40 \r402B 40 \r402C 40 \r402D 40 \r402E 40 \r402F 40 \r4030 40 \r4031 40 \r4032 40 \r4033 40 \r4034 40 \r4035 40 \r4036 40 \r4037 40 \r4038 40 \r4039 40 \r403A 40 \r403B 40 \r403C 40 \r403D 40 \r403E 40 \r403F 40 \r4040 40 \r4041 40 \r4042 40 \r4043 40 \r4044 40 \r4045 40 \r4046 40 \r4047 40 \r4048 40 \r4049 40 \r404A 40 \r404B 40 \r404C 40 \r404D 40 \r404E 40 \r404F 40 \r4050 40 \r4051 40 \r4052 40 \r4053 40 \r4054 40 \r4055 40 \r4056 40 \r4057 40 \r4058 40 \r4059 40 \r405A 40 \r405B 40 \r405C 40 \r405D 40 \r405E 40 \r405F 40 \r4060 40 \r4061 40 \r4062 40 \r4063 40 \r4064 40 \r4065 40 \r4066 40 \r4067 40 \r4068 40 \r4069 40 \r406A 40 \r406B 40 \r406C 40 \r406D 40 \r406E 40 \r406F 40 \r4070 40 \r4071 40 \r4072 40 \r4073 40 \r4074 40 \r4075 40 \r4076 40 \r4077 40 \r4078 40 \r4079 40 \r407A 40 \r407B 40 \r407C 40 \r407D 40 \r407E 40 \r407F 40 \r4080 40 \r4081 40 \r4082 40 \r4083 40 \r4084 40 \r4085 40 \r4086 40 \r4087 40 \r4088 40 \r4089 40 \r408A 40 \r408B 40 \r408C 40 \r408D 40 \r408E 40 \r408F 40 \r4090 40 \r4091 40 \r4092 40 \r4093 40 \r4094 40 \r4095 40 \r4096 40 \r4097 40 \r4098 40 \r4099 40 \r409A 40 \r409B 40 \r409C 40 \r409D 40 \r409E 40 \r409F 40 \r40A0 40 \r40A1 40 \r40A2 40 \r40A3 40 \r40A4 40 \r40A5 40 \r40A6 40 \r40A7 40 \r40A8 40 \r40A9 40 \r40AA 40 \r40AB 40 \r40AC 40 \r40AD 40 \r40AE 40 \r40AF 40 \r40B0 40 \r40B1 40 \r40B2 40 \r40B3 40 \r40B4 40 \r40B5 40 \r40B6 40 \r40B7 40 \r40B8 40 \r40B9 40 \r40BA 40 \r40BB 40 \r40BC 40 \r40BD 40 \r40BE 40 \r40BF 40 \r40C0 40 \r40C1 40 \r40C2 40 \r40C3 40 \r40C4 40 \r40C5 40 \r40C6 40 \r40C7 40 \r40C8 40 \r40C9 40 \r40CA 40 \r40CB 40 \r40CC 40 \r40CD 40 \r40CE 40 \r40CF 40 \r40D0 40 \r40D1 40 \r40D2 40 \r40D3 40 \r40D4 40 \r40D5 40 \r40D6 40 \r40D7 40 \r40D8 40 \r40D9 40 \r40DA 40 \r40DB 40 \r40DC 40 \r40DD 40 \r40DE 40 \r40DF 40 \r40E0 40 \r40E1 40 \r40E2 40 \r40E3 40 \r40E4 40 \r40E5 40 \r40E6 40 \r40E7 40 \r40E8 40 \r40E9 40 \r40EA 40 \r40EB 40 \r40EC 40 \r40ED 40 \r40EE 40 \r40EF 40 \r40F0 40 \r40F1 40 \r40F2 40 \r40F3 40 \r40F4 40 \r40F5 40 \r40F6 40 \r40F7 40 \r40F8 40 \r40F9 40 \r40FA 40 \r40FB 40 \r40FC 40 \r40FD 40 \r40FE 40 \r40FF 40 \u{1b}[0;37m\nPassed\n"
);
}

#[test]
fn cpu_dummy_reads() {
    hash_test(
        "cpu/cpu_dummy_reads/cpu_dummy_reads.nes",
        68,
        13203765892084420806,
    );
}

#[test]
fn branch_timing_tests_branch_basics() {
    hash_test(
        "cpu/branch_timing_tests/1.Branch_Basics.nes",
        28,
        15118594316271482838,
    );
}

#[test]
fn branch_timing_tests_branch_backward() {
    hash_test(
        "cpu/branch_timing_tests/2.Backward_Branch.nes",
        28,
        8175657970949101239,
    );
}

#[test]
fn branch_timing_tests_branch_forward() {
    hash_test(
        "cpu/branch_timing_tests/3.Forward_Branch.nes",
        27,
        10319321477101815957,
    );
}
