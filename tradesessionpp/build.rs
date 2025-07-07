fn main() {
    cxx_build::bridge("src/tradesessionpp.rs")
        .std("c++14")
        .compile("tradesessionpp");

    println!("cargo:rerun-if-changed=src/tradesessionpp.rs");
}
