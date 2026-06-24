from machine import Pin
import time

# LEDs
blue = Pin(18, Pin.OUT)
green = Pin(19, Pin.OUT)
yellow = Pin(20, Pin.OUT)
white = Pin(21, Pin.OUT)
red = Pin(22, Pin.OUT)

# Buttons (active low)
blue_in = Pin(9, Pin.IN, Pin.PULL_UP)
green_in = Pin(10, Pin.IN, Pin.PULL_UP)
yellow_in = Pin(11, Pin.IN, Pin.PULL_UP)
white_in = Pin(12, Pin.IN, Pin.PULL_UP)
red_in = Pin(13, Pin.IN, Pin.PULL_UP)

leds = [blue, green, yellow, white, red]
buttons = [blue_in, green_in, yellow_in, white_in, red_in]

# Turn everything off initially
for led in leds:
    led.off()

# Track LED states
states = [False] * 5

# Track previous button state for edge detection
previous = [1] * 5

print("Button/LED test ready")

for led in leds:
    led.on()
time.sleep(0.2)
for led in leds:
    led.off()
time.sleep(0.2)
for led in leds:
    led.on()
time.sleep(0.2)
for led in leds:
    led.off()
time.sleep(0.05)

print("Button/LED init sequence sent")

while True:
    for i, button in enumerate(buttons):
        current = button.value()

        # Detect button press (HIGH -> LOW)
        if previous[i] == 1 and current == 0:
            states[i] = not states[i]

            if states[i]:
                leds[i].on()
            else:
                leds[i].off()

            print(f"Button {i+1}: {'ON' if states[i] else 'OFF'}")

            # Simple debounce
            time.sleep_ms(50)

        previous[i] = current

    # Success condition: all LEDs on
    if all(states):
        print("All buttons tested!")

        # Turn off all LEDs
        for led in leds:
            led.off()

        time.sleep(0.2)

        # Flash once
        for led in leds:
            led.on()

        time.sleep(0.2)

        for led in leds:
            led.off()

        # Reset state tracking
        states = [False] * 5
        time.sleep(0.5)

    time.sleep_ms(10)
