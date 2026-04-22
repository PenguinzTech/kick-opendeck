# kick-opendeck

![Version: v0.1.0](https://img.shields.io/badge/version-v0.1.0-blue)

Kick stream control plugin for OpenDeck — manage your Kick live stream directly from your deck. Send chat messages, view live viewers, toggle slow mode, manage bans and mutes, and more.

## Features

Control 7 Kick actions from your OpenDeck device:

- **Kick Setup** — Configure Kick authentication. Press to manually refresh your auth token
- **Chat Message** — Send preset messages to your Kick chat
- **Viewer Count** — Display current live viewer count
- **Slow Mode** — Toggle slow mode in your chat
- **Ban User** — Ban a user from your channel
- **Unban User** — Unban a user from your channel
- **Mute User** — Mute a user in your channel

## Setup

### 1. Install OpenDeck

Follow the [OpenDeck installation guide](https://opendeck.io/wiki/software/) for your operating system.

### 2. Install the Plugin

```bash
# Clone or download this repository
git clone https://github.com/PenguinzTech/kick-opendeck.git
cd kick-opendeck

# Install the plugin
make install
```

This copies the plugin to:
- **macOS/Linux**: `~/.config/opendeck/plugins/dev.penguin.kick.sdPlugin/`
- **Windows**: `%APPDATA%\Elgato\StreamDeck\Plugins\dev.penguin.kick.sdPlugin\`

### 3. Add a Kick Action to Your Deck

1. Open OpenDeck
2. Add any Kick action to a button (Chat Message, Viewer Count, etc.)
3. The Property Inspector appears on the right

### 4. Authenticate with Kick

1. Add the **Kick Setup** action to a button
2. Click **Login with Kick** in the Property Inspector
3. A popup window displays an authorization URL
4. Visit the URL in your browser
5. Authorize the plugin to access your Kick account
6. Return to OpenDeck — authentication is complete
7. Press the **Kick Setup** button to manually refresh your auth token if needed

### Done

Your OpenDeck device is now authenticated with Kick. Actions will execute immediately when pressed.

## Actions Reference

| Action | Description |
|--------|-------------|
| Kick Setup | Configure Kick authentication. Press to manually refresh your auth token. |
| Chat Message | Send a preset message to your Kick chat |
| Viewer Count | Display current live viewer count |
| Slow Mode | Toggle slow mode in your chat |
| Ban User | Ban a user from your channel |
| Unban User | Unban a user from your channel |
| Mute User | Mute a user in your channel |

## Building from Source

### Prerequisites

- Rust 1.70 or later
- OpenDeck (installed)

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

### Build and Install

```bash
git clone https://github.com/PenguinzTech/kick-opendeck.git
cd kick-opendeck

# Build and install the plugin
make install

# Or build only
make build

# Or develop with live reload
make dev
```

## Supported Platforms

- **Windows** (x86_64)
- **macOS** (Intel x86_64, Apple Silicon ARM64)
- **Linux** (x86_64, ARM64)

## Important Notes

### Permissions & Restrictions

- **Moderation Actions** (Ban User, Unban User, Mute User, Slow Mode) require you to be:
  - The channel broadcaster, OR
  - A moderator in the channel

- **Chat Message** works with any authentication but requires the channel to be live

- **Viewer Count** works with any authentication but displays 0 if the channel is offline

### Token Management

Tokens are automatically:
- Persisted in OpenDeck's global settings (encrypted)
- Refreshed before expiration
- Validated on each button press

You can revoke access at any time from your Kick account settings.

## Configuration Per Action

Some actions support customization via the Property Inspector:

| Action | Configurable Fields |
|--------|-------------------|
| Chat Message | Preset message text |
| Slow Mode | Delay duration (configurable) |
| Ban User | Username to ban |
| Unban User | Username to unban |
| Mute User | Username to mute |

## Troubleshooting

**"Plugin not found"** — Run `make install` again to ensure the plugin directory structure is correct.

**"Login failed"** — Verify your Kick account credentials and that you have API access enabled.

**"Action failed: Not authorized"** — Ensure you are a moderator in the channel or the channel broadcaster.

**"Viewer count shows 0"** — The channel may be offline. Viewer count only displays for live streams.

## Development

### Project Structure

```
kick-opendeck/
├── src/
│   ├── main.rs              # Plugin entry point and message dispatcher
│   ├── auth.rs              # OAuth2 authentication implementation
│   ├── settings.rs          # OpenDeck property persistence
│   ├── kick_api.rs          # Kick API client
│   ├── global_handler.rs    # Global plugin state and initialization
│   └── actions/             # Individual action implementations
│       ├── chat_message.rs
│       ├── viewer_count.rs
│       ├── slow_mode.rs
│       └── ... (other actions)
├── Cargo.toml               # Rust dependencies
├── Makefile                 # Build and install targets
├── plugin/
│   └── manifest.json        # Plugin manifest and action definitions
└── README.md                # This file
```

### Running Tests

```bash
make test
```

## License

This plugin is open source. See LICENSE file for details.

## Support

For issues, feature requests, or contributions, visit the [GitHub Issues](https://github.com/PenguinzTech/kick-opendeck/issues) page.
