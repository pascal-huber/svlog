use clap::CommandFactory;

fn main() -> std::io::Result<()> {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    // FIXME: Find a better way to store ./target/X is rather nasty
    let man_dir = std::path::PathBuf::from(out_dir).join("../../../man/man1/");

    let cmd = svlog_cli::Args::command();
    let man = clap_mangen::Man::new(cmd);
    let mut buffer: Vec<u8> = Default::default();
    man.render(&mut buffer)?;
    std::fs::create_dir_all(&man_dir)?;
    std::fs::write(man_dir.join("svlog.1"), buffer)?;
    Ok(())
}
