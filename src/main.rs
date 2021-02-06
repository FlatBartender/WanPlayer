use minimp3::{Frame, Error};
use cpal::{
    traits::{HostTrait, DeviceTrait, StreamTrait},
    Stream,
};
use serde::Deserialize;
use tokio::io::{
    duplex,
    DuplexStream,
    AsyncWriteExt,
};
use ringbuf::{
    RingBuffer,
    Producer,
    Consumer,
};
use hyper::body::HttpBody;

use std::sync::{
    Arc,
    atomic::{
        AtomicU8,
        Ordering,
    }
};

#[derive(Debug, Clone)]
enum PlayerMessage {
    PlayPause,
    VolumeChanged,
}

#[derive(PartialEq, Eq)]
enum StreamStatus {
    Playing,
    Paused,
}

async fn stream_thread(mut tx: DuplexStream) {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::client::Client::builder()
        .build::<_, hyper::Body>(https);

    let mut res = client.get(hyper::Uri::from_static("https://stream.gensokyoradio.net/1/"))
        .await
        .expect("Failed to request stream");

    while let Some(chunk) = res.data().await {
        let slice = &chunk.expect("Failed to convert response body to byte chunk")[..];
        tx.write_all(slice).await.expect("Failed to send stream to decoder");
    }
}

async fn decoder_thread(mut tx: Producer<i16>, rx: DuplexStream) {
    let mut decoder = minimp3::Decoder::new(rx);

    loop {
        match decoder.next_frame_future().await {
            Ok(Frame {data, ..}) => {
                let mut iter = data.into_iter();
                tx.push_iter(&mut iter);
            },
            Err(Error::Eof) => break,
            Err(e) => println!("{:?}", e),
        }
    }
}
fn player_init(mut rx: Consumer<i16>, volume: Arc<AtomicU8>) -> Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to acquire default output device");
    let mut configs = device.supported_output_configs().expect("Failed to list supported output configs");
    let config = configs.next().expect("Failed to get output config")
        .with_max_sample_rate().config();

    let stream = device.build_output_stream(&config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            let factor = volume.load(Ordering::Relaxed) as f32 / 100.0;
            rx.pop_slice(data);
            data.iter_mut().for_each(|d| *d = (*d as f32 * factor) as i16);
        },
        move |err| {
            println!("{}", err);
        },
    ).expect("Failed to create stream");

    stream
}

struct ApiClient {
    client: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl ApiClient {
    pub fn new() -> ApiClient {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::client::Client::builder()
            .build::<_, hyper::Body>(https);
        ApiClient {
            client
        }
    }

    pub async fn get_song_info(&mut self) -> GRApiAnswer {
        let res = self.client.get(hyper::Uri::from_static("https://gensokyoradio.net/json/"))
            .await
            .expect("Failed to request song info");

        let data = hyper::body::to_bytes(res.into_body()).await.expect("Failed to collect song info request body");

        serde_json::from_slice(&data[..]).expect("Failed to parse song info")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
struct SongInfo {
    title: String,
    artist: String,
    album: String,
    year: String,
    circle: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
struct SongTimes {
    duration: String,
    played: u16,
    remaining: u16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
struct GRApiAnswer {
    songinfo: SongInfo,
    songtimes: SongTimes,
}

const PLAY_SVG: &str = include_str!("resources/play.svg");

#[tokio::main]
async fn main() {
    let volume = Arc::new(AtomicU8::new(10));

    let (stream_rx, stream_tx) = duplex(2usize.pow(16));
    let stream_handle = tokio::spawn(stream_thread(stream_tx));

    let (decoder_tx, decoder_rx) = RingBuffer::new(2usize.pow(18)).split();
    let decoder_handle = tokio::spawn(decoder_thread(decoder_tx, stream_rx));

    let stream = player_init(decoder_rx, volume.clone());

    let mut playing = StreamStatus::Paused;
    let mut api_client = ApiClient::new();

    tokio::try_join!(stream_handle, decoder_handle).unwrap();
}
