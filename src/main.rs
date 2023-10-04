mod execute;

use anyhow::Result;
//use structopt::StructOpt;


use hyper::service::{make_service_fn, service_fn};
use hyper::{Body, Method, Request, Response, Server};
use std::convert::Infallible;
//use std::fs::File;
use std::os::unix::net::{UnixListener};
use std::io::{ErrorKind,Read};

use crate::execute::*;
/*
#[derive(StructOpt, PartialEq, Debug, Serialize)]
pub struct Daemon {
    #[structopt(short, long, parse(from_occurrences))]
    pub verbose: u8,
}
*/

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

            if run_execute(body_str)!=0 {
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
