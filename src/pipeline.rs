use minimp3::{Frame, Error};
use cpal::traits::{HostTrait, DeviceTrait, StreamTrait};
use tokio::io::{
    duplex,
    DuplexStream,
    AsyncWriteExt,
};
use tokio::sync::Semaphore;
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

#[derive(Debug)]
pub enum PlayerControl {
    Volume(u8),
    Play,
    Pause,
}

enum PlaybackControl {
    Volume(u8),
    Play,
    Stop,
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

async fn decoder_thread(mut tx: Producer<i16>, rx: DuplexStream, sem: Arc<Semaphore>) {
    let mut decoder = minimp3::Decoder::new(rx);

    loop {
        match decoder.next_frame_future().await {
            Ok(Frame {data, ..}) => {
                let permit = sem.acquire_many(data.len() as u32).await.expect("Failed to acquire ringbuffer semaphore");
                tx.push_iter(&mut data.into_iter());
                permit.forget();
            },
            Err(Error::Eof) => {},
            Err(e) => panic!("An error happened while waiting for the next frame in decoder: {}", e),
        }
    }
}

fn playback_init(mut data_rx: Consumer<i16>, playback_control_rx: Receiver<PlaybackControl>, sem: Arc<Semaphore>) {
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
            let written = data_rx.pop_slice(data);
            sem.add_permits(written);
            data.iter_mut().for_each(|d| *d = (*d as f32 * factor) as i16);
        },
        move |err| {
            println!("{}", err);
        },
        ).expect("Failed to create stream");

    while let Ok(command) = playback_control_rx.recv() {
        match command {
            PlaybackControl::Volume(v) => {
                volume.store(v, Ordering::Relaxed);
            },
            PlaybackControl::Play => stream.play().expect("Failed to play stream"),
            PlaybackControl::Stop => return,
        }
    }
}

pub fn setup_pipeline() -> tokio::sync::mpsc::UnboundedSender<PlayerControl> {

    let (player_tx, mut player_rx) = tokio::sync::mpsc::unbounded_channel();

    tokio::spawn(async move {
        let mut volume = 10;
        let mut decoder = None;
        let mut stream = None;
        let mut playback_control_tx: Option<Sender<PlaybackControl>> = None;

        while let Some(msg) = player_rx.recv().await {
            match msg {
                PlayerControl::Volume(v) => {
                    volume = v;
                    if let Some(ref pctx) = playback_control_tx {
                        pctx.send(PlaybackControl::Volume(v)).expect("Failed to send volume to thread");
                    }
                },
                PlayerControl::Play => {
                    let (pctx, playback_control_rx) = channel();
                    playback_control_tx = Some(pctx);
                    let (stream_rx, stream_tx) = duplex(2usize.pow(12));
                    let rb_size = 2usize.pow(12);
                    let (decoder_tx, decoder_rx) = RingBuffer::new(rb_size).split();
                    let sem = Arc::new(Semaphore::new(rb_size));

                    stream = Some(tokio::spawn(stream_thread(stream_tx)));
                    decoder = Some(tokio::spawn(decoder_thread(decoder_tx, stream_rx, sem.clone())));
                    std::thread::spawn(move || playback_init(decoder_rx, playback_control_rx, sem));

                    if let Some(ref pctx) = playback_control_tx {
                        pctx.send(PlaybackControl::Volume(volume)).expect("Failed to send volume when creating pipeline");
                        pctx.send(PlaybackControl::Play).expect("Failed to send play when creating pipeline");
                    }
                },
                PlayerControl::Pause => {
                    if let Some(ref pctx) = playback_control_tx {
                        pctx.send(PlaybackControl::Stop).expect("Failed to stop playback");
                    }
                    playback_control_tx = None;
                    if let Some(ref stream) = stream {
                        stream.abort();
                    }
                    stream = None;
                    if let Some(ref decoder) = decoder {
                        decoder.abort();
                    }
                    decoder = None;
                }
            }
        }
    });

    player_tx
}
