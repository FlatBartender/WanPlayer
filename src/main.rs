use minimp3::{Decoder, Frame, Error};
use cpal::{
    traits::{HostTrait, DeviceTrait, StreamTrait},
    Stream,
};
use hyper::{
    body::HttpBody,
};
use tokio::{
    io::{
        duplex,
        DuplexStream,
        AsyncRead,
        AsyncReadExt,
        AsyncWrite,
        AsyncWriteExt,
    },
    sync::mpsc::{
        UnboundedSender,
        UnboundedReceiver,
        unbounded_channel,
    }
};

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
        tx.write_all(slice);
    }
}

async fn decoder_thread(tx: UnboundedSender<i16>, rx: DuplexStream, volume: Arc<AtomicU8>) {
    let mut decoder = Decoder::new(rx);

    loop {
        match decoder.next_frame_future().await {
            Ok(Frame {data, ..}) => {
                let factor = volume.load(Ordering::Relaxed) as f32;
                data.iter()
                    .map(|s| (*s as f32 / 100.0 * factor) as i16)
                    .for_each(|s| tx.send(s).expect("Failed to send sample"));
            },
            Err(Error::Eof) => break,
            Err(e) => println!("{:?}", e),
        }
    }
}
fn player_init(mut rx: UnboundedReceiver<i16>) -> Stream {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to acquire default output device");
    let mut configs = device.supported_output_configs().expect("Failed to list supported output configs");
    let config = configs.next().expect("Failed to get output config")
        .with_max_sample_rate().config();

    let stream = device.build_output_stream(&config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
                data.iter_mut().for_each(|d| *d = rx.blocking_recv().unwrap_or(0));
            },
            move |err| {
                println!("{}", err);
            },
    ).expect("Failed to create stream");

    stream
}

#[tokio::main]
async fn main() {
    let volume = Arc::new(AtomicU8::new(100));

    let (stream_tx, stream_rx) = duplex(2usize.pow(22));
    let stream_handle = tokio::spawn(stream_thread(stream_tx));

    let (decoder_tx, decoder_rx) = unbounded_channel();
    let decoder_handle = tokio::spawn(decoder_thread(decoder_tx, stream_rx, volume.clone()));

    let stream = player_init(decoder_rx);

    stream.play().expect("Couldn't start playback");

    tokio::try_join!(stream_handle, decoder_handle).expect("Failed to join tasks");
}
