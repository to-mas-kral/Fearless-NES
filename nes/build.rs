mod cpu_generator;

fn main() {
    println!("cargo:rerun-if-changed=cpu_generator");
    cpu_generator::generate_cpu();
}
