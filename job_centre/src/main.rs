mod models;
mod queue;

use models::{Request, Response};
use queue::{Job, QueueManager};
use shared::pool::ThreadPool;
use std::{
    collections::HashMap,
    io::{prelude::*, BufReader, Result},
    net::{TcpListener, TcpStream},
    sync::{Arc, Mutex},
    thread::{self},
    time::Duration,
};

fn main() -> Result<()> {
    let listener = TcpListener::bind("0.0.0.0:8080")?;
    let pool = ThreadPool::new(1000);
    let queue_manager = Arc::new(Mutex::new(QueueManager::new()));

    for stream in listener.incoming() {
        let stream = stream?;
        let manager = Arc::clone(&queue_manager);

        pool.execute(|_| {
            if let Err(e) = handle_connection(stream, manager) {
                eprintln!("Connection error: {}", e);
            }
        })
    }

    Ok(())
}

fn handle_connection(stream: TcpStream, queue_manager: Arc<Mutex<QueueManager>>) -> Result<()> {
    let reader = BufReader::new(&stream);
    let mut writer = &stream;

    let mut active_jobs: HashMap<u32, Job> = HashMap::new();

    for line in reader.lines() {
        let line = line?;

        match serde_json::from_str(&line) {
            Ok(Request::Put { job, queue, pri }) => {
                let id = queue_manager.lock().unwrap().add_job(queue, job, pri);
                writeln!(
                    writer,
                    "{}",
                    serde_json::to_string(&Response::Put {
                        status: "ok".to_string(),
                        id,
                    })?
                )?;
            }
            Ok(Request::Get { queues, wait }) => loop {
                {
                    if let Some(job) = queue_manager
                        .lock()
                        .unwrap()
                        .get_highest_priority_job(queues.clone())
                    {
                        writeln!(
                            writer,
                            "{}",
                            serde_json::to_string(&Response::Get {
                                status: "ok".to_string(),
                                id: job.id,
                                job: job.job.clone(),
                                pri: job.pri,
                                queue: job.queue_id.clone(),
                            })?
                        )?;

                        active_jobs.insert(job.id, job);
                        break;
                    }
                }

                if wait {
                    thread::sleep(Duration::from_secs(1));

                    stream.set_nonblocking(true)?;

                    let mut reader = BufReader::new(&stream);
                    if matches!(reader.fill_buf(), Ok(buf) if buf.is_empty()) {
                        break;
                    }

                    stream.set_nonblocking(false)?;
                } else {
                    writeln!(
                        writer,
                        "{}",
                        serde_json::to_string(&Response::Abort {
                            status: "no-job".to_string(),
                        })?
                    )?;
                    break;
                }
            },
            Ok(Request::Delete { id }) => {
                if active_jobs.contains_key(&id) {
                    active_jobs.remove(&id);
                }

                let status = queue_manager.lock().unwrap().delete_job(id);
                writeln!(
                    writer,
                    "{}",
                    serde_json::to_string(&Response::Delete { status })?
                )?;
            }
            Ok(Request::Abort { id }) => {
                if active_jobs.contains_key(&id) {
                    let job = active_jobs.remove(&id).unwrap();
                    let status = queue_manager.lock().unwrap().abort_job(job);

                    writeln!(
                        writer,
                        "{}",
                        serde_json::to_string(&Response::Abort { status })?
                    )?;
                } else {
                    writeln!(
                        writer,
                        "{}",
                        serde_json::to_string(&Response::Error {
                            status: "error".to_string(),
                            error: "Client has not been assigned job".to_string(),
                        })?
                    )?;
                }
            }
            Err(_) => {
                writeln!(
                    writer,
                    "{}",
                    serde_json::to_string(&Response::Error {
                        status: "error".to_string(),
                        error: "Invalid request".to_string(),
                    })?
                )?;
            }
        };
    }

    for job in active_jobs.into_values() {
        queue_manager.lock().unwrap().abort_job(job);
    }

    Ok(())
}
