envValues = None

def parseEnvFile():
    env_data = {}

    try:
        with open("pi_config.env", "r") as file:
            for line in file.readlines():
                line = line.strip()  # Remove leading/trailing whitespace
                if "=" in line:
                    key, value = line.split("=", 1)
                    env_data[key] = value
    except:
        print(
            "Config couldn't be loaded. Is the config 'pi_config.example.env' renamed to 'pi_config.env' AND uploaded to the pi?"
        )
        return None

    return env_data


def getEnvValue(key):
    global envValues

    if envValues is None:
        envValues = parseEnvFile()

    try:
        value = envValues[key]
        return value
    except:
        print(f"The key '{key}' could not be loaded from the .env")
        return ""