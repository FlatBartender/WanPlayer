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

use discord_game_sdk::*;

use std::sync::mpsc::Receiver;

pub enum DiscordControl {
    SongInfo(super::gensokyo_radio::GRApiAnswer),
}

struct DiscordEventHandler;
impl EventHandler for DiscordEventHandler {}

const DISCORD_CLIENT_ID: i64 = 808130280976023563;

pub fn discord_main_loop(discord_rx: Receiver<DiscordControl>) {
    std::thread::spawn(move || {
        let mut discord: Option<Discord<'_, _>> = None;
        loop {
            let result = discord_rx.recv_timeout(std::time::Duration::from_secs(1));
            match result {
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    let result = match discord {
                        None => None,
                        Some(ref mut d) => d.run_callbacks().ok(),
                    };
                    // Ensure we don't keep the "bad" discord if it doesn't work anymore
                    if result.is_none() {
                        discord = None;
                    }
                },
                Err(_) => panic!(),
                Ok(DiscordControl::SongInfo(song_info)) => {
                    if discord.is_none() {
                        discord = new_discord_client(DISCORD_CLIENT_ID);
                    }

                    if let Some(ref mut discord) = discord {
                        let activity = activity_from_song_info(&song_info);
                        discord.update_activity(&activity, |_, result| {
                            if let Err(err) = result {
                                eprintln!("Error: {}", err);
                            }
                        });
                    }
                }
            }
        }

    });
}

fn new_discord_client(client_id: i64) -> Option<Discord<'static, DiscordEventHandler>> {
    let discord = Discord::with_create_flags(client_id, CreateFlags::NoRequireDiscord).ok();
    if discord.is_none() {
        return None;
    }

    let mut discord = discord.unwrap();
    discord.register_launch_command("https://gensokyoradio.net/music/playing/").expect("Failed to register launch command");
    *discord.event_handler_mut() = Some(DiscordEventHandler);

    Some(discord)
}

fn activity_from_song_info(song_info: &super::gensokyo_radio::GRApiAnswer) -> Activity {
    let mut activity = Activity::empty();
    activity
        .with_state("Listening to Gensokyo Radio")
        .with_details(&format!("{} - {}", &song_info.songinfo.artist, &song_info.songinfo.title))
        .with_start_time(song_info.songtimes.songstart as i64)
        .with_end_time(song_info.songtimes.songend as i64)
        .with_large_image_key("presence_image");

    activity
}
