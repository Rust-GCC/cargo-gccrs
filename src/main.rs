mod gccrs;

use gccrs::Gccrs;

fn main() {
    Gccrs::maybe_install().expect("gccrs should be installed");
}
