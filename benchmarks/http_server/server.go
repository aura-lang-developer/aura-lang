package main

import (
	"encoding/json"
	"fmt"
	"net/http"
	"os"
	"time"
)

type BenchResponse struct {
	Status string `json:"status"`
	Engine string `json:"engine"`
	Time   int64  `json:"time"`
}

func main() {
	mux := http.NewServeMux()
	mux.HandleFunc("/bench", func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Content-Type", "application/json; charset=utf-8")
		w.WriteHeader(http.StatusOK)
		json.NewEncoder(w).Encode(BenchResponse{
			Status: "ok",
			Engine: "golang",
			Time:   time.Now().UnixMilli(),
		})
	})

	port := os.Getenv("PORT")
	if port == "" {
		port = "8092"
	}
	fmt.Printf("Go HTTP server listening on :%s\n", port)
	http.ListenAndServe(":"+port, mux)
}
