use minimp3::{Frame, Error};
use cpal::{
    traits::{HostTrait, DeviceTrait, StreamTrait},
    Stream,
};
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
use fltk::{
    app::*,
    window::*,
    button::*,
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
    NeverEnding(()),   // Designed for futures that never stop
}

#[derive(PartialEq, Eq)]
enum StreamStatus {
    Playing,
    Paused,
}

struct Player {
    volume: Arc<AtomicU8>,
    stream: Stream,
    stream_status: StreamStatus,
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

async fn decoder_thread(mut tx: Producer<i16>, rx: DuplexStream, volume: Arc<AtomicU8>) {
    let mut decoder = minimp3::Decoder::new(rx);

    loop {
        match decoder.next_frame_future().await {
            Ok(Frame {data, ..}) => {
                let factor = volume.load(Ordering::Relaxed) as f32 / 100.0;
                let mut iter = data.into_iter()
                    .map(|s| (s as f32 * factor) as i16);
                tx.push_iter(&mut iter);
            },
            Err(Error::Eof) => break,
            Err(e) => println!("{:?}", e),
        }
    }
}
fn player_init(mut rx: Consumer<i16>) -> Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to acquire default output device");
    let mut configs = device.supported_output_configs().expect("Failed to list supported output configs");
    let config = configs.next().expect("Failed to get output config")
        .with_max_sample_rate().config();

    let stream = device.build_output_stream(&config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                rx.pop_slice(data);
            },
            move |err| {
                println!("{}", err);
            },
    ).expect("Failed to create stream");

    stream
}

#[tokio::main]
async fn main() {
    let volume = Arc::new(AtomicU8::new(10));

    let (stream_rx, stream_tx) = duplex(2usize.pow(16));
    let stream_handle = tokio::spawn(stream_thread(stream_tx));

    let (decoder_tx, decoder_rx) = RingBuffer::new(2usize.pow(22)).split();
    let decoder_handle = tokio::spawn(decoder_thread(decoder_tx, stream_rx, volume.clone()));

    let stream = player_init(decoder_rx);

    let app = App::default();
    let mut win = Window::new(100, 100, 400, 300, "Wan Player");
    let mut but = Button::new(10, 10, 100, 100, "Play/Pause");

    win.end();
    win.show();

    let (s, r) = fltk::app::channel::<PlayerMessage>();

    but.emit(s, PlayerMessage::PlayPause);

    let mut playing = StreamStatus::Paused;

    while app.wait() {
        match r.recv() {
            Some(PlayerMessage::PlayPause) if playing == StreamStatus::Paused => {
                stream.play().expect("Failed to play stream");
                playing = StreamStatus::Playing;
            },
            Some(PlayerMessage::PlayPause) if playing == StreamStatus::Paused => {
                stream.pause().expect("Failed to pause stream");
                playing = StreamStatus::Paused;
            },
            _ => (),
        }
    }
}
