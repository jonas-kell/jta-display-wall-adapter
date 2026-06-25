import rp2
import utime as time
from environment import getEnvValue
import network

# Config Parameters

wlanSSID = getEnvValue("ssid")
wlanPW = getEnvValue("password")
rp2.country("DE")

# WLAN-Connection
def wlanConnect():

    print(f"Trying to connect to ssid: {wlanSSID}")
    print(f"Connecting with password: {wlanPW}")

    wlan = network.WLAN(network.STA_IF)
    if not wlan.isconnected():
        wlan.active(True)

        print("Connecting to WLAN-Networks")
        for ap in wlan.scan():
            print(ap)

        print("Connecting to WLAN-Network")

        wlan.connect(wlanSSID, wlanPW)
        for i in range(10):  # wait at most 10 seconds for wifi chip to connect
            if wlan.status() < 0 or wlan.status() >= 3:
                break
            print(".")
            time.sleep(1)
    if wlan.isconnected():
        print("Successfully connected to WLAN-Network")
        netConfig = wlan.ifconfig()
        print("IPv4-Address:", netConfig[0])
        print()
        return netConfig[0]
    else:
        print(
            "No WLAN-Network Connected. Have you set the correct configurational parameters in the .env?"
        )
        print("WLAN-Status:", wlan.status())
        print()
        return ""