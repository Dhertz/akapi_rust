extern crate akapi_rust;

fn main() {
    let mut threads = Vec::new();
    threads.push(akapi_rust::run_purple_mailer(3600));
    for thread in threads {
        thread.join().unwrap();
    }
    println!("done");
}
