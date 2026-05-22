# text-games-app

`text-games-app` is a playable web hub for ink-rs text games. It uses
`ink-dioxus` as the game engine and keeps game content in embedded `.ink`
source files under `assets/ink`.

This crate is an application, not a language test harness. Compiler, runtime,
and format bugs found while developing games should be recorded under
`docs/issues_found/` and handled by the relevant core-language work.

## Game Content

Each direct child directory under `assets/ink` is one playable game. The build
script discovers `.ink` files recursively inside each game directory and
generates the embedded source catalog consumed by the web binary.

```text
assets/ink/text-snake-10x10/story.ink
assets/ink/text-jrpg-vertical-slice/story.ink
assets/ink/text-jrpg-vertical-slice/battle.ink
```

## Local Play

Run the web app on all local interfaces so another device on the same network
can connect to the Mac's LAN IP address:

```bash
dx serve --package text-games-app --web --addr 0.0.0.0 --port 8080 --open false
```

Then open `http://<mac-lan-ip>:8080` from the phone.

For a deployment-like LAN preview, build the static web bundle and serve the
generated files on all local interfaces:

```bash
dx build --package text-games-app --web --release --debug-symbols false
python3 -m http.server 8080 --bind 0.0.0.0 --directory target/dx/text-games-app/release/web/public
```
