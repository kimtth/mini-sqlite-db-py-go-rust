use mini_sqlite::cli::shell::run_shell;
use mini_sqlite::web::server::run_server;
/// Entry point for the mini SQL project.
use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.contains(&"--web".to_string()) {
        let host = args
            .iter()
            .position(|arg| arg == "--host")
            .and_then(|i| args.get(i + 1))
            .map(|s| s.as_str())
            .unwrap_or("127.0.0.1");

        let port = args
            .iter()
            .position(|arg| arg == "--port")
            .and_then(|i| args.get(i + 1))
            .and_then(|s| s.parse().ok())
            .unwrap_or(8000);

        run_server(host, port);
    } else {
        run_shell();
    }
}
