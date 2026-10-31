# Systemd Setup

Note: The author is a monkey that doesn't know systemd and is unimpressed by the manual.
Take these notes with a *giant* grain of salt.

These examples makes a bunch of assumptions oriented around my local setup on my Steam Machine:
-   This repository has been cloned to `/home/deck/local/mmuhttpd`
-   The clone is built in release mode (e.g. `cargo build --release`)
-   The clone is owned by `deck`.
-   The clone is okay to run as `deck`.
-   ~~You've got a distrobox vm named `dev` that's setup to build rust.~~

## Lazy Socket Activation

```ini
# /etc/systemd/system/mmuhttpd.socket
[Unit]
Description=Example mmuhttpd socket

[Socket]
ListenStream=127.0.0.1:80

[Install]
WantedBy=sockets.target
```

```ini
# /etc/systemd/system/mmuhttpd.service
[Unit]
Description=Example mmuhttpd service
Requires=mmuhttpd.socket

[Service]
ExecStart=/home/deck/local/mmuhttpd/target/release/mmuhttpd --systemd /home/deck/local/mmuhttpd/demo
User=deck   # I can't even find the official documentation for this, but I've confirmed this causes mmuhttpd to run as `deck` instead of as `root`.
Group=deck  # Appears redundant.  Testing was done by patching `mmuhttpd` to run `id`
```

```sh
# Shell Commands

systemctl       status  mmuhttpd.socket     # inspect listening socket status
systemctl       status  mmuhttpd.service    # inspect underlying service process status

sudo systemctl  daemon-reload               # take notice of updated /etc/systemd/system/mmuhttpd.*

sudo systemctl  start   mmuhttpd.socket     # start listening immediately (does not start mmuhttpd.service until a connection is attempted)
sudo systemctl  stop    mmuhttpd.socket     # stop listening immediately (also stops mmuhttpd.service)
sudo systemctl  enable  mmuhttpd.socket     # enable  listening on reboot
sudo systemctl  disable mmuhttpd.socket     # disable listening on reboot
```
