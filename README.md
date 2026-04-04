# Display Wall Module by JTA

[![CC BY-NC-ND 4.0][cc-by-nc-nd-shield]][cc-by-nc-nd]

Software used for running custom display walls.

## Dev

Init diesel database and make schema changes

```cmd
# inital setup
docker compose -f docker-compose.diesel.yml run --rm diesel setup && docker compose -f docker-compose.diesel.yml down --remove-orphans
# create migration
docker compose -f docker-compose.diesel.yml run --rm diesel migration generate init && docker compose -f docker-compose.diesel.yml down --remove-orphans
# run migrations and print schema to file
docker compose -f docker-compose.diesel.yml run --rm diesel migration run && docker compose -f docker-compose.diesel.yml run --rm diesel migration redo && docker compose -f docker-compose.diesel.yml down --remove-orphans
# redo migrations while developing on a running instance
docker compose -f docker-compose.diesel.yml run --rm diesel migration run && docker compose -f docker-compose.diesel.yml run --rm diesel migration redo
```

Run dev mode

```cmd
echo -e "UID=$UID\nHOME=$HOME" > .env
docker compose up
```

## Build and run

```cmd
docker compose -f docker-compose.buildrun.yml up
```

## Build for Linux

```cmd
docker compose -f docker-compose.build.yml up --abort-on-container-exit && docker compose -f docker-compose.build.yml down --remove-orphans
```

Or legacy linux versions (older glibc).

```cmd
docker compose -f docker-compose.buildlegacy.yml up --abort-on-container-exit && docker compose -f docker-compose.buildlegacy.yml down --remove-orphans
```

## Cross compile Windows executable on Linux

You need to do [these steps](./PrepareForWindowsCompilation.md) once per machine.

```cmd
docker compose -f docker-compose.buildwindows.yml up --abort-on-container-exit && docker compose -f docker-compose.buildwindows.yml down --remove-orphans
```

## Build and push to docker hub

CAUTION: js must have been compiled beforehand -> run Build for linux, legacy or windows first!!!!

```cmd
docker buildx create --use
docker login
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t kellehorreur/jta-display-wall-adapter:latest \
  -f docker/run/Dockerfile \
  --push .
```

Re-tag if forgotten in last step (could also duplicate the tag line in the command above):

```cmd
docker buildx imagetools create -t kellehorreur/jta-display-wall-adapter:vX.X.X kellehorreur/jta-display-wall-adapter:latest
```

## Start on a Windows live system

- Copy over the executable from the previous compilation step (`/target/x86_64-pc-windows-gnu/release/jta-display-wall-adapter.exe`)
- Also copy the batch file `/start-jta-display-wall-adapter.bat` and the icon `/Link-Icon.ico`
- Put that together in a folder somewhere you desire
- Now right click on the .bat file "Send to Desktop (create a shortcut)"
- Edit the shortcut and prepend/edit the location with `C:\Windows\System32\cmd.exe /c "<...this is the content that was previously in the field...>"`
    - Now this is seen as an executable and you can pin it to the taskbar by dragging
    - Now for this shortcut you can set the icon
- For running the Wind server automatically
    - Do everything in [this readme](./usb_sniffer/Readme.md)
    - Get the correct Com-Port from the device manager
    - Modify the `start-jta-display-wall-adapter.bat` for use with wind-server mode (set Com-port and usb-iteration delay if you want) by setting mode to `wind`
    - Place a windows compiled version alongsite the .bat in a folder
    - Create a link to the .bat, by right-clicking -> Desktop-shortcut
    - Open the startup-folder by `Win+R`: `shell:startup`
    - Place the link inside it

## Install display client on a Raspberry Pi

See [this step-by-step tutorial](./RaspiDisplay.md).

## Generate Third Party license texts

Rust:

```cmd
cargo install cargo-license
cargo license -a > THIRD-PARTY-LICENSES-RUST
```

JS:

```cmd
npx generate-license-file
cd web_client
npx generate-license-file \
  --input package.json \
  --output ./../THIRD-PARTY-LICENSES-JS \
  --overwrite
```

## License

Copyright © 2025 `Just in Time Association` - [Website](https://just-in-time-association.de/)

This work is licensed under a
[Creative Commons Attribution-NonCommercial-NoDerivs 4.0 International License][cc-by-nc-nd].

[![CC BY-NC-ND 4.0][cc-by-nc-nd-image]][cc-by-nc-nd]

Additional Requirement:
Users must notify `Just in time Association` at [contact@just-in-time-association.de](mailto:contact@just-in-time-association.de) prior to any public use or distribution of this software.

[cc-by-nc-nd]: http://creativecommons.org/licenses/by-nc-nd/4.0/
[cc-by-nc-nd-image]: https://licensebuttons.net/l/by-nc-nd/4.0/88x31.png
[cc-by-nc-nd-shield]: https://img.shields.io/badge/License-CC%20BY--NC--ND%204.0-lightgrey.svg

### Third Party Licenses

This project includes third-party assets licensed under separate terms. See [Third Party Licenses](./THIRD_PARTY_LICENSES.md)
