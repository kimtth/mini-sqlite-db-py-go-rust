package storage

type LogEntry map[string]interface{}

type LSMTreeStorage struct {
	segments []LogEntry
}

func NewLSMTreeStorage() *LSMTreeStorage {
	return &LSMTreeStorage{segments: make([]LogEntry, 0)}
}

func (l *LSMTreeStorage) Log(entry LogEntry) {
	l.segments = append(l.segments, entry)
}

func (l *LSMTreeStorage) Pending() int {
	return len(l.segments)
}

func (l *LSMTreeStorage) Snapshot() []LogEntry {
	copySegments := make([]LogEntry, len(l.segments))
	copy(copySegments, l.segments)
	return copySegments
}

func (l *LSMTreeStorage) Commit() []LogEntry {
	flushed := make([]LogEntry, len(l.segments))
	copy(flushed, l.segments)
	l.segments = make([]LogEntry, 0)
	l.compact()
	return flushed
}

func (l *LSMTreeStorage) compact() {
	if len(l.segments) > 10 {
		l.segments = l.segments[len(l.segments)-10:]
	}
}
