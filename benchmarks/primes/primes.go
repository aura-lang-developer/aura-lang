package main

import (
	"fmt"
	"time"
)

func countPrimes(limit int) int {
	isPrime := make([]bool, limit+1)
	for i := 0; i <= limit; i++ {
		isPrime[i] = true
	}

	for p := 2; p*p <= limit; p++ {
		if isPrime[p] {
			for multiple := p * p; multiple <= limit; multiple += p {
				isPrime[multiple] = false
			}
		}
	}

	count := 0
	for k := 2; k <= limit; k++ {
		if isPrime[k] {
			count++
		}
	}
	return count
}

func main() {
	limit := 2000000
	t0 := time.Now()
	total := countPrimes(limit)
	dur := time.Since(t0)
	fmt.Printf("[GO] Primes up to %d = %d | Time: %d ms\n", limit, total, dur.Milliseconds())
}
