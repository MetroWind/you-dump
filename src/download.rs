use std::process::{Command, Stdio};
use std::path::Path;
use std::thread::JoinHandle;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use serde::Serialize;
use serde::ser::Serializer;
use chrono::prelude::*;

use crate::error::Error;
use crate::config::Configuration;

fn serializeDateTime<S>(dt: &DateTime<Utc>, serializer: S) ->
    Result<S::Ok, S::Error> where S: Serializer,
{
    serializer.serialize_i64(dt.timestamp())
}

#[derive(Clone, Serialize)]
pub struct NewJob
{
    pub uri: String,
    #[serde(serialize_with = "serializeDateTime")]
    time_create: DateTime<Utc>,
}

#[derive(Serialize)]
pub enum StopReason
{
    Done, Error(Error),
}

#[derive(Serialize)]
pub struct StoppedJob
{
    pub uri: String,
    pub stop_reason: StopReason,
    #[serde(serialize_with = "serializeDateTime")]
    pub time_stopped: DateTime<Utc>,
}

#[derive(Serialize)]
pub struct OngoingJob
{
    pub uri: String,
    #[serde(serialize_with = "serializeDateTime")]
    time_start: DateTime<Utc>,
    #[serde(skip_serializing)]
    thread: JoinHandle<StoppedJob>,
    #[serde(skip_serializing)]
    finished: Arc<AtomicBool>,
}

impl NewJob
{
    pub fn new(uri: String) -> Self
    {
        Self {
            uri: uri,
            time_create: Utc::now(),
        }
    }
}

impl OngoingJob
{
    fn fromNewJob(job: NewJob, thread: JoinHandle<StoppedJob>,
                  finished: Arc<AtomicBool>) -> Self
    {
        Self {
            uri: job.uri,
            time_start: Utc::now(),
            thread: thread,
            finished: finished,
        }
    }

    pub fn finished(&self) -> bool
    {
        self.finished.load(Ordering::Relaxed)
    }

    pub fn result(self) -> StoppedJob
    {
        self.thread.join().unwrap()
    }
}

impl StoppedJob
{
    pub fn fromNewJob(job: NewJob, reason: StopReason) -> Self
    {
        Self {
            uri: job.uri,
            stop_reason: reason,
            time_stopped: Utc::now(),
        }
    }
}

fn tryDownload(job: &NewJob, config: Configuration) -> Result<(), Error>
{
    let dir = Path::new(&config.download_dir).canonicalize().map_err(
        |_| rterr!("Invalid download dir: {}", config.download_dir))?;
    if !dir.exists()
    {
        std::fs::create_dir_all(&dir).map_err(
            |e| rterr!("Failed to create download dir: {}", e))?;
    }
    let dir_str = dir.to_str().ok_or_else(
        || rterr!("Download dir is too weired"))?;
    let mut proc = Command::new(&config.ydl_exec)
        .args(["-P", dir_str, &job.uri])
        .stdout(Stdio::null())
        .spawn().map_err(
            |e| rterr!("Failed to spawn {}: {}", config.ydl_exec, e))?;

    let status = proc.wait().map_err(
        |e| rterr!("Failed to run {}: {}", config.ydl_exec, e))?;

    if status.success()
    {
        Ok(())
    }
    else if let Some(code) = status.code()
    {
        Err(rterr!("Failed with code {}", code))
    }
    else
    {
        Err(rterr!("Failed with signal"))
    }
}

fn download(job: NewJob, config: Configuration, finished: Arc<AtomicBool>) ->
    StoppedJob
{
    let result = match tryDownload(&job, config)
    {
        Ok(_) => StoppedJob::fromNewJob(job, StopReason::Done),
        Err(e) => StoppedJob::fromNewJob(job, StopReason::Error(e)),
    };
    finished.store(true, Ordering::Relaxed);
    result
}

pub fn createDownload(uri: String, config: Configuration) ->
    OngoingJob
{
    let job = NewJob::new(uri);
    let job2 = job.clone();
    let finished = Arc::new(AtomicBool::new(false));
    let finished2 = finished.clone();
    let proc = std::thread::spawn(move || {
        download(job2, config, finished2)
    });
    OngoingJob::fromNewJob(job, proc, finished)
}
