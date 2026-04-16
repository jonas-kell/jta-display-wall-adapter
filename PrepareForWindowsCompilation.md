# Prepare for cross compilation to Windows

## Npcap

[Docs](https://npcap.com/#download)

```cmd
sudo apt install mingw-w64-tools
sudo apt install binutils-mingw-w64
mkdir generated_libs
cd generated_libs
curl https://npcap.com/dist/npcap-1.87.exe > lib.exe
7z x lib.exe  # Always

gendef wpcap_x64.dll
mv wpcap_x64.def wpcap.def
x86_64-w64-mingw32-dlltool -d wpcap.def -l libwpcap.a

gendef Packet_x64.dll
mv Packet_x64.def Packet.def
x86_64-w64-mingw32-dlltool -d Packet.def -l libpacket.a
```

<!-- curl https://npcap.com/dist/npcap-sdk-1.16.zip > lib.zip # no longer needed -->
<!-- unzip lib.zip # no longer needed -->

<!-- commented here https://github.com/rust-pcap/pcap/issues/246 that this works -->
