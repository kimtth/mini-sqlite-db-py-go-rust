package main

import (
	"flag"
	"mini_sqlite/cli"
	"mini_sqlite/web"
)

func main() {
	webMode := flag.Bool("web", false, "Launch web UI instead of interactive shell")
	host := flag.String("host", "127.0.0.1", "Host for web UI")
	port := flag.Int("port", 8000, "Port for web UI")
	flag.Parse()

	if *webMode {
		web.RunServer(*host, *port)
	} else {
		cli.RunShell()
	}
}
