use clap::Shell;
mod cli {
    include!("src/cli.rs");
}

fn build_completions(name: &str, shells: &[clap::Shell]) {
    let outdir = match std::env::var_os("OUT_DIR") {
        None => return,
        Some(outdir) => outdir,
    };
    let stamp_path = std::path::Path::new(&outdir).join("mctl-stamp");
    if let Err(err) = std::fs::File::create(&stamp_path) {
        panic!("failed to write {} {}", stamp_path.display(), err);
    }
    let mut cli = cli::build_cli();

    for shell in shells {
        cli.gen_completions(name, *shell, &outdir);
    }
}

fn main() {
    if Ok("release".to_owned()) == std::env::var("PROFILE") {
        build_completions("mctl", &[Shell::Fish, Shell::Bash, Shell::Zsh]);
    }
}
