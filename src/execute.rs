use serde::{Serialize, Deserialize};
use serde_json;
use std::process::{Command,Child};
use std::thread;
use shlex::Shlex;


//Run Json Instance
#[derive(Debug, Serialize, Deserialize)]
struct Runj {
    app_args: String,
    image_url: String,
    #[serde(default = "default_blanks")]
    on_app_ready: String,
    #[serde(default = "default_blanks")]
    passphrase_file: String,
    #[serde(default = "default_blanks")]
    preserved_paths: String,
    #[serde(default = "default_false")]
    no_restore: bool,
    #[serde(default = "default_false")]
    allow_bad_image: bool,
    #[serde(default = "default_false")]
    leave_stopped: bool,
    #[serde(default = "default_zero")]
    verbose: u8,
    #[serde(default = "default_blankv")]
    envs: Vec<String>,
}

//Checkpoint Json Instance
#[derive(Debug, Serialize, Deserialize)]
struct Checkpointj {
    #[serde(default = "default_blanks")]
    image_url: String,
    #[serde(default = "default_blanks")]
    passphrase_file: String,
    #[serde(default = "default_blanks")]
    preserved_paths: String,
    #[serde(default = "default_false")]
    leave_running: bool,
    #[serde(default = "default_blanks")]
    num_shards: String,
    #[serde(default = "default_blanks")]
    cpu_budget: String,
    #[serde(default = "default_zero")]
    verbose: u8,
    #[serde(default = "default_blankv")]
    envs: Vec<String>,
}

pub fn run_execute(body_str: String, is_begin: bool) -> u8 {

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
    if r.no_restore || is_begin{
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

    for env in r.envs.into_iter() {
        let parts: Vec<&str> = env.split('=').collect();
        cmd.env(parts[0],parts[1]);
    }
    
    let _ = thread::spawn(move ||{
        let mut child_process: Child = cmd.spawn().expect("Failed to start ff.run process");

        let status = child_process.wait().expect("Failed to wait for ff.run process");
        
        println!("ff.chk process exited with status: {:?}", status);
    });

    return 0;
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

    for env in r.envs.into_iter() {
        let parts: Vec<&str> = env.split('=').collect();
        cmd.env(parts[0],parts[1]);
    }
   
    let _ = thread::spawn(move ||{
        let mut child_process: Child = cmd.spawn().expect("Failed to start ff.chk process");

        let status = child_process.wait().expect("Failed to wait for ff.chk process");
        
        println!("ff.chk process exited with status: {:?}", status);
    });
    return 0;
}

fn default_blanks() -> String {
    "".to_string()
}

fn default_false() -> bool {
    false
}

fn default_zero() -> u8 {
    0
}
fn default_blankv() -> Vec<String> {
    vec![]
}