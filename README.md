# Daymark

Daymark is a local-first macOS productivity tracker built with Tauri 2, React, TypeScript, Rust, and SQLite. It records foreground app intervals and optional Brave website intervals without page content, keystrokes, passwords, or an account.

## macOS development setup

Requirements: macOS 12+, Node 20+, Rust stable, Xcode Command Line Tools, and Brave.

On current Homebrew, `rustup-init` is no longer provided. Use the keg-only Rust proxies directly:

```bash
brew install rustup
echo 'export PATH="/opt/homebrew/opt/rustup/bin:$HOME/.cargo/bin:$PATH"' >> ~/.zshrc
source ~/.zshrc
rustup toolchain install stable --profile minimal
rustup default stable
```

Verify with `cargo --version` and `rustc --version` before running the Tauri commands below.

```bash
npm install
npm run tauri dev
```

On the first run, macOS will require Automation / Accessibility permission for foreground application observation. Grant access to Daymark in System Settings → Privacy & Security → Accessibility, then restart Daymark. Activity is stored in `~/Library/Application Support/com.daymark.productivity/daymark.sqlite`.

### Brave extension

1. Start Daymark once with `npm run tauri dev`.
2. In Brave, open `brave://extensions`, enable **Developer mode**, choose **Load unpacked**, and select this repository’s `extension/` directory.
3. Copy the extension ID shown on the extension card.
4. From the repository root, install the Native Messaging host:

   ```bash
   ./native-host/install.sh YOUR_EXTENSION_ID
   ```

   This writes the exact host manifest to `~/Library/Application Support/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.daymark.productivity.json` and points it at the repository’s absolute `host.py` path.

5. Reload the extension in `brave://extensions`, then open a normal (non-incognito) tab. The extension is intentionally disabled in Incognito.

The extension uses Native Messaging and only sends domain, URL, title, and timestamps. Incognito is disabled by default. Browser intervals are a separate source and are not added on top of the browser application's own interval in the aggregation layer.

## Development notes

- `src-tauri/src/core.rs` contains deterministic interval clipping, merging, and idle tests.
- `src-tauri/src/db.rs` owns the SQLite schema and WAL configuration.
- `src-tauri/src/tracker.rs` is the macOS focused-app adapter; sleep/lock events should close the active interval before the next event.
- `extension/background.js` handles tab switching, navigation, window focus, browser idle, and shutdown.

Run the checks with:

```bash
npm run build
npm test
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
cargo test --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

Package a signed app with `npm run tauri build` after adding the desired signing identity.

## Privacy controls

Pause tracking from the dashboard and classify apps/sites as productive / neutral / distracting. No classification is inferred: new apps and websites are neutral until you choose a category.

## Manual end-to-end checklist

- [ ] Launch Daymark and grant Accessibility / Automation permission; confirm the focused app appears after about 5 seconds.
- [ ] Switch between two applications; confirm the first interval closes and the second starts.
- [ ] Leave the Mac idle for five minutes or lock the screen; confirm no idle time is added.
- [ ] Pause tracking, switch applications, resume, and confirm the paused interval is absent.
- [ ] In Brave, load the unpacked extension, install the Native Messaging host, and confirm no “Native host has exited” error appears.
- [ ] Switch tabs, navigate within a tab, focus another window, and close Brave; confirm website intervals end at each transition.
- [ ] Expand Brave in the dashboard; confirm website rows come from SQLite rather than placeholder data.
- [ ] Compare Brave application time with its website rows; confirm website time is a breakdown and is not added to total tracked time.
- [ ] Change an app and a domain between productive, neutral, and distracting; reload Daymark and confirm classifications persist.
- [ ] Drag the pet, switch applications, and confirm it stays above other windows without taking focus.
- [ ] Use the tray menu to open Daymark, pause/resume tracking, and quit.
- [ ] Verify only app/domain metadata and timestamps are stored; no page content, keystrokes, passwords, or form data.
