from environment import getEnvValue


def startTcpClient(ip):
    targetIp = getEnvValue("targetip")
    targetPort = getEnvValue("targetport")

    print(f"Network IP: {ip}")
    print(f"Target IP:Port - {targetIp}:{targetPort}")
