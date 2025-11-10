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

## Cross compile Windows executable on Linux

```cmd
rustup target add x86_64-pc-windows-gnu
sudo apt install mingw-w64
cargo build --release --target x86_64-pc-windows-gnu
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
Furthermore, the window system should be Wayland.

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
sudo nano /etc/lightdm/lightdm.conf
```

Modify the config file that opens.
At the Part `[Seat:*]` (probably it reads `rpd-labwc` -> keep that) replace with the following settings

```config
user-session=rpd-labwc
autologin-session=rpd-labwc
autologin.user=wall # the user of the pi is called wall
autologin-user-timeout=0
```

Save, then the command `echo $XDG_SESSION_TYPE` should produce `wayland`.
And `echo $DESKTOP_SESSION` should yield `rpd-labwc`.
On re-log you should also immediately boot into desktop now.

We use XWayland though, because wayland compositor does not allow us to reposition windows...

```cmd
Xwayland -version
ps aux | grep Xwayland # should tell something like "-auth /home/wall/.Xauthority :0 -rootless"
sudo apt-get install -y x11-utils x11-apps
```

Setup the program to auto-start:

```cmd
sudo apt-get install dex
mkdir -p ~/.config/autostart
nano ~/.config/autostart/jta-adapter.desktop
```

Fill the new file with the following configuration (this assumes, that the repo was cloned to `home/wall/Desktop/jta-display-wall-adapter`).

```config
[Desktop Entry]
Type=Application
Name=JTA Wall Adapter
Exec=lxterminal -e bash -c "cd /home/wall/Desktop/jta-display-wall-adapter/ && ./run.sh"
Path=/home/wall/Desktop/jta-display-wall-adapter/
X-GNOME-Autostart-enabled=true
```

Save and run the following commands to activate

```cmd
chmod +x ~/.config/autostart/jta-adapter.desktop
dex ~/.config/autostart/jta-adapter.desktop
```

Setup a static IP on VLAN11 and DHCP on VLAN12:

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
