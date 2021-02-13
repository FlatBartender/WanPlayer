// Copyright 2021 Flat Bartender <flat.bartender@gmail.com>
// 
//    Licensed under the Apache License, Version 2.0 (the "License");
//    you may not use this file except in compliance with the License.
//    You may obtain a copy of the License at
// 
//        http://www.apache.org/licenses/LICENSE-2.0
// 
//    Unless required by applicable law or agreed to in writing, software
//    distributed under the License is distributed on an "AS IS" BASIS,
//    WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
//    See the License for the specific language governing permissions and
//    limitations under the License.
//
// I do not own the Gensokyo Radio name or any of the services provided by the website.



use serde::Deserialize;

pub struct ApiClient {
    client: hyper::client::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

pub const GR_API: &str = "https://gensokyoradio.net/json/";
pub const GR_STREAM: &str = "https://stream.gensokyoradio.net/1/";
pub const GR_ALBUMART_ROOT: &str = "https://gensokyoradio.net/images/albums/200/";

const RETRY_SLEEP: u64 = 5;

impl ApiClient {
    pub fn new() -> ApiClient {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::client::Client::builder()
            .build::<_, hyper::Body>(https);
        ApiClient {
            client
        }
    }

    pub async fn get_song_info(&self) -> GRApiAnswer {
        let mut response = loop {
            let res = self.client.get(hyper::Uri::from_static(GR_API))
                .await
                .expect("Failed to request song info");

            let data = hyper::body::to_bytes(res.into_body()).await.expect("Failed to collect song info request body");

            match serde_json::from_slice::<GRApiAnswer>(&data[..]) {
                Ok(song_info) => break song_info,
                Err(error) => println!("{}", error),
            }
            tokio::time::sleep(std::time::Duration::from_secs(RETRY_SLEEP)).await;
        };
        response.songtimes.duration= response.songtimes.duration_str.parse().expect("Failed to parse duration");
        response
    }

    pub async fn get_album_image(&self, ans: &GRApiAnswer) -> Option<Vec<u8>> {
        let req_path = format!("{}{}", GR_ALBUMART_ROOT, ans.misc.albumart);
        let res = self.client.get(req_path.parse::<hyper::Uri>().expect("Failed to parse album art as uri"))
            .await
            .ok()?;

        let data = hyper::body::to_bytes(res.into_body()).await.ok()?;

        Some(data.to_vec())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="UPPERCASE")]
pub struct SongInfo {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub circle: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="UPPERCASE")]
pub struct SongTimes {
    #[serde(rename = "DURATION")]
    pub duration_str: String,
    #[serde(skip)]
    pub duration: u64,
    pub played: u64,
    pub remaining: u64,
    pub songstart: u64,
    pub songend: u64,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="UPPERCASE")]
pub struct Misc {
    circlelink: String,
    circleart: String,
    albumart: String,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all="UPPERCASE")]
pub struct GRApiAnswer {
    pub songinfo: SongInfo,
    pub songtimes: SongTimes,
    pub misc: Misc,
}
