# ID cam stream docs

Here, we explain how to capture the ID cam video feed for further use, like displaying it on an additional display.

There are two main streams, one can utilize:

- **MJGPEG**: http://user:password@ip/axis-cgi/mjpg/video.cgi
- **RTSP**: rtsp://user:password@ip/axis-video/video.rtsp

The rtsp stream is much more performant, easily handling the max fps (30) with 5 or more clients without any performance loss.

The mjpg stream on the other hand can only deliver 30 fps on fullHD resolution and delivers roughly 14fps on 4k.

If you want to quickly try out capturing the streams, below is one short python scripts using the opencv-python pip package.

```python
import cv2
import time

user="username"
password="password"
ip="192.168.xxx.xxx"
# use this version for mjpg
url = f"http://{user}:{password}@{ip}/mjpg/video.mjpg"
# use this version for rtsp
url = f"rtsp://{user}:{password}@{ip}/axis-media/media.amp"

# Open the video stream
cap = cv2.VideoCapture(url)

if not cap.isOpened():
    print("Error: Could not open video stream.")
    exit()

# Variables to calculate FPS
frame_count = 0
start_time = time.time()

while True:
    ret, frame = cap.read()
    if not ret:
        print("Error: Could not read frame. Return code:", ret)
        break

    frame_count += 1

    # Calculate FPS every 10 frames
    if frame_count % 10 == 0:
        elapsed_time = time.time() - start_time
        fps = 10 / elapsed_time
        start_time = time.time()
        print(f"FPS: {fps:.2f}, shape: {frame.shape}")

    # Optional: Display the frame (for testing purposes)
    # Does not work if you use ssh or some sort of remote access
    # cv2.imshow('Video Stream', frame)

cap.release()
cv2.destroyAllWindows()
```
