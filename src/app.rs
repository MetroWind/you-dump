use std::sync::{Arc, Mutex};

use log::error as log_error;
use log::{info, debug};

use warp::{Filter, Reply};
use warp::http::status::StatusCode;
use warp::reply::Response;
use serde_json::json;
use serde::Deserialize;

// use crate::rterr;
use crate::error::Error;
use crate::config::Configuration;
use crate::download::{OngoingJob, StoppedJob, createDownload};

type OngoingJobs = Arc<Mutex<Vec<OngoingJob>>>;
type StoppedJobs = Arc<Mutex<Vec<StoppedJob>>>;

static INDEX_HTML: &'static str = include_str!("../index.html");

trait ToResponse
{
    fn toResponse(self) -> Response;
}

impl ToResponse for Result<String, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(s) => warp::reply::html(s).into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            },
        }
    }
}

impl ToResponse for Result<&'static str, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(s) => warp::reply::html(s).into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            },
        }
    }
}

impl ToResponse for Result<serde_json::Value, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(j) => warp::reply::json(&j).into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            },
        }
    }
}

impl ToResponse for Result<Response, Error>
{
    fn toResponse(self) -> Response
    {
        match self
        {
            Ok(s) => s.into_response(),
            Err(e) => {
                log_error!("{}", e);
                warp::reply::with_status(
                e.to_string(), StatusCode::INTERNAL_SERVER_ERROR)
                    .into_response()
            }
        }
     }
}

pub struct App
{
    config: Configuration,
    ongoing_jobs: OngoingJobs,
    stopped_jobs: StoppedJobs,
}

fn index() -> Result<&'static str, Error>
{
    Ok(INDEX_HTML)
}

fn listJobs(ongoing: &OngoingJobs, stopped: &StoppedJobs) ->
    Result<serde_json::Value, Error>
{
    let mut jobs_ongoing = ongoing.lock().unwrap();
    let mut jobs_stopped = stopped.lock().unwrap();
    let mut i = 0;

    loop
    {
        if i >= jobs_ongoing.len()
        {
            break;
        }
        if jobs_ongoing[i].finished()
        {
            let result: StoppedJob = jobs_ongoing.remove(i).result();
            jobs_stopped.push(result);
        }
        else
        {
            i += 1;
        }
    }

    Ok(json!({"ongoing": *jobs_ongoing, "stopped": *jobs_stopped}))
}

#[derive(Deserialize)]
struct CreateJobRequest
{
    uri: String,
}

fn createJob(req: CreateJobRequest, config: &Configuration,
             ongoing: &OngoingJobs) -> Result<serde_json::Value, Error>
{
    let job = createDownload(req.uri, config.clone());
    let mut jobs_ongoing = ongoing.lock().unwrap();
    jobs_ongoing.push(job);
    Ok(json!("ok"))
}

impl App
{
    pub fn new(config: Configuration) -> Self
    {
        Self {
            config: config,
            ongoing_jobs: Arc::new(Mutex::new(Vec::new())),
            stopped_jobs: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn serve(self) -> Result<(), Error>
    {
        let index = warp::get().and(warp::path::end()).map(move || {
            index().toResponse()
        });

        let jobs_ongoing = self.ongoing_jobs.clone();
        let jobs_stopped = self.stopped_jobs.clone();
        let list_jobs = warp::get().and(warp::path("api"))
            .and(warp::path("jobs")).map(move || {
                listJobs(&jobs_ongoing, &jobs_stopped).toResponse()
            });

        let config = self.config.clone();
        let jobs_ongoing = self.ongoing_jobs.clone();
        let create_job = warp::post().and(warp::path("api"))
            .and(warp::path("new_job")).and(warp::body::json())
            .map(move |req: CreateJobRequest| {
                createJob(req, &config, &jobs_ongoing).toResponse()
            });

        let addr = std::net::SocketAddr::new(
            self.config.listen_address.parse().map_err(
                |_| rterr!("Invalid listen address: {}",
                           self.config.listen_address))?,
            self.config.listen_port);

        info!("Listening at {}:{}...", self.config.listen_address,
              self.config.listen_port);
        if let Some(d) = self.config.static_dir
        {
            let statics = warp::path("static").and(warp::fs::dir(d));
            warp::serve(statics.or(index).or(list_jobs).or(create_job))
                .run(addr).await;
        }
        else
        {
            warp::serve(index.or(list_jobs).or(create_job))
                .run(addr).await;
        }
        Ok(())
    }
}
