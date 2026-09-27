package main

import (
	"fmt"
	"sync"
	"time"
)

func main() {
	rounds := 100000
	chA := make(chan int, 1)
	chB := make(chan int, 1)
	var wg sync.WaitGroup
	wg.Add(2)

	t0 := time.Now()

	// Goroutine 1 (Ping)
	go func() {
		defer wg.Done()
		for i := 0; i < rounds; i++ {
			chA <- i
			<-chB
		}
	}()

	// Goroutine 2 (Pong)
	go func() {
		defer wg.Done()
		for j := 0; j < rounds; j++ {
			<-chA
			chB <- j
		}
	}()

	wg.Wait()
	dur := time.Since(t0)
	totalMessages := rounds * 2
	fmt.Printf("[GO] Channel Ping-Pong: %d msgs in %d ms\n", totalMessages, dur.Milliseconds())
}
