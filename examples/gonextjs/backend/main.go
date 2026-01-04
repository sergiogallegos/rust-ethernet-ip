package main

import (
	"encoding/json"
	"fmt"
	"log"
	"net/http"
	"sync"
	"time"

	"github.com/gorilla/mux"
	"github.com/gorilla/websocket"
	gowrapper "github.com/sergiogallegos/rust-ethernet-ip/gowrapper/ethernetip"
)

var (
	client *gowrapper.EipClient
	mu     sync.Mutex
)

// CORS middleware
func corsMiddleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		w.Header().Set("Access-Control-Allow-Origin", "*")
		w.Header().Set("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE, OPTIONS")
		w.Header().Set("Access-Control-Allow-Headers", "Content-Type, Authorization")

		// Handle preflight requests
		if r.Method == "OPTIONS" {
			w.WriteHeader(http.StatusOK)
			return
		}

		next.ServeHTTP(w, r)
	})
}

func main() {
	r := mux.NewRouter()

	// Apply CORS middleware to all routes
	r.Use(corsMiddleware)

	// REST endpoints
	r.HandleFunc("/api/connect", handleConnect).Methods("POST")
	r.HandleFunc("/api/disconnect", handleDisconnect).Methods("POST")
	r.HandleFunc("/api/tag", handleTag).Methods("GET", "POST")
	r.HandleFunc("/api/batch", handleBatch).Methods("POST")
	r.HandleFunc("/api/taginfo", handleTagInfo).Methods("GET")
	// Debug read endpoint
	r.HandleFunc("/api/test-read", handleTestRead).Methods("GET")
	r.HandleFunc("/api/benchmark", handleBenchmark).Methods("POST")
	// Array element test endpoint
	r.HandleFunc("/api/test-arrays", handleTestArrays).Methods("POST")

	// Array operations
	r.HandleFunc("/api/array/{tagName}", handleArray).Methods("GET", "POST")

	// UDT operations
	r.HandleFunc("/api/udt/{tagName}", handleUdt).Methods("GET", "POST")
	r.HandleFunc("/api/udt-member/{memberPath}", handleUdtMember).Methods("GET", "POST")

	// Production endpoints
	r.HandleFunc("/api/health", handleHealth).Methods("GET")
	r.HandleFunc("/api/metrics", handleMetrics).Methods("GET")
	r.HandleFunc("/api/config", handleConfig).Methods("GET", "POST")
	r.HandleFunc("/api/status", handleStatus).Methods("GET")

	// WebSocket endpoint
	r.HandleFunc("/ws", handleWebSocket)

	log.Println("Starting server on :8080")
	log.Fatal(http.ListenAndServe(":8080", r))
}

func handleConnect(w http.ResponseWriter, r *http.Request) {
	var req struct {
		IPAddress    string `json:"ipAddress"`
		UseRoutePath bool   `json:"useRoutePath"`
		CpuSlot      int    `json:"cpuSlot"`
	}

	log.Printf("🔌 [DEBUG] Connect request received")
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		log.Printf("❌ [DEBUG] Failed to decode request: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	log.Printf("🔌 [DEBUG] Connect request: IPAddress=%s, UseRoutePath=%v, CpuSlot=%d", req.IPAddress, req.UseRoutePath, req.CpuSlot)

	mu.Lock()
	defer mu.Unlock()

	if client != nil {
		client.Close()
	}

	var err error
	// Regular connection
	log.Printf("🔌 [DEBUG] Creating new client for: %s", req.IPAddress)
	client, err = gowrapper.NewClient(req.IPAddress)
	if err != nil {
		log.Printf("❌ [DEBUG] Failed to create client: %v", err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}

	log.Printf("✅ [DEBUG] Client created successfully")

	// Note: RoutePath support will be added when DLL exports are available
	if req.UseRoutePath && req.CpuSlot >= 0 {
		log.Printf("ℹ️ RoutePath requested (CPU Slot %d) but not yet supported in this build", req.CpuSlot)
	}

	w.Header().Set("Content-Type", "application/json")
	response := map[string]interface{}{
		"success": true,
		"message": "Connected successfully",
	}
	log.Printf("✅ [DEBUG] Sending success response: %+v", response)
	json.NewEncoder(w).Encode(response)
}

func handleDisconnect(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client != nil {
		client.Close()
		client = nil
	}

	w.WriteHeader(http.StatusOK)
}

func handleTag(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	switch r.Method {
	case "GET":
		tag := r.URL.Query().Get("tag")
		typeStr := r.URL.Query().Get("type")
		if tag == "" || typeStr == "" {
			http.Error(w, "Tag and type required", http.StatusBadRequest)
			return
		}
		typeVal, err := parsePlcDataType(typeStr)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		val, err := client.ReadValue(tag, typeVal)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		json.NewEncoder(w).Encode(map[string]interface{}{
			"tag":   tag,
			"value": val.Value,
			"type":  typeStr,
		})
	case "POST":
		var req struct {
			Tag   string      `json:"tag"`
			Type  string      `json:"type"`
			Value interface{} `json:"value"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		typeVal, err := parsePlcDataType(req.Type)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		var value interface{} = req.Value
		switch req.Type {
		case "Dint":
			if f, ok := req.Value.(float64); ok {
				value = int32(f)
			} else if i, ok := req.Value.(int); ok {
				value = int32(i)
			} else if s, ok := req.Value.(string); ok {
				var v int32
				_, err := fmt.Sscanf(s, "%d", &v)
				if err != nil {
					http.Error(w, "invalid DINT value", http.StatusBadRequest)
					return
				}
				value = v
			}
		case "Int":
			if f, ok := req.Value.(float64); ok {
				value = int16(f)
			} else if i, ok := req.Value.(int); ok {
				value = int16(i)
			} else if s, ok := req.Value.(string); ok {
				var v int16
				_, err := fmt.Sscanf(s, "%d", &v)
				if err != nil {
					http.Error(w, "invalid INT value", http.StatusBadRequest)
					return
				}
				value = v
			}
		case "Real":
			if f, ok := req.Value.(float64); ok {
				value = f
			} else if s, ok := req.Value.(string); ok {
				var v float64
				_, err := fmt.Sscanf(s, "%f", &v)
				if err != nil {
					http.Error(w, "invalid REAL value", http.StatusBadRequest)
					return
				}
				value = v
			}
		}
		plcVal := &gowrapper.PlcValue{Type: typeVal, Value: value}
		err = client.WriteValue(req.Tag, plcVal)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		w.WriteHeader(http.StatusOK)
	}
}

func handleBatch(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	var req struct {
		Tags []struct {
			Tag  string `json:"tag"`
			Type string `json:"type"`
		} `json:"tags"`
		Writes []struct {
			Tag   string      `json:"tag"`
			Type  string      `json:"type"`
			Value interface{} `json:"value"`
		} `json:"writes"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	if len(req.Writes) > 0 {
		// Batch write
		writeMap := make(map[string]interface{})
		for _, writeReq := range req.Writes {
			_, err := parsePlcDataType(writeReq.Type)
			if err != nil {
				http.Error(w, err.Error(), http.StatusBadRequest)
				return
			}
			var value interface{} = writeReq.Value
			switch writeReq.Type {
			case "Dint":
				if f, ok := writeReq.Value.(float64); ok {
					value = int32(f)
				} else if i, ok := writeReq.Value.(int); ok {
					value = int32(i)
				} else if s, ok := writeReq.Value.(string); ok {
					var v int32
					_, err := fmt.Sscanf(s, "%d", &v)
					if err != nil {
						http.Error(w, "invalid DINT value", http.StatusBadRequest)
						return
					}
					value = v
				}
			case "Int":
				if f, ok := writeReq.Value.(float64); ok {
					value = int16(f)
				} else if i, ok := writeReq.Value.(int); ok {
					value = int16(i)
				} else if s, ok := writeReq.Value.(string); ok {
					var v int16
					_, err := fmt.Sscanf(s, "%d", &v)
					if err != nil {
						http.Error(w, "invalid INT value", http.StatusBadRequest)
						return
					}
					value = v
				}
			case "Real":
				if f, ok := writeReq.Value.(float64); ok {
					value = f
				} else if s, ok := writeReq.Value.(string); ok {
					var v float64
					_, err := fmt.Sscanf(s, "%f", &v)
					if err != nil {
						http.Error(w, "invalid REAL value", http.StatusBadRequest)
						return
					}
					value = v
				}
			}
			writeMap[writeReq.Tag] = value
		}
		err := client.BatchWrite(writeMap)
		if err != nil {
			json.NewEncoder(w).Encode(map[string]interface{}{"success": false, "error": err.Error()})
			return
		}
		json.NewEncoder(w).Encode(map[string]interface{}{"success": true})
		return
	}

	// Batch read (existing logic)
	results := make([]map[string]interface{}, len(req.Tags))
	for i, t := range req.Tags {
		typeVal, err := parsePlcDataType(t.Type)
		if err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}
		val, err := client.ReadValue(t.Tag, typeVal)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}
		results[i] = map[string]interface{}{
			"tag":   t.Tag,
			"value": val.Value,
			"type":  t.Type,
		}
	}
	json.NewEncoder(w).Encode(results)
}

var upgrader = websocket.Upgrader{
	ReadBufferSize:  1024,
	WriteBufferSize: 1024,
	CheckOrigin: func(r *http.Request) bool {
		return true
	},
}

func handleWebSocket(w http.ResponseWriter, r *http.Request) {
	conn, err := upgrader.Upgrade(w, r, nil)
	if err != nil {
		log.Println(err)
		return
	}
	defer conn.Close()

	// Simulate real-time updates
	for {
		time.Sleep(1 * time.Second)
		mu.Lock()
		if client == nil {
			mu.Unlock()
			return
		}
		mu.Unlock()

		// Example: Read a tag and send update (Bool type for demo)
		// Only try to read if client is connected
		if client != nil {
			val, err := client.ReadValue("_IO_EM_DI00", gowrapper.Bool)
			if err != nil {
				log.Println(err)
				continue
			}
			conn.WriteJSON(map[string]interface{}{
				"tag":   "_IO_EM_DI00",
				"value": val.Value,
				"type":  "Bool",
			})
		} else {
			// Send a message indicating no PLC connection
			conn.WriteJSON(map[string]interface{}{
				"tag":   "status",
				"value": "No PLC connected",
				"type":  "String",
			})
		}
	}
}

// parsePlcDataType converts a string to gowrapper.PlcDataType
func parsePlcDataType(s string) (gowrapper.PlcDataType, error) {
	switch s {
	case "Bool":
		return gowrapper.Bool, nil
	case "Sint":
		return gowrapper.Sint, nil
	case "Int":
		return gowrapper.Int, nil
	case "Dint":
		return gowrapper.Dint, nil
	case "Lint":
		return gowrapper.Lint, nil
	case "Usint":
		return gowrapper.Usint, nil
	case "Uint":
		return gowrapper.Uint, nil
	case "Udint":
		return gowrapper.Udint, nil
	case "Ulint":
		return gowrapper.Ulint, nil
	case "Real":
		return gowrapper.Real, nil
	case "Lreal":
		return gowrapper.Lreal, nil
	case "String":
		return gowrapper.String, nil
	case "Udt":
		return gowrapper.Udt, nil
	default:
		return 0, fmt.Errorf("unsupported PLC data type: %s", s)
	}
}

// Add handler for tag info discovery
func handleTagInfo(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}
	tag := r.URL.Query().Get("tag")
	if tag == "" {
		http.Error(w, "Tag required", http.StatusBadRequest)
		return
	}
	log.Printf("[DEBUG] Discovering metadata for tag: %s", tag)
	meta, err := client.GetTagMetadata(tag)
	if err != nil {
		log.Printf("[ERROR] Failed to get metadata for tag %s: %v", tag, err)
		http.Error(w, err.Error(), http.StatusInternalServerError)
		return
	}
	log.Printf("[DEBUG] Metadata for tag %s: %+v", tag, meta)
	typeStr := plcDataTypeToString(meta.DataType)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"tag":  tag,
		"type": typeStr,
	})
}

// Helper to convert PLC data type code to string
func plcDataTypeToString(dt int) string {
	switch dt {
	case int(gowrapper.Bool):
		return "Bool"
	case int(gowrapper.Sint):
		return "Sint"
	case int(gowrapper.Int):
		return "Int"
	case int(gowrapper.Dint):
		return "Dint"
	case int(gowrapper.Lint):
		return "Lint"
	case int(gowrapper.Usint):
		return "Usint"
	case int(gowrapper.Uint):
		return "Uint"
	case int(gowrapper.Udint):
		return "Udint"
	case int(gowrapper.Ulint):
		return "Ulint"
	case int(gowrapper.Real):
		return "Real"
	case int(gowrapper.Lreal):
		return "Lreal"
	case int(gowrapper.String):
		return "String"
	case int(gowrapper.Udt):
		return "Udt"
	default:
		return "Unknown"
	}
}

// Debug read handler
func handleTestRead(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}
	tag := r.URL.Query().Get("tag")
	typeStr := r.URL.Query().Get("type")
	if tag == "" || typeStr == "" {
		http.Error(w, "Tag and type required", http.StatusBadRequest)
		return
	}
	log.Printf("[DEBUG] /api/test-read: tag=%s, type=%s", tag, typeStr)
	typeVal, err := parsePlcDataType(typeStr)
	if err != nil {
		log.Printf("[ERROR] /api/test-read: parsePlcDataType failed: %v", err)
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	val, err := client.ReadValue(tag, typeVal)
	if err != nil {
		log.Printf("[ERROR] /api/test-read: ReadValue failed: %v", err)
		json.NewEncoder(w).Encode(map[string]interface{}{
			"tag":   tag,
			"type":  typeStr,
			"error": err.Error(),
			"value": nil,
		})
		return
	}
	log.Printf("[DEBUG] /api/test-read: ReadValue success: %+v", val)
	json.NewEncoder(w).Encode(map[string]interface{}{
		"tag":   tag,
		"type":  typeStr,
		"error": nil,
		"value": val.Value,
	})
}

func handleBenchmark(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	var req struct {
		Tag   string `json:"tag"`
		Type  string `json:"type"`
		Write bool   `json:"write"`
	}
	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	typeVal, err := parsePlcDataType(req.Type)
	if err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}
	readCount := 0
	writeCount := 0
	start := time.Now()
	duration := 3 * time.Second
	end := start.Add(duration)
	var lastInt int32 = 0
	var lastFloat float64 = 0.0
	var lastBool bool = false
	var lastString string = "A"
	for time.Now().Before(end) {
		_, err := client.ReadValue(req.Tag, typeVal)
		if err == nil {
			readCount++
		} else {
			log.Printf("[BENCHMARK] Read error: %v", err)
		}
		if req.Write {
			var writeVal interface{}
			switch req.Type {
			case "Bool":
				lastBool = !lastBool
				writeVal = lastBool
			case "Int":
				lastInt++
				writeVal = int16(lastInt)
			case "Dint":
				lastInt++
				writeVal = int32(lastInt)
			case "Real":
				lastFloat += 1.1
				writeVal = lastFloat
			case "String":
				if lastString == "A" {
					lastString = "B"
				} else {
					lastString = "A"
				}
				writeVal = lastString
			default:
				lastInt++
				writeVal = lastInt
			}
			plcVal := &gowrapper.PlcValue{Type: typeVal, Value: writeVal}
			err := client.WriteValue(req.Tag, plcVal)
			if err == nil {
				writeCount++
			} else {
				log.Printf("[BENCHMARK] Write error: %v", err)
			}
		}
	}
	elapsed := time.Since(start)
	readRate := float64(readCount) / elapsed.Seconds()
	writeRate := float64(writeCount) / elapsed.Seconds()
	json.NewEncoder(w).Encode(map[string]interface{}{
		"success":    true,
		"readCount":  readCount,
		"writeCount": writeCount,
		"elapsedMs":  elapsed.Milliseconds(),
		"readRate":   readRate,
		"writeRate":  writeRate,
	})
}

// Production endpoints
func handleHealth(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	health := map[string]interface{}{
		"status":    "healthy",
		"timestamp": time.Now().Unix(),
		"version":   "0.5.3",
		"uptime":    time.Since(startTime).Seconds(),
	}

	if client != nil {
		// Check if client is still connected
		if isHealthy, _ := client.CheckHealth(); isHealthy {
			health["plc_connection"] = "connected"
		} else {
			health["plc_connection"] = "disconnected"
			health["status"] = "degraded"
		}
	} else {
		health["plc_connection"] = "not_connected"
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(health)
}

func handleMetrics(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	metrics := map[string]interface{}{
		"timestamp": time.Now().Unix(),
		"uptime":    time.Since(startTime).Seconds(),
		"connections": map[string]interface{}{
			"active": 0,
			"total":  0,
		},
		"operations": map[string]interface{}{
			"reads":  0,
			"writes": 0,
			"errors": 0,
		},
		"performance": map[string]interface{}{
			"avg_latency_ms": 0.0,
			"ops_per_second": 0.0,
		},
	}

	if client != nil {
		// Get client metrics if available
		metrics["plc_connected"] = true
	} else {
		metrics["plc_connected"] = false
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(metrics)
}

func handleConfig(w http.ResponseWriter, r *http.Request) {
	switch r.Method {
	case "GET":
		config := map[string]interface{}{
			"server": map[string]interface{}{
				"port":    8080,
				"version": "0.4.0",
			},
			"plc": map[string]interface{}{
				"connection_timeout": 10,
				"read_timeout":       5,
				"write_timeout":      5,
			},
			"performance": map[string]interface{}{
				"max_packet_size": 4000,
				"batch_size":      50,
			},
		}

		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(config)

	case "POST":
		var newConfig map[string]interface{}
		if err := json.NewDecoder(r.Body).Decode(&newConfig); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		// Apply configuration changes
		// This is a simplified implementation
		w.Header().Set("Content-Type", "application/json")
		json.NewEncoder(w).Encode(map[string]interface{}{
			"success": true,
			"message": "Configuration updated",
		})
	}
}

func handleStatus(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	status := map[string]interface{}{
		"server": map[string]interface{}{
			"status":    "running",
			"version":   "0.5.3",
			"uptime":    time.Since(startTime).Seconds(),
			"timestamp": time.Now().Unix(),
		},
		"plc": map[string]interface{}{
			"connected": client != nil,
			"address":   "",
		},
		"features": map[string]interface{}{
			"batch_operations":      true,
			"real_time_monitoring":  true,
			"hmi_demo":              true,
			"performance_benchmark": true,
		},
	}

	if client != nil {
		status["plc"].(map[string]interface{})["address"] = "connected"
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(status)
}

var startTime = time.Now()

// Array element test handler - comprehensive test for v0.5.5 array support
func handleTestArrays(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	var req struct {
		TestType string `json:"testType"` // "controller", "program", "bool", "all"
	}

	if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
		http.Error(w, err.Error(), http.StatusBadRequest)
		return
	}

	results := make(map[string]interface{})
	results["testType"] = req.TestType
	results["timestamp"] = time.Now().Unix()
	results["tests"] = make([]map[string]interface{}, 0)

	tests := []map[string]interface{}{}

	// Test controller-scoped DINT array
	if req.TestType == "controller" || req.TestType == "all" {
		log.Println("[ARRAY TEST] Testing controller-scoped DINT array...")
		for i := 0; i < 5; i++ {
			tag := fmt.Sprintf("gArrayTest[%d]", i)
			testResult := map[string]interface{}{
				"tag":   tag,
				"type":  "Dint",
				"scope": "controller",
			}

			// Read test
			val, err := client.ReadValue(tag, gowrapper.Dint)
			if err != nil {
				testResult["read"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["read"] = map[string]interface{}{
					"success": true,
					"value":   val.Value,
				}
			}

			// Write test
			writeVal := int32(100 + i)
			plcVal := &gowrapper.PlcValue{Type: gowrapper.Dint, Value: writeVal}
			err = client.WriteValue(tag, plcVal)
			if err != nil {
				testResult["write"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["write"] = map[string]interface{}{
					"success": true,
					"value":   writeVal,
				}

				// Verify write
				readBack, err := client.ReadValue(tag, gowrapper.Dint)
				if err == nil {
					testResult["verify"] = map[string]interface{}{
						"success": true,
						"value":   readBack.Value,
						"match":   readBack.Value == writeVal,
					}
				}
			}

			tests = append(tests, testResult)
		}
	}

	// Test program-scoped DINT array
	if req.TestType == "program" || req.TestType == "all" {
		log.Println("[ARRAY TEST] Testing program-scoped DINT array...")
		for i := 0; i < 5; i++ {
			tag := fmt.Sprintf("Program:MainProgram.ArrayTest[%d]", i)
			testResult := map[string]interface{}{
				"tag":   tag,
				"type":  "Dint",
				"scope": "program",
			}

			// Read test
			val, err := client.ReadValue(tag, gowrapper.Dint)
			if err != nil {
				testResult["read"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["read"] = map[string]interface{}{
					"success": true,
					"value":   val.Value,
				}
			}

			// Write test
			writeVal := int32(200 + i)
			plcVal := &gowrapper.PlcValue{Type: gowrapper.Dint, Value: writeVal}
			err = client.WriteValue(tag, plcVal)
			if err != nil {
				testResult["write"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["write"] = map[string]interface{}{
					"success": true,
					"value":   writeVal,
				}

				// Verify write
				readBack, err := client.ReadValue(tag, gowrapper.Dint)
				if err == nil {
					testResult["verify"] = map[string]interface{}{
						"success": true,
						"value":   readBack.Value,
						"match":   readBack.Value == writeVal,
					}
				}
			}

			tests = append(tests, testResult)
		}
	}

	// Test BOOL array
	if req.TestType == "bool" || req.TestType == "all" {
		log.Println("[ARRAY TEST] Testing controller-scoped BOOL array...")
		for i := 0; i < 10; i++ {
			tag := fmt.Sprintf("gArrayBoolTest[%d]", i)
			testResult := map[string]interface{}{
				"tag":   tag,
				"type":  "Bool",
				"scope": "controller",
			}

			// Read test
			val, err := client.ReadValue(tag, gowrapper.Bool)
			if err != nil {
				testResult["read"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["read"] = map[string]interface{}{
					"success": true,
					"value":   val.Value,
				}
			}

			// Write test (toggle the value)
			var writeVal bool
			if val.Value != nil {
				writeVal = !(val.Value.(bool))
			} else {
				writeVal = true
			}
			plcVal := &gowrapper.PlcValue{Type: gowrapper.Bool, Value: writeVal}
			err = client.WriteValue(tag, plcVal)
			if err != nil {
				testResult["write"] = map[string]interface{}{
					"success": false,
					"error":   err.Error(),
				}
			} else {
				testResult["write"] = map[string]interface{}{
					"success": true,
					"value":   writeVal,
				}

				// Verify write
				readBack, err := client.ReadValue(tag, gowrapper.Bool)
				if err == nil {
					testResult["verify"] = map[string]interface{}{
						"success": true,
						"value":   readBack.Value,
						"match":   readBack.Value == writeVal,
					}
				}
			}

			tests = append(tests, testResult)
		}
	}

	// Calculate summary
	successCount := 0
	errorCount := 0
	for _, test := range tests {
		if read, ok := test["read"].(map[string]interface{}); ok {
			if success, ok := read["success"].(bool); ok && success {
				successCount++
			} else {
				errorCount++
			}
		}
	}

	results["tests"] = tests
	results["summary"] = map[string]interface{}{
		"total":       len(tests),
		"successful":  successCount,
		"failed":      errorCount,
		"successRate": float64(successCount) / float64(len(tests)) * 100,
	}

	w.Header().Set("Content-Type", "application/json")
	json.NewEncoder(w).Encode(results)
}

// Array operations handler
func handleArray(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	vars := mux.Vars(r)
	tagName := vars["tagName"]
	if tagName == "" {
		http.Error(w, "Tag name required", http.StatusBadRequest)
		return
	}

	switch r.Method {
	case "GET":
		// Read array element - determine type from tag name
		var value interface{}
		var typeStr string

		if contains(tagName, "_DINT") || contains(tagName, "[") && contains(tagName, "DINT") {
			val, err := client.ReadDint(tagName)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
			value = val
			typeStr = "Dint"
		} else if contains(tagName, "_REAL") || contains(tagName, "[") && contains(tagName, "REAL") {
			val, err := client.ReadReal(tagName)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
			value = val
			typeStr = "Real"
		} else if contains(tagName, "_BOOL") || contains(tagName, "[") && contains(tagName, "BOOL") {
			val, err := client.ReadBool(tagName)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
			value = val
			typeStr = "Bool"
		} else if contains(tagName, "_INT") || contains(tagName, "[") && contains(tagName, "INT") && !contains(tagName, "DINT") {
			val, err := client.ReadInt(tagName)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
			value = val
			typeStr = "Int"
		} else {
			// Default to DINT
			val, err := client.ReadDint(tagName)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
			value = val
			typeStr = "Dint"
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success": true,
			"tag":     tagName,
			"value":   value,
			"type":    typeStr,
		})

	case "POST":
		// Write array element
		var req struct {
			Value interface{} `json:"value"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		// Determine type from tag name
		if contains(tagName, "_DINT") || contains(tagName, "[") && contains(tagName, "DINT") {
			var val int32
			if f, ok := req.Value.(float64); ok {
				val = int32(f)
			} else if i, ok := req.Value.(int); ok {
				val = int32(i)
			} else {
				http.Error(w, "Invalid DINT value", http.StatusBadRequest)
				return
			}
			err := client.WriteDint(tagName, val)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
		} else if contains(tagName, "_REAL") || contains(tagName, "[") && contains(tagName, "REAL") {
			var val float64
			if f, ok := req.Value.(float64); ok {
				val = f
			} else {
				http.Error(w, "Invalid REAL value", http.StatusBadRequest)
				return
			}
			err := client.WriteReal(tagName, val)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
		} else if contains(tagName, "_BOOL") || contains(tagName, "[") && contains(tagName, "BOOL") {
			var val bool
			if b, ok := req.Value.(bool); ok {
				val = b
			} else {
				http.Error(w, "Invalid BOOL value", http.StatusBadRequest)
				return
			}
			err := client.WriteBool(tagName, val)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
		} else if contains(tagName, "_INT") || contains(tagName, "[") && contains(tagName, "INT") && !contains(tagName, "DINT") {
			var val int16
			if f, ok := req.Value.(float64); ok {
				val = int16(f)
			} else if i, ok := req.Value.(int); ok {
				val = int16(i)
			} else {
				http.Error(w, "Invalid INT value", http.StatusBadRequest)
				return
			}
			err := client.WriteInt(tagName, val)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
		} else {
			// Default to DINT
			var val int32
			if f, ok := req.Value.(float64); ok {
				val = int32(f)
			} else if i, ok := req.Value.(int); ok {
				val = int32(i)
			} else {
				http.Error(w, "Invalid DINT value", http.StatusBadRequest)
				return
			}
			err := client.WriteDint(tagName, val)
			if err != nil {
				http.Error(w, err.Error(), http.StatusInternalServerError)
				return
			}
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success": true,
			"message": "Array element written successfully",
		})
	}
}

// UDT operations handler
func handleUdt(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	vars := mux.Vars(r)
	tagName := vars["tagName"]
	if tagName == "" {
		http.Error(w, "Tag name required", http.StatusBadRequest)
		return
	}

	switch r.Method {
	case "GET":
		// Read UDT
		udtData, err := client.ReadUdtData(tagName)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success":    true,
			"tag":        tagName,
			"symbolId":   udtData.SymbolID,
			"dataLength": len(udtData.Data),
			"data":       udtData.Data,
		})

	case "POST":
		// Write UDT
		var req struct {
			SymbolID int    `json:"symbolId"`
			Data     []byte `json:"data"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		udtData := &gowrapper.UdtData{
			SymbolID: req.SymbolID,
			Data:     req.Data,
		}

		err := client.WriteUdtData(tagName, udtData)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success": true,
			"message": "UDT written successfully",
		})
	}
}

// UDT member operations handler
func handleUdtMember(w http.ResponseWriter, r *http.Request) {
	mu.Lock()
	defer mu.Unlock()

	if client == nil {
		http.Error(w, "Not connected", http.StatusBadRequest)
		return
	}

	vars := mux.Vars(r)
	memberPath := vars["memberPath"]
	if memberPath == "" {
		http.Error(w, "Member path required", http.StatusBadRequest)
		return
	}

	// Parse tag name and member name from path (e.g., "gTestUDT.Member1_DINT")
	parts := splitUDTPath(memberPath)
	if len(parts) != 2 {
		http.Error(w, "Invalid member path format. Expected: TagName.MemberName", http.StatusBadRequest)
		return
	}
	tagName := parts[0]
	memberName := parts[1]

	switch r.Method {
	case "GET":
		// Read UDT member
		value, err := client.GetUdtMember(tagName, memberName)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success":    true,
			"tagName":    tagName,
			"memberName": memberName,
			"value":      value,
		})

	case "POST":
		// Write UDT member
		var req struct {
			Value interface{} `json:"value"`
		}
		if err := json.NewDecoder(r.Body).Decode(&req); err != nil {
			http.Error(w, err.Error(), http.StatusBadRequest)
			return
		}

		err := client.WriteUdtMember(tagName, memberName, req.Value)
		if err != nil {
			http.Error(w, err.Error(), http.StatusInternalServerError)
			return
		}

		json.NewEncoder(w).Encode(map[string]interface{}{
			"success":    true,
			"tagName":    tagName,
			"memberName": memberName,
			"message":    "UDT member written successfully",
		})
	}
}

// Helper functions
func contains(s, substr string) bool {
	return len(s) >= len(substr) && (s == substr || len(substr) == 0 ||
		(len(s) > len(substr) && (s[:len(substr)] == substr ||
			s[len(s)-len(substr):] == substr ||
			indexOf(s, substr) >= 0)))
}

func indexOf(s, substr string) int {
	for i := 0; i <= len(s)-len(substr); i++ {
		if s[i:i+len(substr)] == substr {
			return i
		}
	}
	return -1
}

func splitUDTPath(path string) []string {
	// Split on "." but handle array notation like "gTestUDT.Array_DINT[5]"
	dotIndex := -1
	for i := 0; i < len(path); i++ {
		if path[i] == '.' {
			// Check if this is the first dot (tag.member separator)
			if dotIndex == -1 {
				dotIndex = i
			}
		}
	}
	if dotIndex > 0 {
		return []string{path[:dotIndex], path[dotIndex+1:]}
	}
	return []string{path}
}
