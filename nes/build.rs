mod cpu_generator;

fn main() {
    println!("cargo:rerun-if-changed=src/nes/cpu");
    cpu_generator::generate_cpu();
}
