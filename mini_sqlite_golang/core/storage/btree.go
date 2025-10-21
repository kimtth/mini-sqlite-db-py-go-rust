package storage

import (
	"encoding/json"
	"fmt"
)

type Row map[string]interface{}
type Index map[string][]Row

type TableMeta struct {
	Columns []string
	Rows    []Row
	Indexes map[string]Index
}

type tableDump struct {
	Columns []string `json:"columns"`
	Rows    []Row    `json:"rows"`
	Indexes []string `json:"indexes"`
}

type BTreeStorage struct {
	pager  *Pager
	tables map[string]*TableMeta
}

func NewBTreeStorage(pager *Pager) *BTreeStorage {
	b := &BTreeStorage{
		pager:  pager,
		tables: make(map[string]*TableMeta),
	}
	b.load()
	return b
}

func (b *BTreeStorage) CreateTable(name string, columns []string) {
	b.tables[name] = &TableMeta{
		Columns: columns,
		Rows:    make([]Row, 0),
		Indexes: make(map[string]Index),
	}
	b.persist()
}

func (b *BTreeStorage) DropTable(name string) {
	delete(b.tables, name)
	b.persist()
}

func (b *BTreeStorage) TableExists(name string) bool {
	_, ok := b.tables[name]
	return ok
}

func (b *BTreeStorage) AddColumn(name, column string) {
	table := b.tables[name]
	for _, col := range table.Columns {
		if col == column {
			return
		}
	}
	table.Columns = append(table.Columns, column)
	for i := range table.Rows {
		table.Rows[i][column] = nil
	}
	for col := range table.Indexes {
		b.rebuildIndex(name, col)
	}
	b.persist()
}

func (b *BTreeStorage) CreateIndex(name, column string) {
	table := b.tables[name]
	if _, ok := table.Indexes[column]; !ok {
		b.rebuildIndex(name, column)
	}
	b.persist()
}

func (b *BTreeStorage) DropIndex(name, column string) {
	if table, ok := b.tables[name]; ok {
		delete(table.Indexes, column)
	}
	b.persist()
}

func (b *BTreeStorage) InsertRow(name string, values []interface{}) Row {
	table := b.tables[name]
	row := make(Row)
	for i, col := range table.Columns {
		if i < len(values) {
			row[col] = values[i]
		}
	}
	table.Rows = append(table.Rows, row)

	for column, index := range table.Indexes {
		key := makeKey(row[column])
		index[key] = append(index[key], row)
	}

	b.persist()

	return row
}

func (b *BTreeStorage) SelectRows(table string, columns []string, where map[string]interface{}, join map[string]interface{}) []Row {
	if join != nil {
		return b.joinRows(table, columns, where, join)
	}

	rows := b.rowsFor(table, where)
	if len(columns) == 1 && columns[0] == "*" {
		result := make([]Row, len(rows))
		for i, row := range rows {
			result[i] = copyRow(row)
		}
		return result
	}

	var selected []Row
	for _, row := range rows {
		projected := make(Row)
		for _, col := range columns {
			parts := splitLast(col, ".")
			lookup := parts[len(parts)-1]
			projected[col] = row[lookup]
		}
		selected = append(selected, projected)
	}
	return selected
}

func (b *BTreeStorage) UpdateRows(table string, assignments map[string]interface{}, where map[string]interface{}) int {
	rows := b.rowsFor(table, where)
	for _, row := range rows {
		for k, v := range assignments {
			row[k] = v
		}
	}

	tableMeta := b.tables[table]
	for col := range assignments {
		if _, ok := tableMeta.Indexes[col]; ok {
			b.rebuildIndex(table, col)
		}
	}

	if len(rows) > 0 {
		b.persist()
	}
	return len(rows)
}

func (b *BTreeStorage) DeleteRows(table string, where map[string]interface{}) int {
	tableMeta := b.tables[table]
	if where == nil {
		deleted := len(tableMeta.Rows)
		tableMeta.Rows = make([]Row, 0)
		for col := range tableMeta.Indexes {
			b.rebuildIndex(table, col)
		}
		if deleted > 0 {
			b.persist()
		}
		return deleted
	}

	column := where["column"].(string)
	value := where["value"]
	var kept []Row
	for _, row := range tableMeta.Rows {
		if !valuesEqual(row[column], value) {
			kept = append(kept, row)
		}
	}
	deleted := len(tableMeta.Rows) - len(kept)
	tableMeta.Rows = kept

	for col := range tableMeta.Indexes {
		b.rebuildIndex(table, col)
	}

	if deleted > 0 {
		b.persist()
	}
	return deleted
}

func (b *BTreeStorage) rowsFor(table string, where map[string]interface{}) []Row {
	tableMeta := b.tables[table]
	if where == nil {
		return tableMeta.Rows
	}

	column := where["column"].(string)
	value := where["value"]

	if index, ok := tableMeta.Indexes[column]; ok {
		return index[makeKey(value)]
	}

	var result []Row
	for _, row := range tableMeta.Rows {
		if valuesEqual(row[column], value) {
			result = append(result, row)
		}
	}
	return result
}

func (b *BTreeStorage) joinRows(table string, columns []string, where, join map[string]interface{}) []Row {
	leftRows := b.rowsFor(table, where)
	rightTable := join["table"].(string)
	rightMeta := b.tables[rightTable]
	rightColumn := join["right_column"].(string)

	index, ok := rightMeta.Indexes[rightColumn]
	if !ok {
		b.rebuildIndex(rightTable, rightColumn)
		index = rightMeta.Indexes[rightColumn]
	}

	leftTableName := join["left_table"].(string)
	rightTableName := join["right_table"].(string)
	leftColumn := join["left_column"].(string)

	var result []Row
	for _, left := range leftRows {
		key := makeKey(left[leftColumn])
		for _, right := range index[key] {
			combined := make(Row)
			for _, col := range b.tables[table].Columns {
				combined[leftTableName+"."+col] = left[col]
			}
			for _, col := range rightMeta.Columns {
				combined[rightTableName+"."+col] = right[col]
			}

			if len(columns) == 1 && columns[0] == "*" {
				result = append(result, combined)
			} else {
				projected := make(Row)
				for _, col := range columns {
					if contains(col, ".") {
						projected[col] = combined[col]
					} else if containsStr(b.tables[table].Columns, col) {
						projected[col] = left[col]
					} else if containsStr(rightMeta.Columns, col) {
						projected[col] = right[col]
					} else {
						projected[col] = nil
					}
				}
				result = append(result, projected)
			}
		}
	}
	return result
}

func (b *BTreeStorage) rebuildIndex(table, column string) {
	tableMeta := b.tables[table]
	index := make(Index)
	for _, row := range tableMeta.Rows {
		key := makeKey(row[column])
		index[key] = append(index[key], row)
	}
	tableMeta.Indexes[column] = index
}

func (b *BTreeStorage) Describe() map[string]map[string]interface{} {
	summary := make(map[string]map[string]interface{})
	for name, meta := range b.tables {
		var indexKeys []string
		for k := range meta.Indexes {
			indexKeys = append(indexKeys, k)
		}
		summary[name] = map[string]interface{}{
			"columns":   meta.Columns,
			"row_count": len(meta.Rows),
			"indexes":   indexKeys,
		}
	}
	return summary
}

func (b *BTreeStorage) load() {
	if b.pager == nil {
		return
	}
	data := b.pager.ReadBlob()
	if len(data) == 0 {
		return
	}
	var dump map[string]tableDump
	if err := json.Unmarshal(data, &dump); err != nil {
		return
	}
	for name, meta := range dump {
		rows := make([]Row, len(meta.Rows))
		for i, src := range meta.Rows {
			row := make(Row)
			for k, v := range src {
				row[k] = v
			}
			rows[i] = row
		}
		table := &TableMeta{
			Columns: append([]string{}, meta.Columns...),
			Rows:    rows,
			Indexes: make(map[string]Index),
		}
		b.tables[name] = table
		for _, column := range meta.Indexes {
			b.rebuildIndex(name, column)
		}
	}
}

func (b *BTreeStorage) persist() {
	if b.pager == nil {
		return
	}
	dump := make(map[string]tableDump)
	for name, table := range b.tables {
		rows := make([]Row, len(table.Rows))
		for i, src := range table.Rows {
			row := make(Row)
			for k, v := range src {
				row[k] = v
			}
			rows[i] = row
		}
		var indexes []string
		for col := range table.Indexes {
			indexes = append(indexes, col)
		}
		dump[name] = tableDump{
			Columns: append([]string{}, table.Columns...),
			Rows:    rows,
			Indexes: indexes,
		}
	}
	data, err := json.Marshal(dump)
	if err != nil {
		return
	}
	b.pager.WriteBlob(data)
}

func valuesEqual(a, b interface{}) bool {
	return makeKey(a) == makeKey(b)
}

func copyRow(row Row) Row {
	result := make(Row)
	for k, v := range row {
		result[k] = v
	}
	return result
}

func makeKey(v interface{}) string {
	switch val := v.(type) {
	case int:
		return fmt.Sprintf("i:%d", val)
	case int32:
		return fmt.Sprintf("i:%d", val)
	case int64:
		return fmt.Sprintf("i:%d", val)
	case float64:
		return fmt.Sprintf("f:%g", val)
	case float32:
		return fmt.Sprintf("f:%g", val)
	case string:
		return "s:" + val
	case nil:
		return "n:null"
	default:
		return fmt.Sprintf("o:%v", val)
	}
}

func splitLast(s, sep string) []string {
	idx := -1
	for i := len(s) - 1; i >= 0; i-- {
		if s[i] == sep[0] {
			idx = i
			break
		}
	}
	if idx == -1 {
		return []string{s}
	}
	return []string{s[:idx], s[idx+1:]}
}

func contains(s, substr string) bool {
	for i := 0; i < len(s); i++ {
		if i+len(substr) <= len(s) && s[i:i+len(substr)] == substr {
			return true
		}
	}
	return false
}

func containsStr(arr []string, s string) bool {
	for _, item := range arr {
		if item == s {
			return true
		}
	}
	return false
}
