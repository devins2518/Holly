mod holly;

use std::{env, time, thread::sleep};
use reqwest::blocking::multipart;
use rust_socketio::{ClientBuilder, Payload, Client};
use std::io::Read;

fn main() {
	let args: Vec<String> = env::args().collect();

	let form = multipart::Form::new()
		.text("skin", "blueberry_v1_7_0")
		.text("resolution", "1280x720")
		.text("username", "Holly")
		.file("replayFile", &args[1]).unwrap();

	let client = reqwest::blocking::Client::new();
	let mut res = client.post("https://ordr-api.issou.best/renders")
		.multipart(form)
		.send().unwrap();

	// check if status code is 429
	if res.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
	    // get X-RateLimit-Reset header
	    let now = time::Instant::now();
	    let stamp = res.headers().get("X-RateLimit-Reset").unwrap().to_str().unwrap().parse::<u64>().unwrap();
	    let duration = time::Duration::from_secs(stamp) - now.elapsed();

	    println!("{}", duration.as_secs());
		println!("Ratelimited, try again later...");
		return;
	}

	let mut body = String::new();
	res.read_to_string(&mut body).unwrap();

	let r: holly::SentRender = serde_json::from_str(&body).unwrap();
	let render_id = r.renderID;

    let done_callback = move |payload: Payload, socket: Client| {
        let data = match payload {
            Payload::String(s) => s,
            _ => "".to_string(),
        };

        let p: holly::RenderDone = serde_json::from_str(&data).unwrap();
        if p.renderID == render_id {
            println!("Render finished!");
            socket.disconnect();
            return;
        }
    };

	let progress_callback = move |payload: Payload, _: Client| {
		let data = match payload {
			Payload::String(s) => s,
			_ => "".to_string(),
		};
		let p: holly::RenderProgress = serde_json::from_str(&data).unwrap();
		if p.renderID == render_id {
			println!("{}", p.progress);
		}
	};

	let failed_callback = move |payload: Payload, socket: Client| {
        let data = match payload {
            Payload::String(s) => s,
            _ => "".to_string(),
        };

        let p: holly::RenderFailed = serde_json::from_str(&data).unwrap();
        if p.renderID == render_id {
            println!("Render failed. Error: {} ({})", p.errorMessage, p.errorCode);
            socket.disconnect();
            return;
        }
    };

	ClientBuilder::new("https://ordr-ws.issou.best")
		 .on("render_progress_json", progress_callback)
		 .on("render_done_json", done_callback)
		 .on("render_failed_json", failed_callback)
		 .connect()
		 .expect("Connection failed");

	sleep(time::Duration::from_secs(1000));
}
