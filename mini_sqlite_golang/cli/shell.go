package cli

import (
	"bufio"
	"fmt"
	"mini_sqlite/core"
	"os"
	"strings"
)

const prompt = "db> "

var exitCommands = map[string]bool{"quit": true, "exit": true, ":q": true}

func RunShell() {
	engine := core.NewDatabaseEngine()
	fmt.Println("Welcome to the mini SQL shell. Type 'exit' to quit.")

	scanner := bufio.NewScanner(os.Stdin)
	for {
		fmt.Print(prompt)
		if !scanner.Scan() {
			fmt.Println()
			break
		}

		query := strings.TrimSpace(scanner.Text())
		if query == "" {
			continue
		}
		if exitCommands[strings.ToLower(query)] {
			break
		}

		results := engine.Execute(query)
		for _, line := range results {
			fmt.Println(line)
		}
	}
}
