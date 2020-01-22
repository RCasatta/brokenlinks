
use std::sync::mpsc::{Sender,channel};
use std::collections::HashSet;
use std::time::Duration;
use std::io::Read;
use clap::{Arg, App};
use url::{Url, Position};
use select::document::Document;
use select::predicate::Name;
use threadpool::ThreadPool;
use reqwest::blocking::Client;
use reqwest::header::CONTENT_TYPE;
use std::error::Error;


fn main() {
    let matches  = App::new("BrokenLinks")
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("timeout")
                .short("t")
                .long("timeout")
                .takes_value(true)
                .help("Timeout value example (default: 10)")
        )
        .arg(
            Arg::with_name("thread")
                .short("T")
                .long("thread")
                .takes_value(true)
                .help("Thread used (default: 20)")
        )
        .arg(
            Arg::with_name("BASE")
                .help("Sets the base domain (eg brokenlinks https://example.com)")
                .required(true)
                .index(1)
        )
        .get_matches();

    match run(matches) {
        Ok(_) => println!("Finish"),
        Err(e) => println!("{:?}",e),
    }
}

fn run(matches : clap::ArgMatches) -> Result<(), Box<dyn Error>> {
    let base = matches.value_of("BASE").unwrap();
    let timeout : u64 = matches.value_of("timeout").unwrap_or("10").parse().unwrap_or(10);
    let n_thread : usize = matches.value_of("thread").unwrap_or("4").parse().unwrap_or(4);
    let base = Url::parse(base)?;
    let pool = ThreadPool::new(n_thread);
    let mut url_done = HashSet::new();

    let (tx, rx) = channel();

    let cloned_base = base.clone();
    let cloned_tx = tx.clone();

    pool.execute(move|| {
        cloned_tx.send(cloned_base).expect("channel will be there waiting for the pool");
    });

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
            pool.execute(move|| {

                match get_url_and_extract(&url, &cloned_base, cloned_tx) {
                    Ok(_) => println!("OK {}", url),
                    Err(e) => println!("KO {} {}", url, e),
                }
            });

        }

    }
    eprintln!("{} urls", url_done.len());

    Ok(())
}

fn normalize(url : &url::Url) -> String {
    match url.fragment() {
        Some(_) => String::from(&url[..Position::BeforeFragment]), // TODO should be -1 ???
        None => url.to_string(),
    }
}

fn get_url_and_extract(url : &url::Url, base : &url::Url, tx : Sender<url::Url>)  -> Result<(), Box<dyn Error>> {

    let client = Client::new();
    let mut is_head = true;

    let response = match client.head(url.clone()).send() {
        Ok(response) => response,
        Err(_) => { // sometimes HEAD is not supported by the server
            is_head=false;
            client.get(url.clone()).send()?
        }
    };

    if response.status().is_success() {
        let is_internal = url.host() == base.host();
        let is_html = match response.headers().get(CONTENT_TYPE) {
            Some(content_type) => format!("{:?}", content_type ).contains("text/html"),
            None => false,
        };
        if is_internal && is_html {
            let mut response = match is_head {
                true => reqwest::blocking::get(url.clone())?,
                false => response,  // no need to redo the get if already done
            };
            let mut body = String::new();
            response.read_to_string(&mut body)?;
            let body = body.as_bytes();

            let elements = vec!( ("a","href"),("script","src"),("img","src"),("link","src") );
            for element in elements {
                Document::from_read(body)?
                    .find(Name(element.0))
                    .filter_map(|n| n.attr(element.1))
                    .for_each(|x| {
                        if let Some(url) = validate_and_make_full_url(&x, &base) {
                            let _ = tx.send(url);
                        }
                    });
            }
        }
    }

    Ok(())
}


fn validate_and_make_full_url(path_or_external : &str, base : &url::Url) -> Option<url::Url> {
    let url = match Url::parse(path_or_external) {
        Ok(url) => Some(url),
        _ => match base.join(path_or_external) {
            Ok(url) => Some(url),
            _ => None,
        }
    };

    match url {
        Some(url) => {
            if url.scheme()=="https" || url.scheme()=="http" {
                Some(url)
            } else {
                None
            }
        }
        _ => None,
    }
}
