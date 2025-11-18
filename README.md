# Display Wall Module by JTA

Software used for running custom display walls.

## Dev

```cmd
docker compose up
```

## Build and run

```cmd
docker compose -f docker-compose.buildrun.yml up
```

## Build and push to docker hub

```cmd
docker buildx create --use
docker login
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t kellehorreur/jta-display-wall-adapter:latest \
  -f docker/run/Dockerfile \
  --push .
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

```cmd
docker compose -f docker-compose.buildwindows.yml up --abort-on-container-exit && docker compose -f docker-compose.buildwindows.yml down --remove-orphans
```

## Start on a Windows live system

-   Copy over the executable from the previous compilation step (`/target/x86_64-pc-windows-gnu/release/jta-display-wall-adapter.exe`)
-   Also copy the batch file `/start-jta-display-wall-adapter.bat` and the icon `/Link-Icon.ico`
-   Put that together in a folder somewhere you desire
-   Now right click on the .bat file "Send to Desktop (create a shortcut)"
-   Edit the shortcut and prepend/edit the location with `C:\Windows\System32\cmd.exe /c "<...this is the content that was previously in the field...>"`
    -   Now this is seen as an executable and you can pin it to the taskbar by dragging
    -   Now for this shortcut you can set the icon

## Install display client on a Raspberry Pi

Generally, to run this you require an installation of [Docker](https://www.docker.com/) with [Docker compose](https://docs.docker.com/compose/).

In the case of this all being available, you only need to run the following command inside the main folder of this repository.

```cmd
sudo ./run.sh
```

### Fresh Installation on a Raspberry Pi 3B

Starting with a blank Raspberry Pi 3B and flashing the default Raspberry Pi OS `Trixie (64 bit)` image with desktop with the use of [Raspberry Pi Imager](https://www.raspberrypi.com/software/).
In this case, the user that is used in installation is called `wall` (as we use this for a local video wall).

Upgrade the installation and install docker:

```cmd
sudo apt-get update
sudo apt-get upgrade
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
sudo usermod -aG docker $USER
newgrp docker
sudo systemctl enable docker
sudo systemctl start docker
mkdir -p ~/.docker/cli-plugins/
ARCH=$(uname -m)
curl -SL https://github.com/docker/compose/releases/download/v2.40.3/docker-compose-linux-$ARCH -o ~/.docker/cli-plugins/docker-compose
chmod +x ~/.docker/cli-plugins/docker-compose
```

Set the session to Wayland (should be already) and make the user auto-login on boot:

```cmd
sudo raspi-config # check that is set to wayland -> screen blank can be also dissable there, but I do not think it does anything after switching to sway
sudo apt install sway foot
sudo nano /usr/share/wayland-sessions/sway.desktop # check if it is there and filled
```

```cmd
sudo nano /etc/lightdm/lightdm.conf
```

Modify the config file that opens.
At the Part `[Seat:*]` (probably it reads `rpd-labwc` -> keep that) replace with the following settings

```config
user-session=sway
autologin-session=sway
autologin.user=wall # the user of the pi is called wall
autologin-user-timeout=0
```

```cmd
mkdir ~/.config/sway
sudo nano ~/.config/sway/config
```

```config
# Set mod key (Super/Windows key)
set $mod Mod4

# Basic keybindings
bindsym $mod+Return exec foot

# Move focus
bindsym $mod+h focus left
bindsym $mod+j focus down
bindsym $mod+k focus up
bindsym $mod+l focus right
bindsym $mod+Left focus left
bindsym $mod+Down focus down
bindsym $mod+Up focus up
bindsym $mod+Right focus right

# Close window
bindsym $mod+Shift+q kill

# Bar (status bar) at the bottom
bar {
    position bottom
}

# hide cursor
seat * hide_cursor 1000

exec foot -e sh -c "cd /home/wall/Desktop/jta-display-wall-adapter && ./run.sh"
```

On Sway you can open a console with `Windows + Enter`.

```cmd
swaymsg reload

swaymsg -t get_tree
swaymsg "[title=\"JTA Display Window\"]" move position 100 200
```

Save, then the command `echo $XDG_SESSION_TYPE` should produce `wayland`.
And `echo $DESKTOP_SESSION` should yield `sway`.
On re-log you should also immediately boot into sway tiling window manager now.

### Setup the program to auto-start

Is already achieved with the last line of the sway config.
Fill the new file with the correct folder location (this assumes, that the repo was cloned to `home/wall/Desktop/jta-display-wall-adapter`).

### Setup a static IP on VLAN11 and DHCP on VLAN12:

The following command shows the name of the wired connection (use any other interface you want to target).
We assume, that the system uses `NetworkManager`.
Caution, you probably lose internet access after this, if your network can't handle the newly static assigned IP/Subnet.

```cmd
nmcli device status # find devices (later, name is already filled in as "eth0")

nmcli connection # find the name of the default connection (later, name is already filled in as "Wired connection 1")
```

Modify the connection (Here the IP is pre-filled with `192.168.150.150`, replace with your IP if needed):

```cmd

sudo nmcli connection add type vlan con-name vlan12 dev eth0 id 12 \
  ipv4.method auto \
  ipv6.method ignore \
  connection.autoconnect yes

sudo nmcli connection add type vlan con-name vlan11 dev eth0 id 11 \
  ipv4.addresses 192.168.150.150/24 \
  ipv4.method manual \
  ipv6.method ignore \
  connection.autoconnect yes

sudo nmcli connection del "Wired connection 1" # remove default connection -> might come back after restart, but not connect
sudo systemctl restart NetworkManager
```

Now running `ifconfig` whould show that the main network connection has a static IP on V11 and DHCP on V12.

This is a relativiely specific configuration, tailored to the needs to this setup. It is most likely not useful exactly for anyone else, as probably noone has the exact VLAN configuration at their raspberry pis ethernet port.

Make the networking delay on boot (as otherwise the switch is not yet ready...):

```cmd
sudo mkdir -p /etc/systemd/system/NetworkManager.service.d
sudo nano /etc/systemd/system/NetworkManager.service.d/delay.conf
```

Fill the newly opened config with the following;

```config
[Service]
ExecStartPre=/bin/sleep 30
```

And run the remaining commands to enable the changes and trigger a reboot to test everything:

```cmd
sudo systemctl daemon-reexec
sudo systemctl daemon-reload
sudo reboot
```
