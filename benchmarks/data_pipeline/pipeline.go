package main

import (
	"fmt"
	"time"
)

func main() {
	count := 1000000
	data := make([]int, count)
	for i := 0; i < count; i++ {
		data[i] = i
	}

	t0 := time.Now()
	// Filter
	evens := make([]int, 0, count/2)
	for _, x := range data {
		if x%2 == 0 {
			evens = append(evens, x)
		}
	}
	// Map
	mapped := make([]int, len(evens))
	for i, x := range evens {
		mapped[i] = x * 3
	}
	// Reduce / Sum
	sum := int64(0)
	for _, x := range mapped {
		sum += int64(x)
	}
	dur := time.Since(t0)

	fmt.Printf("[GO] Processed %d items | Sum: %d | Time: %d ms\n", count, sum, dur.Milliseconds())
}
