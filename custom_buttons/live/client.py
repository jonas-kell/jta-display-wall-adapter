import socket
import json
import time

from environment import getEnvValue
from led import readButtonState, leds


STOP_BYTE = b"\x1E"


class TcpClient:
    def __init__(self, receivedCallback):
        self.targetIp = getEnvValue("targetip")
        self.targetPort = int(getEnvValue("targetport"))

        self.receivedCallback = receivedCallback

        self.sock = None
        self.buffer = b""

    def connect(self):
        while self.sock is None:
            try:
                print(f"Connecting to {self.targetIp}:{self.targetPort}...")

                s = socket.socket()
                s.connect((self.targetIp, self.targetPort))
                s.settimeout(0.1)  # Non-blocking-ish

                self.sock = s
                self.buffer = b""

                print("Connected.")
            except Exception as e:
                print("Connection failed:", e)
                try:
                    s.close()
                except:
                    pass
                self.sock = None
                time.sleep(2)

    def disconnect(self):
        if self.sock:
            try:
                self.sock.close()
            except:
                pass
        self.sock = None

    def send(self, message: dict):
        if self.sock is None:
            return False

        try:
            payload = json.dumps(message).encode("utf-8") + STOP_BYTE
            self.sock.sendall(payload)
            return True

        except Exception as e:
            print("Send failed:", e)
            self.disconnect()
            return False

    def poll(self):
        """
        Call this frequently from your main loop.
        """

        if self.sock is None:
            self.connect()
            return

        try:
            data = self.sock.recv(512)

            if len(data) == 0:
                raise OSError("Connection closed")

            self.buffer += data

            while STOP_BYTE in self.buffer:
                packet, self.buffer = self.buffer.split(STOP_BYTE, 1)

                if packet:
                    try:
                        obj = json.loads(packet.decode("utf-8"))
                        self.receivedCallback(obj)
                        print("Received:", obj)
                    except Exception as e:
                        print("Invalid JSON:", e)

        except OSError:
            # Timeout is expected
            pass

        except Exception as e:
            print("Socket lost:", e)
            self.disconnect()


def startTcpClient(ip):
    print(f"Network IP: {ip}")

    def receivedCallback(message):
        if "buttons_light_on" in message:
            print("Updating LED state")
            vals = message["buttons_light_on"]
            for index, is_on in enumerate(vals):
                if index < len(leds):
                    if bool(is_on):
                        leds[index].on()
                    else:
                        leds[index].off()

    client = TcpClient(receivedCallback)
    
    buttonState = readButtonState()

    while True:
        client.poll()

        newButtonState = readButtonState()
        if (buttonState != newButtonState):
            buttonState = newButtonState
            client.send({"buttons_pressed": newButtonState})
            print(newButtonState)
            
        time.sleep_ms(10)