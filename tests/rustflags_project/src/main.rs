fn main() {
    // Create an unused variable on purpose so that when using -D warning, rustc errors out
    // and we get a specific compiler flag.
    let a = 15;
}
