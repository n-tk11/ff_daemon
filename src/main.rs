mod execute;

use anyhow::Result;
//use structopt::StructOpt;


use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
//use std::fs::File;
use std::os::unix::net::{UnixListener};
use std::io::{ErrorKind,Read,Write,BufWriter};
use std::{fs,process};
use std::fs::OpenOptions;

use crate::execute::*;
/*
#[derive(StructOpt, PartialEq, Debug, Serialize)]
pub struct Daemon {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
*/

use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
struct Opts {

    #[structopt(short,long)]
    port: Option<u16>,

    #[structopt(short,long,parse(from_os_str))]
    entry: Option<PathBuf>,

    #[structopt(short,long,parse(from_os_str))]
    decider_path: Option<PathBuf>,

    #[structopt(short,long)]
    kill_exit: bool,
}


#[tokio::main]
async fn main() {

    let opts = Opts::from_args();
    match opts.entry {
        Some(entry_path) => {
            entry_mode(entry_path,opts.decider_path,opts.kill_exit);
        },
        None => ()
    }
    let port_num: u16 = match opts.port {
        Some(pn) => pn,
        None => 7878
    };

    let make_svc = make_service_fn(move |_conn| {
        let exit_kill = opts.kill_exit.clone();
        async move{
            Ok::<_, Infallible>(service_fn(move |req| {
                handle_connection(req, exit_kill.clone())
            })) 
        }
    });

    let addr = ([0, 0, 0, 0], port_num).into();
    let server = Server::bind(&addr).serve(make_svc);

    
    write_status_to_pipe(0);
    println!("Listening on http://{}", addr);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    let _ = graceful.await;
}

async fn handle_connection(req: Request<Body>, exit_kill: bool) -> Result<Response<Body>, Infallible> {

    match (req.method(), req.uri().path()) {
        
        (&Method::POST, "/run") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

            if run_execute(body_str,false,exit_kill)!=0 {
               return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Fail to spawn child to run FF\n"))
                .unwrap());
            }
            
            match wait_child() {
                (0,_) => (),
                (2,ec) => {
                    let exit_msg = format!("App Exited with exit_code {}\n",ec);
                    return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(exit_msg.to_string()))
                .unwrap())
                }, 
                (_,_) => return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(""))
                .unwrap()) 
            } 

            write_status_to_pipe(1);

            Ok(Response::builder()
                .status(hyper::StatusCode::OK)
                .body(Body::from("App start successfully\n"))
                .unwrap())
        },
        (&Method::POST, "/checkpoint") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
            //let instance = new_chk_from_json(body_str);
            if checkpoint_execute(body_str)!=0 {
               return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Fail to spawn child to checkpoint FF\n"))
                .unwrap());
            }
            match wait_child() {
                (1,_) => (),
                (2,ec) => {
                let exit_msg = format!("App Exited with exit_code {}\n",ec);
                    return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(exit_msg.to_string()))
                .unwrap())
                }, 
                (_,_) => return Ok(Response::builder()
                .status(hyper::StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(""))
                .unwrap()) 
            } 
            
            write_status_to_pipe(2);

            Ok(Response::builder()
                .status(hyper::StatusCode::OK)
                .body(Body::from("App checkpointed\n"))
                .unwrap())
        },
        _ => {
            Ok(Response::builder()
                .status(hyper::StatusCode::BAD_REQUEST)
                .body(Body::empty())
                .unwrap())
        },
    }
}


async fn shutdown_signal() {
    // Wait for the CTRL+C signal
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C signal handler");
}

fn wait_child() -> (u8,String) {
    let socket_path = "/tmp/ff.sock";
    match std::fs::remove_file(socket_path) {
        Ok(_) => println!("Previous socket file removed"),
        Err(e) if e.kind() == ErrorKind::NotFound => (),
        _ => return (0,String::from("0")),
    }

    let listener = match UnixListener::bind(socket_path) {
        Ok(sock) => sock,
        Err(e) => {
            println!("Couldn't connect: {e:?}");
            return (3,String::from("0"));
        }
    };
    println!("Waiting for child response");
    match listener.accept() {
        Ok((mut stream, _addr)) => {
            let mut buffer = Vec::new();
            let mut byte = [0; 1];
            loop {
                stream.read_exact(&mut byte).unwrap();
                if byte[0] == b'\n' {  
                    break;
                }
                buffer.push(byte[0]);
            }
            let message = String::from_utf8(buffer).unwrap();
            let msg_col = message.split(" ").collect::<Vec<&str>>();
            match msg_col[0] {
                "app_started" => {
                    println!("Got a socket connection, app started"); 
                    return (0,String::from("0"));                
                }, 
                "app_checkpointed" => {
                    println!("Got a socket connection, app checkpointed");
                    return (1,String::from("0"));
                },
                "app_exiting" => {
                    println!("App exited with exit_code {}",msg_col[1]);
                    return (2,String::from(msg_col[1]));
                }, 
                _ => {
                    println!("Unknown Message to sock");
                    return (3,String::from("0"));
                }
            }
        },
        Err(e) => {
                println!("accept function failed: {e:?}");
                return (3,String::from("0"));
        },
    } 
}


fn entry_mode(entry_path: PathBuf, decider_path: Option<PathBuf>,exit_kill: bool) {
    let decider_path = match decider_path {
        Some(path) => path,
        None => PathBuf::from("/decider.txt"),
    };
    let decider = match fs::read_to_string(decider_path) {
        Ok(contents) => {
            contents
        }
        Err(error) => match error.kind() {
            ErrorKind::NotFound => {
                println!("Decider file not found");
                String::from("2")
            }
            _ => {
                panic!("Error reading file: {}", error);
            }
        },
    };
    if let Some(first_char) = decider.chars().next() {
        if first_char == '2' {
            println!("Continue to standby mode");
            write_status_to_pipe(0);
        } else  {
            let entry_data = match fs::read_to_string(entry_path) {
                Ok(content) => content,
                Err(_) => {
                   println!("Error reading Entry data,Continue to standby mode");
                   write_status_to_pipe(0);
                   return 
                }
            };
            let is_begin;
            if first_char == '0' {
                println!("Will Start From Scratch!!");
                is_begin = true;
            }else {
                is_begin = false;
            }
            if run_execute(entry_data,is_begin,exit_kill)!=0 {
                println!("Cannot start the app going to enter standby mode");
                write_status_to_pipe(0);
            } else {
                //This match wait for app to start.
                match wait_child() {
                    (0,_) => (),
                    (2,ec) => {
                        println!("App Exited with exit_code {}\n",ec);
                        let ec_int = ec.parse::<i32>().unwrap();
                        process::exit(ec_int);
                    }, 
                    (_,_) => {
                        println!("Unknown Error!!");
                        process::exit(1);
                    }
                } 
                write_status_to_pipe(1); 
            }
        }
    } else {
        println!("Not a string");
    }
}

fn write_status_to_pipe(status: u8) {
//Write to named pipe to let controller know
    let pipe_path = "/opt/controller/comms/status";
    let file = match OpenOptions::new().write(true).truncate(true).create(true).open(pipe_path) {
        Ok(file) => file,
        Err(_) => return,
    };
    let mut file = BufWriter::new(file);
    if status == 0 {
        file.write_all(b"0").unwrap();
    }else if status == 1 {
        file.write_all(b"1").unwrap();
    }else {
        file.write_all(b"2").unwrap();
    }
}
