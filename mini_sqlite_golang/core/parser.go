package core

import (
	"regexp"
	"strconv"
	"strings"
)

type ParsedCommand struct {
	Command string
	Raw     string
	Details map[string]interface{}
}

type SQLParser struct {
	selectRe *regexp.Regexp
}

func NewSQLParser() *SQLParser {
	return &SQLParser{
		selectRe: regexp.MustCompile(`(?i)SELECT\s+(?P<cols>.+?)\s+FROM\s+(?P<table>\w+)(?:\s+INNER\s+JOIN\s+(?P<join_table>\w+)\s+ON\s+(?P<left_table>\w+)\.(?P<left_col>\w+)\s*=\s*(?P<right_table>\w+)\.(?P<right_col>\w+))?(?:\s+WHERE\s+(?P<where_col>\w+)\s*=\s*(?P<where_val>.+))?$`),
	}
}

func (p *SQLParser) Parse(query string) ParsedCommand {
	raw := strings.TrimSpace(query)
	if raw == "" {
		return ParsedCommand{Command: "EMPTY", Raw: "", Details: map[string]interface{}{}}
	}

	text := strings.TrimSuffix(raw, ";")
	tokens := strings.Fields(text)
	command := strings.ToUpper(tokens[0])

	parsers := map[string]func(string, []string) (string, map[string]interface{}){
		"CREATE DATABASE": func(t string, tok []string) (string, map[string]interface{}) {
			return "CREATE_DATABASE", map[string]interface{}{"name": strings.ToLower(tok[2])}
		},
		"ALTER DATABASE": func(t string, tok []string) (string, map[string]interface{}) {
			return "ALTER_DATABASE", map[string]interface{}{"name": strings.ToLower(tok[2])}
		},
		"CREATE TABLE": func(t string, tok []string) (string, map[string]interface{}) {
			return "CREATE_TABLE", p.parseCreateTable(t)
		},
		"ALTER TABLE": func(t string, tok []string) (string, map[string]interface{}) {
			return "ALTER_TABLE", p.parseAlterTable(t, tok)
		},
		"DROP TABLE": func(t string, tok []string) (string, map[string]interface{}) {
			return "DROP_TABLE", map[string]interface{}{"table": strings.ToLower(tok[2])}
		},
		"CREATE INDEX": func(t string, tok []string) (string, map[string]interface{}) {
			return "CREATE_INDEX", map[string]interface{}{"table": strings.ToLower(tok[2]), "column": strings.ToLower(tok[3])}
		},
		"DROP INDEX": func(t string, tok []string) (string, map[string]interface{}) {
			return "DROP_INDEX", map[string]interface{}{"table": strings.ToLower(tok[2]), "column": strings.ToLower(tok[3])}
		},
	}

	var details map[string]interface{}

	if command == "COMMIT" {
		details = map[string]interface{}{}
	} else if command == "INSERT" {
		command, details = "INSERT", p.parseInsert(text)
	} else if command == "UPDATE" {
		command, details = "UPDATE", p.parseUpdate(text)
	} else if command == "DELETE" {
		command, details = "DELETE", p.parseDelete(text)
	} else if command == "SELECT" {
		command, details = "SELECT", p.parseSelect(text)
	} else if command == "USE" && len(tokens) > 1 {
		command, details = "USE_DATABASE", map[string]interface{}{"name": strings.ToLower(tokens[1])}
	} else if len(tokens) > 1 {
		key := command + " " + strings.ToUpper(tokens[1])
		if parser, ok := parsers[key]; ok {
			command, details = parser(text, tokens)
		} else {
			command, details = "UNKNOWN", map[string]interface{}{}
		}
	} else {
		command, details = "UNKNOWN", map[string]interface{}{}
	}

	return ParsedCommand{Command: command, Raw: raw, Details: details}
}

func (p *SQLParser) parseCreateTable(text string) map[string]interface{} {
	idx := strings.Index(text, "(")
	if idx == -1 {
		return map[string]interface{}{}
	}
	header := text[:idx]
	tokens := strings.Fields(header)
	if len(tokens) < 3 {
		return map[string]interface{}{}
	}
	table := strings.ToLower(tokens[2])

	endIdx := strings.LastIndex(text, ")")
	if endIdx == -1 {
		return map[string]interface{}{}
	}
	inner := text[idx+1 : endIdx]

	var columns []interface{}
	for _, chunk := range strings.Split(inner, ",") {
		parts := strings.Fields(strings.TrimSpace(chunk))
		if len(parts) == 0 {
			continue
		}
		name := strings.ToLower(parts[0])
		colType := "TEXT"
		if len(parts) > 1 {
			colType = strings.ToUpper(parts[1])
		}
		columns = append(columns, map[string]string{"name": name, "type": colType})
	}

	return map[string]interface{}{"table": table, "columns": columns}
}

func (p *SQLParser) parseAlterTable(text string, tokens []string) map[string]interface{} {
	if len(tokens) >= 7 && strings.ToUpper(tokens[3]) == "ADD" && strings.ToUpper(tokens[4]) == "COLUMN" {
		table := strings.ToLower(tokens[2])
		name := strings.ToLower(tokens[5])
		colType := "TEXT"
		if len(tokens) > 6 {
			colType = strings.ToUpper(tokens[6])
		}
		return map[string]interface{}{
			"table":  table,
			"column": map[string]string{"name": name, "type": colType},
		}
	}
	return map[string]interface{}{}
}

func (p *SQLParser) parseInsert(text string) map[string]interface{} {
	re := regexp.MustCompile(`(?i)INSERT\s+INTO\s+(\w+)\s+VALUES\s*\((.+)\)$`)
	matches := re.FindStringSubmatch(text)
	if matches == nil {
		return map[string]interface{}{}
	}
	table := strings.ToLower(matches[1])
	values := p.parseValueList(matches[2])
	return map[string]interface{}{"table": table, "values": values}
}

func (p *SQLParser) parseUpdate(text string) map[string]interface{} {
	upper := strings.ToUpper(text)
	var prefix, wherePart string
	if idx := strings.Index(upper, " WHERE "); idx != -1 {
		prefix = text[:idx]
		wherePart = text[idx+7:]
	} else {
		prefix = text
	}

	tokens := strings.Fields(prefix)
	if len(tokens) < 2 {
		return map[string]interface{}{}
	}
	table := strings.ToLower(tokens[1])

	setIdx := strings.Index(strings.ToUpper(prefix), " SET ")
	if setIdx == -1 {
		return map[string]interface{}{}
	}
	setClause := prefix[setIdx+5:]

	assignments := make(map[string]interface{})
	for _, chunk := range strings.Split(setClause, ",") {
		parts := strings.SplitN(chunk, "=", 2)
		if len(parts) == 2 {
			column := strings.ToLower(strings.TrimSpace(parts[0]))
			value := p.parseLiteral(strings.TrimSpace(parts[1]))
			assignments[column] = value
		}
	}

	result := map[string]interface{}{"table": table, "assignments": assignments}
	if wherePart != "" {
		result["where"] = p.parseCondition(wherePart)
	}
	return result
}

func (p *SQLParser) parseDelete(text string) map[string]interface{} {
	upper := strings.ToUpper(text)
	var prefix, wherePart string
	if idx := strings.Index(upper, " WHERE "); idx != -1 {
		prefix = text[:idx]
		wherePart = text[idx+7:]
	} else {
		prefix = text
	}

	tokens := strings.Fields(prefix)
	if len(tokens) < 3 {
		return map[string]interface{}{}
	}
	table := strings.ToLower(tokens[2])

	result := map[string]interface{}{"table": table}
	if wherePart != "" {
		result["where"] = p.parseCondition(wherePart)
	}
	return result
}

func (p *SQLParser) parseSelect(text string) map[string]interface{} {
	matches := p.selectRe.FindStringSubmatch(text)
	if matches == nil {
		return map[string]interface{}{}
	}

	names := p.selectRe.SubexpNames()
	result := make(map[string]string)
	for i, name := range names {
		if i != 0 && name != "" && i < len(matches) {
			result[name] = matches[i]
		}
	}

	cols := strings.Split(result["cols"], ",")
	var columns []interface{}
	for _, col := range cols {
		columns = append(columns, strings.TrimSpace(col))
	}
	table := strings.ToLower(result["table"])

	details := map[string]interface{}{"table": table, "columns": columns}

	if result["where_col"] != "" {
		details["where"] = map[string]interface{}{
			"column": strings.ToLower(result["where_col"]),
			"value":  p.parseLiteral(result["where_val"]),
		}
	}

	if result["join_table"] != "" {
		details["join"] = map[string]interface{}{
			"table":        strings.ToLower(result["join_table"]),
			"left_table":   strings.ToLower(result["left_table"]),
			"left_column":  strings.ToLower(result["left_col"]),
			"right_table":  strings.ToLower(result["right_table"]),
			"right_column": strings.ToLower(result["right_col"]),
		}
	}

	return details
}

func (p *SQLParser) parseValueList(segment string) []interface{} {
	var values []interface{}
	inString := false
	current := ""

	for _, ch := range segment {
		switch ch {
		case '\'':
			if inString {
				values = append(values, current)
				current = ""
				inString = false
			} else {
				inString = true
			}
		case ',':
			if !inString && strings.TrimSpace(current) != "" {
				values = append(values, p.parseLiteral(strings.TrimSpace(current)))
				current = ""
			} else if inString {
				current += string(ch)
			}
		default:
			current += string(ch)
		}
	}

	if strings.TrimSpace(current) != "" {
		if inString {
			values = append(values, current)
		} else {
			values = append(values, p.parseLiteral(strings.TrimSpace(current)))
		}
	}

	return values
}

func (p *SQLParser) parseCondition(clause string) map[string]interface{} {
	if clause == "" {
		return map[string]interface{}{}
	}
	parts := strings.SplitN(clause, "=", 2)
	if len(parts) != 2 {
		return map[string]interface{}{}
	}
	return map[string]interface{}{
		"column": strings.ToLower(strings.TrimSpace(parts[0])),
		"value":  p.parseLiteral(strings.TrimSpace(parts[1])),
	}
}

func (p *SQLParser) parseLiteral(text string) interface{} {
	text = strings.TrimSpace(text)
	if (strings.HasPrefix(text, "'") && strings.HasSuffix(text, "'")) ||
		(strings.HasPrefix(text, "\"") && strings.HasSuffix(text, "\"")) {
		return text[1 : len(text)-1]
	}
	if i, err := strconv.ParseInt(text, 10, 64); err == nil {
		return i
	}
	if f, err := strconv.ParseFloat(text, 64); err == nil {
		return f
	}
	return text
}
