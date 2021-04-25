# homebridge-rusty-spotify

[Spotify](https://www.spotify.com) plugin for [Homebridge](https://github.com/homebridge/homebridge) written in Rust.
The plugin requires a Spotify Premium account.

## Installation and Setup

1. Install [Homebridge](https://github.com/homebridge/homebridge): `sudo npm install -g homebridge`
1. Install the plugin: `sudo npm install -g homebridge-rusty-spotify`
1. Register the plugin as app in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard/login)
    1. Login
    1. Select "Create a client ID"
    1. Provide a name and description in the pop-up; click "Next"
    1. Copy the "Client ID" and "Client Secret" which will be required in the following configuration step
    1. Click "Edit Settings"
    1. Add `http://localhost/callback` as "Redirect URI" and save
1. Configure the plugin using the generated client ID and client secret (see Configuration)

### Configuration

The `generate_config` script can be used to generate the config. It requires for the `client_id`, `client_secret` and
Spotify `username` needs to be set since those are required to authenticate to the Spotify Web API. To run the script,
make sure to have [Python 3](https://www.python.org/download/releases/3.0/) and the [spotipy](https://pypi.org/project/spotipy/)
library installed.

Running the script will open a web browser asking to authenticate to Spotify which is required to retrieve the `refresh_token`.

```bash
$ ./generate_config --help
usage: generate_config [-h] [--client_id CLIENT_ID]
                       [--client_secret CLIENT_SECRET]
                       [--redirect_uri REDIRECT_URI] [--username USERNAME]

Script to retrieve an access and refresh token for using the Spotify API

optional arguments:
  -h, --help            show this help message and exit
  --client_id CLIENT_ID, --client-id CLIENT_ID
                        Spotify client ID
  --client_secret CLIENT_SECRET, --client-secret CLIENT_SECRET
                        Spotify client secret
  --redirect_uri REDIRECT_URI, --redirect-uri REDIRECT_URI
                        Redirect URI
  --username USERNAME   Spotify username


$ ./generate_config --client_id=<client_id> --client_secret=<client_secret> --username=<username>
  {
    "platform": "Spotify",
    "name": "Spotify",
    "service_type": "light",    // "light" or "speaker"; Speaker is not supported by HomeKit
    "client_id": "<client_id>",
    "client_secret": "<client_secret>",
    "refresh_token": "<refresh_token>"
  }
```

The generated config needs to copied to the Homebridge config file (e.g. `~/.homebridge/config.json`). For example:

```json
//...
"platforms": [
  {
    "platform": "Spotify",
    "name": "Spotify",
    "service_type": "light",    // "light" or "speaker"; Speaker is not supported by HomeKit
    "client_id": "<client_id>",
    "client_secret": "<client_secret>",
    "refresh_token": "<refresh_token>",
  }
]
//...
```

`service_type` specifies whether Spotify devices should use the [Lightbulb](https://developers.homebridge.io/#/service/Lightbulb)
or [Speaker](https://developers.homebridge.io/#/service/Speaker) service. If `service_type` is not specified, `"light"` will be used by default.
HomeKit currently does not support Speaker services and will show _"This accessory is not certified and may not work reliably with HomeKit"_. 

## Usage

Add the plugin in the Home app. The plugin will automatically discover available Spotify 
devices and add them as accessories.
Turning a Spotify accessory on will resume playing music on the device, turning off the
accessory will pause the music. The accessory also allows to change the playback volume.

Accessories get refreshed every 10 seconds (or as specified in the configuration file).

## Development

1. Install the Rust toolchain, `wasm-pack`, `cargo-generate` and `npm` by [following this guide](https://rustwasm.github.io/book/game-of-life/setup.html)
1. Clone the repository
1. Run `make`
    * This will create a `pkg/` directory containing all the generated nodejs files
1. Copy the generated files to a device/directory that can be discovered by Homebridge
1. Switch to the directory and run `npm install` to install all required dependencies
1. Run Homebridge in debug mode and specify the directory with the plugin files: `DEBUG=* homebridge -D  -P /path/to/plugin/homebridge-rusty-spotify`

A blog post about writing plugins for Homebridge and specifically this plugin has been published [here](https://scholtzan.net/blog/homebridge-rusty-spotify/). 
