// Copyright 2021-2022 Parity Technologies (UK) Ltd.
// This file is part of Polkadot.

// Polkadot is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Polkadot is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Polkadot.  If not, see <http://www.gnu.org/licenses/>.

use futures::channel::oneshot;
use hyper::{
	header::CONTENT_TYPE,
	service::{make_service_fn, service_fn},
	Body, Method, Request, Response,
};
use lazy_static::lazy_static;
use prometheus::{
	labels, opts, register_counter, register_gauge, register_histogram_vec, Counter, Encoder,
	Gauge, HistogramVec, TextEncoder,
};

lazy_static! {
	pub static ref SUBMISSIONS_STARTED: Counter = register_counter!(opts!(
		"staking_miner_submissions_started",
		"Number of submissions started",
		labels! {"handler" => "all",}
	))
	.unwrap();
	pub static ref SUBMISSIONS_SUCCESS: Counter = register_counter!(opts!(
		"staking_miner_submissions_success",
		"Number of submissions finished successfully",
		labels! {"handler" => "all",}
	))
	.unwrap();
	pub static ref MINED_SOLUTION_DURATION: HistogramVec = register_histogram_vec!(
		"staking_miner_mining_duration_ms",
		"The mined solution time in milliseconds.",
		&["all"],
		vec![
			1.0,
			5.0,
			25.0,
			100.0,
			500.0,
			1_000.0,
			2_500.0,
			10_000.0,
			25_000.0,
			100_000.0,
			1_000_000.0,
			10_000_000.0,
		]
	)
	.unwrap();
	pub static ref SUBMIT_SOLUTION_AND_WATCH_DURATION: HistogramVec = register_histogram_vec!(
		"staking_miner_submit_and_watch_duration_ms",
		"The time in milliseconds it took to submit the solution to chain and to be included in block",
		&["all"],
		vec![
			1.0,
			5.0,
			25.0,
			100.0,
			500.0,
			1_000.0,
			2_500.0,
			10_000.0,
			25_000.0,
			100_000.0,
			1_000_000.0,
			10_000_000.0,
		]
	)
	.unwrap();
	pub static ref BALANCE: Gauge = register_gauge!(opts!(
		"staking_miner_balance",
		"The balance of the staking miner account",
		labels! {"handler" => "all",}
	))
	.unwrap();
	pub static ref RUNTIME_UPGRADES: Counter = register_counter!(opts!(
		"staking_miner_runtime_upgrades",
		"Number of runtime upgrades performed",
		labels! {"handler" => "all",}
	))
	.unwrap();
}

async fn serve_req(req: Request<Body>) -> Result<Response<Body>, hyper::Error> {
	let response = match (req.method(), req.uri().path()) {
		(&Method::GET, "/metrics") => {
			let mut buffer = vec![];
			let encoder = TextEncoder::new();
			let metric_families = prometheus::gather();
			encoder.encode(&metric_families, &mut buffer).unwrap();

			Response::builder()
				.status(200)
				.header(CONTENT_TYPE, encoder.format_type())
				.body(Body::from(buffer))
				.unwrap()
		},
		(&Method::GET, "/") => Response::builder().status(200).body(Body::from("")).unwrap(),
		_ => Response::builder().status(404).body(Body::from("")).unwrap(),
	};

	Ok(response)
}

pub fn run() -> oneshot::Sender<()> {
	let (tx, rx) = oneshot::channel();

	// For every connection, we must make a `Service` to handle all
	// incoming HTTP requests on said connection.
	let make_svc = make_service_fn(move |_conn| {
		// This is the `Service` that will handle the connection.
		// `service_fn` is a helper to convert a function that
		// returns a Response into a `Service`.
		async move { Ok::<_, std::convert::Infallible>(service_fn(move |req| serve_req(req))) }
	});

	let addr = ([127, 0, 0, 1], 3000).into();
	let server = hyper::Server::bind(&addr).serve(make_svc);

	log::info!("Started prometheus endpoint on http://{}", addr);

	let graceful = server.with_graceful_shutdown(async {
		rx.await.ok();
	});

	tokio::spawn(async move {
		if let Err(e) = graceful.await {
			log::warn!("server error: {}", e);
		}
	});

	tx
}
