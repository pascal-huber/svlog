mod test_macros;

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::prelude::*;

#[test]
fn help_exists() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("-h");
    cmd.success()
        .stdout(contains_all!("USAGE", "svlog", "--utc"));
    Ok(())
}

#[test]
fn list_services() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("-l");
    cmd.success()
        .stdout(contains_all!(" - daemon", " - kernel"));
    Ok(())
}

#[test]
fn no_args() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!();
    cmd.success().stdout(contains_all!("kern", "daemon"));
    Ok(())
}

#[test]
fn service_kernel() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("kernel", "--utc");
    cmd.success().stdout(
        predicate::str::is_match(
            "^\
            2022-09-02T13:32:53.68972Z kern.debug: happy, happy, happy\n\
            2022-09-02T13:33:53.68972Z kern.info: hello fren!\n\
            2022-09-02T13:34:53.68972Z kern.notice: look at this!\n\
            2022-09-02T13:35:53.68972Z kern.warn: this could be bad, no\\?\n\
            2022-09-02T13:36:53.68972Z kern.err: it is bad!\n\
            2022-09-02T13:37:53.68972Z kern.crit: oh dear... it is very bad!\n\
            2022-09-02T13:38:53.68972Z kern.alert: \\*kernel screams\\*\n\
            2022-09-02T13:39:53.68972Z kern.emerg: the kernel has uninvited you from its birthday party\n\
            $",
        )
        .unwrap(),
    );
    Ok(())
}

#[test]
fn priority() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("-p", "alert..err");
    cmd.success().stdout(
        predicate::str::is_match(
            "^\
            .*daemon.err.*\n\
            .*daemon.crit.*\n\
            .*daemon.alert.*\n\
            .*kern.err.*\n\
            .*kern.crit.*\n\
            .*kern.alert.*\n\
            $",
        )
        .unwrap(),
    );
    Ok(())
}

#[test]
fn date_since() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("-s", "2022-09-02 13:37:53", "--utc", "kernel");
    cmd.success().stdout(
        predicate::str::is_match(
            "^\
            2022-09-02T13:37:53.*\n\
            2022-09-02T13:38:53.*\n\
            2022-09-02T13:39:53.*\n\
            $",
        )
        .unwrap(),
    );
    Ok(())
}

#[test]
fn date_until() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("-u", "2022-09-02 13:36:53", "--utc", "kernel");
    cmd.success().stdout(
        predicate::str::is_match(
            "^\
            2022-09-02T13:32:53.*\n\
            2022-09-02T13:33:53.*\n\
            2022-09-02T13:34:53.*\n\
            2022-09-02T13:35:53.*\n\
            2022-09-02T13:36:53.*\n\
            $",
        )
        .unwrap(),
    );
    Ok(())
}

#[test]
fn lines() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--lines", "4", "kernel", "--utc");
    cmd.success().stdout(
        predicate::str::is_match(
            "^\
            2022-09-02T13:36:53.*\n\
            2022-09-02T13:37:53.*\n\
            2022-09-02T13:38:53.*\n\
            2022-09-02T13:39:53.*\n\
            $",
        )
        .unwrap(),
    );
    Ok(())
}

#[test]
fn invalid_service() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("yoyoyo");
    cmd.failure()
        .stderr(contains_all!("Service \"yoyoyo\" not found"));
    Ok(())
}

#[test]
fn invalid_args_1() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--boot-offset", "1", "--follow");
    cmd.failure().stderr(contains_all!(
        "--boot-offset",
        "cannot be used with",
        "--follow",
    ));
    Ok(())
}

#[test]
fn invalid_args_2() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--boot-offset", "1", "--boot");
    cmd.failure().stderr(contains_all!(
        "--boot-offset",
        "cannot be used with",
        "--boot",
    ));
    Ok(())
}

#[test]
fn invalid_args_3() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--lines", "1", "--boot");
    cmd.failure()
        .stderr(contains_all!("--lines", "cannot be used with", "--boot",));
    Ok(())
}

#[test]
fn invalid_args_4() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--lines", "1", "--since", "today");
    cmd.failure()
        .stderr(contains_all!("--lines", "cannot be used with", "--since",));
    Ok(())
}

#[test]
fn invalid_args_5() -> Result<(), Box<dyn std::error::Error>> {
    let cmd = svlog!("--lines", "1", "--until", "today");
    cmd.failure()
        .stderr(contains_all!("--lines", "cannot be used with", "--until",));
    Ok(())
}
