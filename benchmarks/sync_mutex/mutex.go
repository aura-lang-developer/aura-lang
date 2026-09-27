package main

import (
	"fmt"
	"sync"
	"time"
)

func main() {
	var mu sync.Mutex
	count := 50000
	counter := 0
	var wg sync.WaitGroup
	wg.Add(count)

	t0 := time.Now()
	for i := 0; i < count; i++ {
		go func() {
			mu.Lock()
			counter++
			mu.Unlock()
			wg.Done()
		}()
	}

	wg.Wait()
	dur := time.Since(t0)
	fmt.Printf("[GO] Synchronized counter: %d in %d ms\n", counter, dur.Milliseconds())
}
