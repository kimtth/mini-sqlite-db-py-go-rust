package storage

import (
	"bytes"
	"encoding/binary"
	"os"
	"path/filepath"
)

const (
	headerSize = 16
	magic      = "MDB1"
)

type Pager struct {
	PageSize int
	path     string
	pages    [][]byte
	length   int
}

func NewPager(path string, pageSize int) *Pager {
	_ = os.MkdirAll(filepath.Dir(path), 0o755)
	p := &Pager{PageSize: pageSize, path: path}
	p.load()
	return p
}

func (p *Pager) AllocatePage() int {
	p.pages = append(p.pages, make([]byte, p.PageSize))
	return len(p.pages) - 1
}

func (p *Pager) WritePage(index int, data []byte) {
	for index >= len(p.pages) {
		p.pages = append(p.pages, make([]byte, p.PageSize))
	}
	buf := make([]byte, p.PageSize)
	copy(buf, data)
	p.pages[index] = buf
	p.flush()
}

func (p *Pager) ReadPage(index int) []byte {
	if index < len(p.pages) {
		return append([]byte(nil), p.pages[index]...)
	}
	return nil
}

func (p *Pager) WriteBlob(data []byte) {
	p.length = len(data)
	if len(data) == 0 {
		p.pages = nil
		p.flush()
		return
	}
	total := ((len(data) + p.PageSize - 1) / p.PageSize) * p.PageSize
	buffer := make([]byte, total)
	copy(buffer, data)
	p.pages = make([][]byte, total/p.PageSize)
	for i := range p.pages {
		start := i * p.PageSize
		end := start + p.PageSize
		page := make([]byte, p.PageSize)
		copy(page, buffer[start:end])
		p.pages[i] = page
	}
	p.flush()
}

func (p *Pager) ReadBlob() []byte {
	if len(p.pages) == 0 || p.length == 0 {
		return nil
	}
	buffer := make([]byte, len(p.pages)*p.PageSize)
	for i, page := range p.pages {
		copy(buffer[i*p.PageSize:], page)
	}
	return buffer[:p.length]
}

func (p *Pager) Stats() map[string]int {
	return map[string]int{"pages": len(p.pages), "page_size": p.PageSize}
}

func (p *Pager) load() {
	data, err := os.ReadFile(p.path)
	if err != nil || len(data) < headerSize || string(data[:4]) != magic {
		return
	}
	storedSize := int(binary.LittleEndian.Uint32(data[4:8]))
	if storedSize > 0 {
		p.PageSize = storedSize
	}
	p.length = int(binary.LittleEndian.Uint64(data[8:16]))
	payload := data[headerSize:]
	p.pages = make([][]byte, 0)
	for offset := 0; offset < len(payload); offset += p.PageSize {
		end := offset + p.PageSize
		if end > len(payload) {
			end = len(payload)
		}
		chunk := make([]byte, p.PageSize)
		copy(chunk, payload[offset:end])
		p.pages = append(p.pages, chunk)
	}
}

func (p *Pager) flush() {
	if len(p.pages) == 0 {
		_ = os.WriteFile(p.path, []byte{}, 0o644)
		return
	}
	head := make([]byte, 12)
	binary.LittleEndian.PutUint32(head[:4], uint32(p.PageSize))
	binary.LittleEndian.PutUint64(head[4:], uint64(p.length))
	var buf bytes.Buffer
	buf.WriteString(magic)
	buf.Write(head)
	for _, page := range p.pages {
		buf.Write(page)
	}
	_ = os.WriteFile(p.path, buf.Bytes(), 0o644)
}
