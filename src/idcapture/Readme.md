# Test the packet capture with netcat

Sending

```cmd
echo "hello" | nc 127.0.0.1 8758

apt-get update
apt-get install netcat-openbsd
echo "hello" | nc 127.0.0.1 8758
```

Receiving

```cmd
nc -l -k 8758
```

Wireshark filter

```txt
tcp && tcp.port == 8758
```
