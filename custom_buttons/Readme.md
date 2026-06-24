# Custom Button Conrtoller

[Need to flash with micropython firmware from here](https://micropython.org/download/RPI_PICO_W/)

## Wiring

TODO

## Manual testing

```micropython
from machine import Pin

blue = Pin(18, Pin.OUT)
green = Pin(19, Pin.OUT)
yellow = Pin(20, Pin.OUT)
white = Pin(21, Pin.OUT)
red = Pin(22, Pin.OUT)

blue.on()
green.on()
yellow.on()
white.on()
red.on()

blue_in = Pin(9, Pin.IN, Pin.PULL_UP)
green_in = Pin(10, Pin.IN, Pin.PULL_UP)
yellow_in = Pin(11, Pin.IN, Pin.PULL_UP)
white_in = Pin(12, Pin.IN, Pin.PULL_UP)
red_in = Pin(13, Pin.IN, Pin.PULL_UP)

blue_in.value()
green_in.value()
yellow_in.value()
white_in.value()
red_in.value()
```
