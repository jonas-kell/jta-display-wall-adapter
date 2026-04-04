# Prepare for cross compilation to Windows

## Npcap

[Docs](https://npcap.com/#download)

```cmd
sudo apt install mingw-w64-tools
sudo apt install binutils-mingw-w64
mkdir generated_libs
cd generated_libs
curl https://npcap.com/dist/npcap-sdk-1.16.zip > lib.zip
curl https://npcap.com/dist/npcap-1.87.exe > lib.exe
unzip lib.zip
7z x lib.exe  # Always

gendef wpcap.dll
gendef Packet.dll
x86_64-w64-mingw32-dlltool -d wpcap.def -l libwpcap.a
x86_64-w64-mingw32-dlltool -d Packet.def -l libpacket.a
```
