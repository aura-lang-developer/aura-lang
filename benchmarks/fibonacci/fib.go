package main

import (
	"fmt"
	"time"
)

func fib(n int) int {
	if n <= 1 {
		return n
	}
	return fib(n-1) + fib(n-2)
}

func main() {
	n := 38
	t0 := time.Now()
	res := fib(n)
	dur := time.Since(t0)
	fmt.Printf("[GO] Fib(%d) = %d | Time: %d ms\n", n, res, dur.Milliseconds())
}
