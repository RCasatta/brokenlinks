use clap::{command, Parser};
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use select::document::Document;
use select::predicate::Name;
use std::collections::HashSet;
use std::error::Error;
use std::io::Read;
use std::sync::mpsc::{channel, Sender};
use std::time::Duration;
use threadpool::ThreadPool;
use url::{Position, Url};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Timeout for the request
    #[arg(short, long, default_value_t = 10)]
    timeout: u8,

    /// Number of thread to use
    #[arg(short, long, default_value_t = 4)]
    thread: u8,

    /// Base url
    #[arg(short, long)]
    base: String,
}

fn main() {
    let args = Args::parse();

    match run(args) {
        Ok(_) => println!("Finish"),
        Err(e) => println!("{:?}", e),
    }
}

fn run(args: Args) -> Result<(), Box<dyn Error>> {
    let base = &args.base;
    let timeout: u64 = args.timeout as u64;
    let n_thread: usize = args.thread as usize;
    let base = Url::parse(base)?;
    let pool = ThreadPool::new(n_thread);
    let mut url_done = HashSet::new();

    let (tx, rx) = channel();

    let cloned_base = base.clone();
    let cloned_tx = tx.clone();

    pool.execute(move || {
        cloned_tx
            .send(cloned_base)
            .expect("channel will be there waiting for the pool");
    });
    let client = Client::new();
    loop {
        let url = match rx.recv_timeout(Duration::from_secs(timeout)) {
            Ok(url) => url,
            Err(_) => break,
        };
        let url_normalized = normalize(&url);
        if !url_done.contains(&url_normalized) {
            url_done.insert(url_normalized);
            let cloned_tx = tx.clone();
            let cloned_base = base.clone();
            let client = client.clone();
            pool.execute(move || {
                match get_url_and_extract(&url, &cloned_base, cloned_tx, client) {
                    Ok(s) => println!(
                        "OK {} {}",
                        url,
                        s.map(|e| e.to_string()).unwrap_or_else(|| "".to_owned())
                    ),
                    Err(e) => println!("KO {} {}", url, e),
                }
            });
        }
    }
    eprintln!("{} urls", url_done.len());

    Ok(())
}

fn normalize(url: &url::Url) -> String {
    match url.fragment() {
        Some(_) => String::from(&url[..Position::BeforeFragment]), // TODO should be -1 ???
        None => url.to_string(),
    }
}

fn get_url_and_extract(
    url: &url::Url,
    base: &url::Url,
    tx: Sender<url::Url>,
    client: Client,
) -> Result<Option<usize>, Box<dyn Error>> {
    let mut is_head = true;

    let response = match client.head(url.clone()).send() {
        Ok(response) => response,
        Err(_) => {
            // sometimes HEAD is not supported by the server
            is_head = false;
            client.get(url.clone()).send()?
        }
    };
    let mut html_size = None;

    if response.status().is_success() {
        let is_internal = url.host() == base.host();
        let is_html = match response.headers().get(CONTENT_TYPE) {
            Some(content_type) => format!("{:?}", content_type).contains("text/html"),
            None => false,
        };
        if is_internal && is_html {
            let mut response = match is_head {
                true => reqwest::blocking::get(url.clone())?,
                false => response, // no need to redo the get if already done
            };
            let mut body = String::new();
            response.read_to_string(&mut body)?;
            let body = body.as_bytes();
            html_size = Some(body.len());

            let elements = vec![
                ("a", "href"),
                ("script", "src"),
                ("img", "src"),
                ("link", "src"),
            ];
            for element in elements {
                Document::from_read(body)?
                    .find(Name(element.0))
                    .filter_map(|n| n.attr(element.1))
                    .filter(|u| !u.contains('#'))
                    .for_each(|x| {
                        if let Some(url) = validate_and_make_full_url(&x, &base) {
                            let _ = tx.send(url);
                        }
                    });
            }
        }
    }

    Ok(html_size)
}

fn validate_and_make_full_url(path_or_external: &str, base: &url::Url) -> Option<url::Url> {
    let url = match Url::parse(path_or_external) {
        Ok(url) => Some(url),
        _ => match base.join(path_or_external) {
            Ok(url) => Some(url),
            _ => None,
        },
    };

    match url {
        Some(url) => {
            if url.scheme() == "https" || url.scheme() == "http" {
                Some(url)
            } else {
                None
            }
        }
        _ => None,
    }
}
