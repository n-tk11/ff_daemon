use serde::{Serialize, Deserialize};
use serde_json;
use std::process::Command;
use shlex::Shlex;

//Run Json Instance
#[derive(Debug, Serialize, Deserialize)]
struct Runj {
    app_args: String,
    image_url: String,
    on_app_ready: String,
    passphrase_file: String,
    preserved_paths: String,
    no_restore: bool,
    allow_bad_image: bool,
    leave_stopped: bool,
    verbose: u8,
}

//Checkpoint Json Instance
#[derive(Debug, Serialize, Deserialize)]
struct Checkpointj {
    image_url: String,
    passphrase_file: String,
    preserved_paths: String,
    leave_running: bool,
    num_shards: String,
    cpu_budget: String,
    verbose: u8,
}

pub fn run_execute(body_str: String) -> u8 {

    //Parse Json into cmd args and call run
    let r: Runj = serde_json::from_str(&body_str).unwrap();
    let mut cmd = Command::new("fastfreeze");
            
    cmd.arg("run");
    
    if r.image_url.as_str() != "" {
        cmd.arg("--image-url").arg(&r.image_url); 
    }
    if r.on_app_ready.as_str() != ""{
        cmd.arg("--on-app-ready").arg(&r.on_app_ready); 
    }
    if r.passphrase_file.as_str() != "" {
        cmd.arg("--passphrase-file").arg(&r.passphrase_file); 
    }
    if r.preserved_paths.as_str() != "" {
        cmd.arg("--preserve-path").arg(&r.preserved_paths); 
    }
    if r.no_restore {
        cmd.arg("--no-restore");
    }
    if r.allow_bad_image {
        cmd.arg("--allow-bad-image-version");
    }
    if r.leave_stopped {
        cmd.arg("--leave-stoped");
    }
    let verbose = format!("-{}","v".repeat(r.verbose.into()));
    if r.verbose!=0 {
        cmd.arg(&verbose);
    }
    cmd.arg("--");
    if r.app_args.as_str() != "" {
        let splited_args: Vec<String> = Shlex::new(&r.app_args).collect();
        for word in splited_args {
            cmd.arg(word);
        }    
    } 
        
    match cmd.spawn() {
        Ok(_) => {
            println!("FF run spawned successfully");
            0
        },
        Err(e) => {
            println!("FF run failed to spawn: {e:?}");
            1
        }
    }
}


pub fn checkpoint_execute(body_str: String) -> u8 {

    ////Parse Json into cmd args and call checkpoint
    let r: Checkpointj = serde_json::from_str(&body_str).unwrap();
    let mut cmd = Command::new("fastfreeze");
            
    cmd.arg("checkpoint");
   
    if r.leave_running {
        cmd.arg("--leave-running");
    }

    if r.image_url.as_str() != "" {
        cmd.arg("--image-url").arg(&r.image_url); 
    }
    if r.preserved_paths.as_str() != "" {
        cmd.arg("--preserve-path").arg(&r.preserved_paths); 
    }
    if r.num_shards.as_str() != ""{
        cmd.arg("--num-shards").arg(&r.num_shards); 
    }
    if r.cpu_budget.as_str() != ""{
        cmd.arg("--cpu-budget").arg(&r.cpu_budget); 
    }
    if r.passphrase_file.as_str() != "" {
        cmd.arg("--passphrase-file").arg(&r.passphrase_file); 
    }

    let verbose = format!("-{}","v".repeat(r.verbose.into()));
    if r.verbose!=0 {
        cmd.arg(&verbose);
    }

    match cmd.spawn() {
        Ok(_) => {
            println!("FF chk spawned successfully");
            0
        },
        Err(e) => {
            println!("FF chk failed to spawn: {e:?}");
            1
        }
    }
}


