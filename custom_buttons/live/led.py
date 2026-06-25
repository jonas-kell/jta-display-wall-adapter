from machine import Pin
from time import sleep

pin = Pin("LED", Pin.OUT)


def blink(num=1, blink=True):
    if blink:
        for i in range(num):
            pin.high()
            sleep(0.1)
            pin.low()
            sleep(0.1)