# Wan Player

A simple desktop player for the [gensokyoradio.net](https://gensokyoradio.net/) web radio with Discord Rich Presence
integration.

![Screenshot of Wan Player's interface](https://raw.githubusercontent.com/FlatBartender/WanPlayer/assets/wan_player_hZ6TbYViX8.png)

## Building

The code should be cross-platform. You'll need the Discord Game SDK as well as bindgen properly setup to build this 
project. All the instructions to do so are available from the
[discord_game_sdk](https://crates.io/crates/discord_game_sdk) crate documentation.

After the Discord Game SDK and `bindgen` are correctly setup, you should be able to build the project with a simple
`cargo build`.

# Redistribution

All the source code files in this project are licensed under the
[Apache 2.0](https://www.apache.org/licenses/LICENSE-2.0.txt) license, except the files in `src/resources` which may
have their own licenses (see the end of this file for more information).

I have included my own Wan Player Discord Application ID in the source code to make this easy to use and compile for
everyone. If you intend to redistribute or fork this project, please make sure to change this ID to use your own Discord
Application.

# Copyright notes

This project uses a number of dependencies. The direct dependencies and their versions are listed in the `Cargo.toml`
manifest file. All of these dependencies recursively pull different dependencies, all of which may have their own
licenses. I assumed that all those licenses were compatible with the Apache 2.0 license used for my own source code.

While I own the source code of this project, I do not own anything from the `src/resources` directory. All those files
are embedded directly in the final build:
- `pause.svg` and `play.svg` are modified versions of the `play_circle_filled` and `pause_circle_filled` icons from the
	[Material Design website](https://material.io/resources/icons/). These images are published by Google and licensed
	under the [Apache License Version 2.0](https://www.apache.org/licenses/LICENSE-2.0.txt). Modifications include
	changing the images' fill color and making the symbol solid and filled.
- `NotoSansSC-Regular.otf` if from the [Google Fonts website](https://fonts.google.com/), is designed by Google and
	licensed under the [Open Font License](https://scripts.sil.org/OFL).
- The Discord Rich Presence image was made by [@vibrantrida](https://twitter.com/vibrantrida) who kindly gave their
	approval for use in this project (even if it is not in the source code)
- `wan_player.ico` was made by [@LavenderTGreat](https://twitter.com/LavenderTGreat) who kindly gave their approval for
	use in this project as an executable, shortcut and window icon.
- `not_found.png` I made this one and honestly it was so little work you might as well use it for free for whatever
	project even commercial ones. This one is under
	[CCO](https://creativecommons.org/publicdomain/zero/1.0/legalcode-plain).

In addition, the `Gensokyo Radio` name, website and APIs used in this project are a property of LunarSpotlight Media.
The APIs in particular are used with permission from the website owner.

For inquiries regarding the licensing, please send me an email. This section of the document was made with my best
efforts with the intent to be as useful as possible but I am not a lawyer and may have missed some things.
