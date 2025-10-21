# Test Servers for Load Balanced Proxy

Simple Node.js test servers for testing the rs-ngnix load balancer.

## Features

- **Two servers** running on ports 3001 and 3002
- **Health endpoint**: `/health` - Returns JSON with server status and port
- **Hello endpoint**: `/hello` - Returns a greeting with the server port

## Usage

### Start both servers (PowerShell):
```powershell
cd test
.\start-servers.ps1
```

### Start servers manually:
```powershell
# Terminal 1
node server.js 3001

# Terminal 2
node server.js 3002
```

### Test the servers:
```powershell
# Test server 1
curl http://127.0.0.1:3001/health
curl http://127.0.0.1:3001/hello

# Test server 2
curl http://127.0.0.1:3002/health
curl http://127.0.0.1:3002/hello
```

## Update config.yaml

Add this to your `config.yaml` to proxy to these test servers:

```yaml
http:
  - listen: 8081
    proxy:
      - "127.0.0.1:3001"
      - "127.0.0.1:3002"
    strategy: "round_robin"  # or "random", "least_connections"
```

Then access through your proxy:
```powershell
curl http://localhost:8081/health
curl http://localhost:8081/hello
```

You should see responses from different backend servers!
