const { log } = require("console");
const http = require("http");

// Get port from command line args or default to 3000
const PORT = process.argv[2] || 3000;

const server = http.createServer((req, res) => {
  const url = req.url;

  if (url === "/health") {
    log(`Health check requested for ${PORT}`);
    res.writeHead(200, { "Content-Type": "application/json" });
    res.end(
      JSON.stringify({
        status: "healthy",
        port: PORT,
        timestamp: new Date().toISOString(),
      })
    );
  } else if (url === "/hello") {
    res.writeHead(200, { "Content-Type": "text/plain" });
    res.end(`Hello from server on port ${PORT}!\n`);
  } else {
    res.writeHead(404, { "Content-Type": "text/plain" });
    res.end("Not Found\n");
  }
});

server.listen(PORT, "127.0.0.1", () => {
  console.log(`Server running at http://127.0.0.1:${PORT}/`);
  console.log(`- Health check: http://127.0.0.1:${PORT}/health`);
  console.log(`- Hello endpoint: http://127.0.0.1:${PORT}/hello`);
});
