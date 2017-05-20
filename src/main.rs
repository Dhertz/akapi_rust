extern crate akapi_rust;

use akapi_rust::run_purple_mailer;

fn main() {
    let t = run_purple_mailer(10);
    t.join();
    println!("done");
}
