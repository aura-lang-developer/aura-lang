package main

import (
	"fmt"
	"sync"
	"time"
)

func main() {
	var mu sync.Mutex
	count := 50000
	store := make([]any, 0, 1000)
	for i := 0; i < 1000; i++ {
		store = append(store, i*10)
	}

	var wg sync.WaitGroup
	wg.Add(count)

	t0 := time.Now()
	for j := 0; j < count; j++ {
		opId := j
		go func() {
			key := opId % 1000
			mu.Lock()
			if opId%3 == 0 {
				store[key] = opId
			}
			readVal := store[key]
			if readVal == 0 {
				opId = opId + 1
			}
			mu.Unlock()
			wg.Done()
		}()
	}

	wg.Wait()
	dur := time.Since(t0)
	fmt.Printf("[GO] KV Cache: %d concurrent ops completed in %d ms\n", count, dur.Milliseconds())
}
