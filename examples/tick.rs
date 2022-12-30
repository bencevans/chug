use chug::Chug;

fn main() {
    let mut chug = Chug::new(10, 100);

    for _ in 0..100 {
        let formatted_eta = match chug.eta() {
            Some(eta) => {
                let eta_secs = eta.as_secs();
                let eta_millis = eta.subsec_millis();
                format!("ETA: {}.{:03}s", eta_secs, eta_millis)
            }
            None => "ETA: None".to_string(),
        };
        println!("{}", formatted_eta);

        // Do some work
        std::thread::sleep(std::time::Duration::from_millis(50));

        chug.tick();
    }
}
