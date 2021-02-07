use serde::Deserialize;

pub struct ApiClient {
    client: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

pub const GR_API: &str = "https://gensokyoradio.net/json/";
pub const GR_STREAM: &str = "https://stream.gensokyoradio.net/1/";

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
        let res = self.client.get(hyper::Uri::from_static(GR_API))
            .await
            .expect("Failed to request song info");

        let data = hyper::body::to_bytes(res.into_body()).await.expect("Failed to collect song info request body");

        serde_json::from_slice(&data[..]).expect("Failed to parse song info")
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub circle: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
pub struct SongTimes {
    pub duration: String,
    pub played: u16,
    pub remaining: u16,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all="UPPERCASE")]
pub struct GRApiAnswer {
    pub songinfo: SongInfo,
    pub songtimes: SongTimes,
}


