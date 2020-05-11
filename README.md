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
Spotify `username` to be set since those are required to authenticate to the Spotify Web API. Optionally, a device ID can
be specified of the device that should be controlled by the plugin. If no device is specified, the device used last
will be controlled.

Running the script will open a web browser asking to authenticate to Spotify which is required to retrieve the `refresh_token`.

```bash
$ ./generate_config --help
usage: generate_config [-h] [--client_id CLIENT_ID]
                       [--client_secret CLIENT_SECRET]
                       [--redirect_uri REDIRECT_URI] [--username USERNAME]
                       [--device_id [DEVICE_ID]]

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
  --device_id [DEVICE_ID], --device-id [DEVICE_ID]
                        Device ID


$ ./generate_config --client_id=<client_id> --client_secret=<client_secret> --username=<username> --device_id=<device_id>
  {
    "accessory": "Spotify",
    "name": "Spotify",
    "client_id": "<client_id>",
    "client_secret": "<client_secret>",
    "refresh_token": "<refresh_token>",
    "device_id": "<device_id>"
  }
```

The generated config needs to copied to the Homebridge config file (e.g. `~/.homebridge/config.json`). For example:

```json
//...
"accessories": [
  {
    "accessory": "Spotify",
    "name": "Spotify",
    "client_id": "<client_id>",
    "client_secret": "<client_secret>",
    "refresh_token": "<refresh_token>",
    "device_id": "<device_id>"
  }
]
//...
```

## Usage

Add the accessory in the Home app. Turning the Spotify accessory on will resume playing music on Spotify, turning off the
accessory will pause the music. The accessory also allows to change the playback volume.

## Development

1. Install the Rust toolchain, `wasm-pack`, `cargo-generate` and `npm` by [following this guide](https://rustwasm.github.io/book/game-of-life/setup.html)
1. Clone the repository
1. Run `make`
    * This will create a `pkg/` directory containing all the generated nodejs files
1. Copy the generated files to a device/directory that can be discovered by Homebridge
1. Switch to the directory and run `npm install` to install all required dependencies
1. Run Homebridge in debug mode and specify the directory with the plugin files: `DEBUG=* homebridge -D  -P /path/to/plugin/homebridge-rusty-spotify`
