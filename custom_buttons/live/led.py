from machine import Pin
from time import sleep

# Onboard
bled = Pin("LED", Pin.OUT)

# LEDs
blue = Pin(18, Pin.OUT)
green = Pin(19, Pin.OUT)
yellow = Pin(20, Pin.OUT)
white = Pin(21, Pin.OUT)
red = Pin(22, Pin.OUT)

leds = [blue, green, yellow, white, red]

bled.off()
blue.off()
green.off()
yellow.off()
white.off()
red.off()

def blink(num=1, blink=True):
    if blink:
        for i in range(num):
            bled.on()
            blue.on()
            sleep(0.1)
            bled.off()
            blue.off()
            sleep(0.1)

# Buttons (active low)
blue_in = Pin(9, Pin.IN, Pin.PULL_UP)
green_in = Pin(10, Pin.IN, Pin.PULL_UP)
yellow_in = Pin(11, Pin.IN, Pin.PULL_UP)
white_in = Pin(12, Pin.IN, Pin.PULL_UP)
red_in = Pin(13, Pin.IN, Pin.PULL_UP)

def readButton(button):
    def capsule():
        return button.value() == 0
    
    return capsule

blue_button = readButton(blue_in)
green_button = readButton(green_in)
yellow_button = readButton(yellow_in)
white_button = readButton(white_in)
red_button = readButton(red_in)

buttons = [
    blue_button,
    green_button,
    yellow_button,
    white_button,
    red_button
]

def readButtonState():
    return [btn() for btn in buttons]