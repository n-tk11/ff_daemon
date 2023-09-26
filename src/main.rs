
use anyhow::Result;
//use structopt::StructOpt;


use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
use std::fs::File;
use std::os::unix::net::UnixListener;

use serde::{Serialize, Deserialize};
use serde_json;
use std::process::Command;
use shlex::Shlex;
/*
#[derive(StructOpt, PartialEq, Debug, Serialize)]
pub struct Daemon {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
*/

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

#[tokio::main]
async fn main() {
    let addr = ([127, 0, 0, 1], 7878).into();
    let make_svc =
        make_service_fn(|_conn| async { Ok::<_, Infallible>(service_fn(handle_connection)) });

    let server = Server::bind(&addr).serve(make_svc);

    println!("Listening on http://{}", addr);

    let graceful = server.with_graceful_shutdown(shutdown_signal());

    // Run this server for... forever!
    let _ = graceful.await;
}

async fn handle_connection(req: Request<Body>) -> Result<Response<Body>, Infallible> {
    match (req.method(), req.uri().path()) {
       
        (&Method::POST, "/run") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let body_str = String::from_utf8(body_bytes.to_vec()).unwrap();

            //call run and wait till app started
            let r: Runj = serde_json::from_str(&body_str).unwrap();
            //let splited_args: Vec<String> = Shlex::new(&r.app_args).collect();
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
            
             
            let _ = cmd.spawn().expect("fastfreeze fail to start");
            /*
            {
                let file = match File::create("/tmp/ff.sock") {
                                    Ok(file) => file,
                                    Err(err) => {
                                        eprintln!("Error creating named pipe: {:?}", err);
                                        return Ok(Response::builder()
                                            .status(400)
                                            .body(Body::from("Cannot create socket"))
                                            .unwrap());
                                    }
                                };
            }
            */
            let listener = match UnixListener::bind("/tmp/ff.sock") {
                                Ok(sock) => sock,
                                Err(e) => {
                                    println!("Couldn't connect: {e:?}");
                                    return Ok(Response::builder()
                                        .status(400)
                                        .body(Body::from("Cannot open socket"))
                                        .unwrap());
                                }
                            };

            match listener.accept() {
                Ok(_) => println!("Got a connection, app started"),
                Err(e) => println!("accept function failed: {e:?}"),
            }

            Ok(Response::builder()
                .status(hyper::StatusCode::OK)
                .body(Body::from("App start successfully\n"))
                .unwrap())
        },
        (&Method::POST, "/checkpoint") => {
            let body_bytes = hyper::body::to_bytes(req.into_body()).await.unwrap();
            let _body_str = String::from_utf8(body_bytes.to_vec()).unwrap();
            //let instance = new_chk_from_json(body_str);
         
            //call checkpoint and wait till checkpointed

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
