from time import sleep
from led import blink
from wlan import wlanConnect
from client import startTcpClient

BLINK = True  # SET TO True FOR DEBUG

# MAIN PROGRAM

blink(1, BLINK)

ip4 = wlanConnect()
sleep(0.1)

if BLINK:
    if ip4 != "":
        blink(2, BLINK)
    else:
        blink(1, BLINK)

startTcpClient(ip4)