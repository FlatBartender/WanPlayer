use minimp3::{Frame, Error};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
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
    mpsc::{
        channel,
        Receiver,
        Sender,
    },
    atomic::{
        AtomicU8,
        Ordering,
    }
};

use crate::gensokyo_radio::GR_STREAM;

pub enum PlayerControl {
    Volume(u8),
    Play,
    Pause,
}

async fn stream_thread(mut tx: DuplexStream) {
    let https = hyper_tls::HttpsConnector::new();
    let client = hyper::client::Client::builder()
        .build::<_, hyper::Body>(https);

    let mut res = client.get(hyper::Uri::from_static(GR_STREAM))
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

fn player_init(mut data_rx: Consumer<i16>, command_rx: Receiver<PlayerControl>) {
    let host = cpal::default_host();
    let device = host.default_output_device().expect("Failed to acquire default output device");
    let mut configs = device.supported_output_configs().expect("Failed to list supported output configs");
    let config = configs.next().expect("Failed to get output config")
        .with_max_sample_rate().config();

    let volume = Arc::new(AtomicU8::new(10));
    let cpal_volume = volume.clone();

    let stream = device.build_output_stream(&config,
        move |data: &mut [i16], _: &cpal::OutputCallbackInfo| {
            let factor = cpal_volume.load(Ordering::Relaxed) as f32 / 100.0;
            data_rx.pop_slice(data);
            data.iter_mut().for_each(|d| *d = (*d as f32 * factor) as i16);
        },
        move |err| {
            println!("{}", err);
        },
    ).expect("Failed to create stream");

    while let Ok(command) = command_rx.recv() {
        match command {
            PlayerControl::Volume(v) => volume.store(v, Ordering::Relaxed),
            PlayerControl::Play => stream.play().expect("Failed to play stream"),
            PlayerControl::Pause => stream.pause().expect("Failed to pause stream"),
        }
    }
}

pub fn setup_pipeline() -> Sender<PlayerControl> {
    let (stream_rx, stream_tx) = duplex(2usize.pow(16));
    let (decoder_tx, decoder_rx) = RingBuffer::new(2usize.pow(18)).split();
    let (player_tx, player_rx) = channel();

    tokio::spawn(async move {
        tokio::spawn(stream_thread(stream_tx));
        tokio::spawn(decoder_thread(decoder_tx, stream_rx));
        std::thread::spawn(move || player_init(decoder_rx, player_rx));
    });

    player_tx
}
