# Test the packet capture with netcat

Sending

```cmd
echo "hello" | nc 127.0.0.1 9797
```

Receiving

```cmd
nc -l -k 9797
```

Wireshark filter

```txt
tcp && tcp.port == 9797
```
