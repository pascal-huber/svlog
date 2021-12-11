use clap::{App, load_yaml};
use glob::glob;

fn list_services(){
    println!("Services:");
    let path = std::path::Path::new("/var/log/socklog/");
    for entry in path.read_dir().expect("read_dir call failed")  {
        if let Ok(entry) = entry {
            let p = entry.path();
            let filename = p.file_name().unwrap().to_str().unwrap();
            println!(" - {}", filename);
        }
    }
}

fn services_files(services: Vec<&str>, files: &mut Vec<std::path::PathBuf>){
    for service in services {
        // TODO: move basedir out of the loop
        let basedir : String = "/var/log/socklog/".to_string();
        let glb = basedir + service + "/*.[us]";
        for entry in glob(&glb[..]).expect("Failed to read glob pattern") {
            if let Ok(path) = entry {
                files.push(path);
            }
        }
    }
}

fn main() {

    let yaml = load_yaml!("cli.yaml");
    let matches = App::from(yaml).get_matches();

    if matches.is_present("list"){
        list_services();
        std::process::exit(0);
    }

    if matches.is_present("services"){

        let services: Vec<&str> = matches.values_of("services").unwrap().collect();
        let mut files : Vec<std::path::PathBuf> = Vec::new();
        services_files(services, &mut files);
        println!("files:");
        for x in files {
            println!(" - {:?}", x);
        }

    }

}
