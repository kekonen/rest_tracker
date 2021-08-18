

use rest_tracker::run;


fn main() {
    run(|s| {
        std::process::Command::new("sh")
            .arg("-c")
            .arg(format!("notify {}", s))
            .output()
            .expect("failed to execute process");
    })
}