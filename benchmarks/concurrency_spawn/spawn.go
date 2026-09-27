package main

import (
	"fmt"
	"sync"
	"time"
)

func main() {
	count := 50000
	var wg sync.WaitGroup
	wg.Add(count)

	t0 := time.Now()
	for i := 0; i < count; i++ {
		go func() {
			wg.Done()
		}()
	}

	wg.Wait()
	dur := time.Since(t0)
	fmt.Printf("[GO] Spawned %d goroutines in %d ms\n", count, dur.Milliseconds())
}
