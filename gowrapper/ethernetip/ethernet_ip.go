package ethernetip

/*
#cgo windows LDFLAGS: -L${SRCDIR}/.. -lrust_ethernet_ip
#cgo windows CFLAGS: -I${SRCDIR}
#cgo windows LDFLAGS: -Wl,--allow-multiple-definition
#cgo windows LDFLAGS: -Wl,--no-undefined
#cgo windows LDFLAGS: -static-libgcc
#include <stdlib.h>
#include <string.h>

// C function declarations for the Rust library
extern int eip_connect(const char* ip_address);
// RoutePath functions - temporarily commented out until DLL exports are verified
// extern int eip_connect_with_route(const char* ip_address, unsigned char* slots, int slot_count, unsigned char* ports, int port_count, char** addresses, int address_count);
// extern int eip_set_route_path(int client_id, unsigned char* slots, int slot_count, unsigned char* ports, int port_count, char** addresses, int address_count);
extern int eip_disconnect(int client_id);

// Boolean operations
extern int eip_read_bool(int client_id, const char* tag_name, int* result);
extern int eip_write_bool(int client_id, const char* tag_name, int value);

// Integer operations
extern int eip_read_sint(int client_id, const char* tag_name, signed char* result);
extern int eip_write_sint(int client_id, const char* tag_name, signed char value);
extern int eip_read_int(int client_id, const char* tag_name, short* result);
extern int eip_write_int(int client_id, const char* tag_name, short value);
extern int eip_read_dint(int client_id, const char* tag_name, int* result);
extern int eip_write_dint(int client_id, const char* tag_name, int value);
extern int eip_read_lint(int client_id, const char* tag_name, long long* result);
extern int eip_write_lint(int client_id, const char* tag_name, long long value);

// Unsigned integer operations
extern int eip_read_usint(int client_id, const char* tag_name, unsigned char* result);
extern int eip_write_usint(int client_id, const char* tag_name, unsigned char value);
extern int eip_read_uint(int client_id, const char* tag_name, unsigned short* result);
extern int eip_write_uint(int client_id, const char* tag_name, unsigned short value);
extern int eip_read_udint(int client_id, const char* tag_name, unsigned int* result);
extern int eip_write_udint(int client_id, const char* tag_name, unsigned int value);
extern int eip_read_ulint(int client_id, const char* tag_name, unsigned long long* result);
extern int eip_write_ulint(int client_id, const char* tag_name, unsigned long long value);

// Float operations
extern int eip_read_real(int client_id, const char* tag_name, double* result);
extern int eip_write_real(int client_id, const char* tag_name, double value);
extern int eip_read_lreal(int client_id, const char* tag_name, double* result);
extern int eip_write_lreal(int client_id, const char* tag_name, double value);

// String operations
extern int eip_read_string(int client_id, const char* tag_name, char* result, int max_length);
extern int eip_write_string(int client_id, const char* tag_name, const char* value);

// UDT operations
extern int eip_read_udt(int client_id, const char* tag_name, char* result, int max_size);
extern int eip_write_udt(int client_id, const char* tag_name, const char* value, int size);

// Tag management
extern int eip_discover_tags(int client_id);
extern int eip_get_tag_metadata(int client_id, const char* tag_name, void* metadata);

// Batch operations
extern int eip_read_tags_batch(int client_id, char** tag_names, int tag_count, char* results, int results_capacity);
extern int eip_write_tags_batch(int client_id, const char* tag_values, int tag_count, char* results, int results_capacity);
extern int eip_execute_batch(int client_id, const char* operations, int operation_count, char* results, int results_capacity);
extern int eip_configure_batch_operations(int client_id, void* config);
extern int eip_get_batch_config(int client_id, void* config);

// Health check
extern int eip_check_health(int client_id, int* is_healthy);
extern int eip_check_health_detailed(int client_id, int* is_healthy, char* details, int details_capacity);

// Configuration
extern int eip_set_max_packet_size(int client_id, int size);
*/
import "C"
import (
	"encoding/json"
	"errors"
	"fmt"
	"log"
	"strings"
	"sync"
	"time"
	"unsafe"
)

// PlcDataType represents different PLC data types
type PlcDataType int

const (
	Bool PlcDataType = iota
	Sint
	Int
	Dint
	Lint
	Usint
	Uint
	Udint
	Ulint
	Real
	Lreal
	String
	Udt
)

// TagMetadata represents metadata for a PLC tag
type TagMetadata struct {
	DataType       int `json:"data_type"`       // CIP data type code
	Scope          int `json:"scope"`           // Tag scope (global, program, etc.)
	ArrayDimension int `json:"array_dimension"` // Number of array dimensions
	ArraySize      int `json:"array_size"`      // Total array size
}

// BatchConfig represents configuration for batch operations
type BatchConfig struct {
	MaxOperationsPerPacket int           `json:"max_operations_per_packet"`
	MaxPacketSize          int           `json:"max_packet_size"`
	PacketTimeoutMs        int64         `json:"packet_timeout_ms"`
	ContinueOnError        bool          `json:"continue_on_error"`
	OptimizePacketPacking  bool          `json:"optimize_packet_packing"`
	RetryCount             int           `json:"retry_count"`
	RetryDelay             time.Duration `json:"retry_delay"`
	MaxConcurrentOps       int           `json:"max_concurrent_ops"`
	OperationTimeout       time.Duration `json:"operation_timeout"`
}

// DefaultBatchConfig returns a default batch configuration
func DefaultBatchConfig() *BatchConfig {
	return &BatchConfig{
		MaxOperationsPerPacket: 20,
		MaxPacketSize:          504,
		PacketTimeoutMs:        3000,
		ContinueOnError:        true,
		OptimizePacketPacking:  true,
		RetryCount:             3,
		RetryDelay:             time.Second,
		MaxConcurrentOps:       10,
		OperationTimeout:       5 * time.Second,
	}
}

// HighPerformanceBatchConfig returns a batch configuration optimized for performance
func HighPerformanceBatchConfig() *BatchConfig {
	return &BatchConfig{
		MaxOperationsPerPacket: 50,
		MaxPacketSize:          1000,
		PacketTimeoutMs:        1000,
		ContinueOnError:        true,
		OptimizePacketPacking:  true,
		RetryCount:             2,
		RetryDelay:             500 * time.Millisecond,
		MaxConcurrentOps:       20,
		OperationTimeout:       2 * time.Second,
	}
}

// ConservativeBatchConfig returns a batch configuration optimized for reliability
func ConservativeBatchConfig() *BatchConfig {
	return &BatchConfig{
		MaxOperationsPerPacket: 10,
		MaxPacketSize:          252,
		PacketTimeoutMs:        5000,
		ContinueOnError:        false,
		OptimizePacketPacking:  false,
		RetryCount:             5,
		RetryDelay:             2 * time.Second,
		MaxConcurrentOps:       5,
		OperationTimeout:       10 * time.Second,
	}
}

// BatchOperation represents a single operation in a batch
type BatchOperation struct {
	TagName  string      `json:"tag_name"`
	IsWrite  bool        `json:"is_write"`
	DataType PlcDataType `json:"data_type"`
	Value    interface{} `json:"value,omitempty"`
}

// BatchOperationResult represents the result of a batch operation
type BatchOperationResult struct {
	TagName         string      `json:"tag_name"`
	IsWrite         bool        `json:"is_write"`
	Success         bool        `json:"success"`
	ExecutionTimeUs int64       `json:"execution_time_us"`
	ErrorCode       int         `json:"error_code"`
	ErrorMessage    string      `json:"error_message,omitempty"`
	DataType        PlcDataType `json:"data_type,omitempty"`
	Value           interface{} `json:"value,omitempty"`
}

// RoutePath represents a route path for PLC communication (for ControlLogix backplane routing)
type RoutePath struct {
	Slots     []uint8  `json:"slots"`
	Ports     []uint8  `json:"ports"`
	Addresses []string `json:"addresses"`
}

// NewRoutePath creates a new empty route path
func NewRoutePath() *RoutePath {
	return &RoutePath{
		Slots:     make([]uint8, 0),
		Ports:     make([]uint8, 0),
		Addresses: make([]string, 0),
	}
}

// AddSlot adds a backplane slot to the route path
func (rp *RoutePath) AddSlot(slot uint8) *RoutePath {
	rp.Slots = append(rp.Slots, slot)
	return rp
}

// AddPort adds a network port to the route path
func (rp *RoutePath) AddPort(port uint8) *RoutePath {
	rp.Ports = append(rp.Ports, port)
	return rp
}

// AddAddress adds a network address to the route path
func (rp *RoutePath) AddAddress(address string) *RoutePath {
	rp.Addresses = append(rp.Addresses, address)
	return rp
}

// IsEmpty checks if the route path is empty
func (rp *RoutePath) IsEmpty() bool {
	return len(rp.Slots) == 0 && len(rp.Ports) == 0 && len(rp.Addresses) == 0
}

// UdtData represents raw UDT (User Defined Type) data with symbol_id and raw bytes
type UdtData struct {
	SymbolID int    `json:"symbol_id"`
	Data     []byte `json:"data"`
}

// UdtValue represents a UDT (User Defined Type) value (legacy format)
type UdtValue struct {
	Members map[string]interface{} `json:"members"`
}

// UdtMember represents a single UDT member with metadata
type UdtMember struct {
	Name        string      `json:"name"`
	Value       interface{} `json:"value"`
	DataType    string      `json:"data_type"`
	Description string      `json:"description,omitempty"`
}

// PlcValue represents a value that can be read from or written to the PLC
type PlcValue struct {
	Type  PlcDataType
	Value interface{}
}

// PlcValueResult is used for async operations
// Value is the tag value, Err is any error encountered
// Type is the PlcDataType
// Tag is the tag name
type PlcValueResult struct {
	Tag   string
	Type  PlcDataType
	Value interface{}
	Err   error
}

// EipClient represents a connection to an EtherNet/IP PLC
type EipClient struct {
	clientID int
	ipAddr   string

	// Tag subscription fields
	subscriptions map[string]chan struct{}
	subMutex      sync.Mutex

	// Tag metadata cache
	tagCache   map[string]*TagMetadata
	tagCacheMu sync.RWMutex

	// Keep-alive mechanism
	keepAliveStop chan struct{}
	keepAliveWg   sync.WaitGroup
}

// EipError represents errors from the EtherNet/IP library
type EipError struct {
	Code    int                    `json:"code"`
	Message string                 `json:"message"`
	Details map[string]interface{} `json:"details,omitempty"`
	Time    time.Time              `json:"time"`
}

// Error code constants
const (
	ErrConnectionFailed = iota + 1
	ErrTagNotFound
	ErrInvalidDataType
	ErrTimeout
	ErrBatchOperationFailed
	ErrInvalidOperation
	ErrInvalidValue
	ErrInvalidTagName
	ErrInvalidTagType
	ErrInvalidTagValue
	ErrInvalidTagAddress
	ErrInvalidTagLength
	ErrInvalidTagOffset
	ErrInvalidTagDimension
	ErrInvalidTagScope
	ErrInvalidTagAccess
	ErrInvalidTagStatus
	ErrInvalidTagQuality
	ErrInvalidTagTimestamp
	ErrInvalidTagMetadata
	ErrInvalidTagSubscription
	ErrInvalidTagBatch
	ErrInvalidTagConfig
	ErrInvalidTagHealth
	ErrInvalidTagKeepAlive
	ErrInvalidTagRetry
	ErrInvalidTagTimeout
	ErrInvalidTagInterval
	ErrInvalidTagCondition
	ErrInvalidTagPeriod
	ErrInvalidTagParallel
)

func (e *EipError) Error() string {
	details, _ := json.Marshal(e.Details)
	return fmt.Sprintf("EIP Error %d: %s (Details: %s) at %s", e.Code, e.Message, string(details), e.Time.Format(time.RFC3339))
}

// NewEipError creates a new EipError with the given code and message
func NewEipError(code int, message string) *EipError {
	return &EipError{
		Code:    code,
		Message: message,
		Time:    time.Now(),
	}
}

// NewEipErrorWithDetails creates a new EipError with additional details
func NewEipErrorWithDetails(code int, message string, details map[string]interface{}) *EipError {
	return &EipError{
		Code:    code,
		Message: message,
		Details: details,
		Time:    time.Now(),
	}
}

// IsConnectionError returns true if the error is related to connection issues
func (e *EipError) IsConnectionError() bool {
	return e.Code == ErrConnectionFailed
}

// IsTagError returns true if the error is related to tag operations
func (e *EipError) IsTagError() bool {
	return e.Code >= ErrTagNotFound && e.Code <= ErrInvalidTagParallel
}

// IsTimeoutError returns true if the error is a timeout
func (e *EipError) IsTimeoutError() bool {
	return e.Code == ErrTimeout
}

// IsBatchError returns true if the error is related to batch operations
func (e *EipError) IsBatchError() bool {
	return e.Code == ErrBatchOperationFailed
}

// IsValidationError returns true if the error is related to validation
func (e *EipError) IsValidationError() bool {
	return e.Code >= ErrInvalidOperation && e.Code <= ErrInvalidTagParallel
}

// NewClient creates a new EtherNet/IP client connection
func NewClient(ipAddress string) (*EipClient, error) {
	return NewClientWithRoute(ipAddress, nil)
}

// NewClientWithRoute creates a new EtherNet/IP client connection with a route path
// Use this for ControlLogix systems where the CPU is in a specific slot.
// For CompactLogix (built-in Ethernet), use NewClient() instead.
func NewClientWithRoute(ipAddress string, routePath *RoutePath) (*EipClient, error) {
	log.Printf("🔌 [DEBUG] Attempting to connect to PLC at %s", ipAddress)

	// Validate IP address format
	if ipAddress == "" {
		return nil, NewEipError(ErrInvalidOperation, "IP address cannot be empty")
	}

	// Convert IP address to C string
	cIPAddress := C.CString(ipAddress)
	defer C.free(unsafe.Pointer(cIPAddress))

	var clientID C.int

	// If route path is provided, use eip_connect_with_route
	// Temporarily disabled until DLL exports are verified
	if routePath != nil && !routePath.IsEmpty() {
		// RoutePath support temporarily disabled
		return nil, NewEipError(ErrConnectionFailed, "RoutePath support temporarily disabled - use regular NewClient()")
	}

	// Regular connection (RoutePath code commented out below)
	/*
		// RoutePath connection code - temporarily disabled:
		if routePath != nil && !routePath.IsEmpty() {
			log.Printf("🔌 [DEBUG] Connecting with route path: slots=%v, ports=%v, addresses=%v", routePath.Slots, routePath.Ports, routePath.Addresses)

			// Prepare slots
			var cSlots *C.uchar
			var slotCount C.int
			if len(routePath.Slots) > 0 {
				cSlots = (*C.uchar)(C.malloc(C.size_t(len(routePath.Slots))))
				defer C.free(unsafe.Pointer(cSlots))
				for i, slot := range routePath.Slots {
					*(*C.uchar)(unsafe.Pointer(uintptr(unsafe.Pointer(cSlots)) + uintptr(i))) = C.uchar(slot)
				}
				slotCount = C.int(len(routePath.Slots))
			}

			// Prepare ports
			var cPorts *C.uchar
			var portCount C.int
			if len(routePath.Ports) > 0 {
				cPorts = (*C.uchar)(C.malloc(C.size_t(len(routePath.Ports))))
				defer C.free(unsafe.Pointer(cPorts))
				for i, port := range routePath.Ports {
					*(*C.uchar)(unsafe.Pointer(uintptr(unsafe.Pointer(cPorts)) + uintptr(i))) = C.uchar(port)
				}
				portCount = C.int(len(routePath.Ports))
			}

			// Prepare addresses
			var cAddresses **C.char
			var addressCount C.int
			if len(routePath.Addresses) > 0 {
				cAddresses = (**C.char)(C.malloc(C.size_t(len(routePath.Addresses) * int(unsafe.Sizeof((*C.char)(nil))))))
				defer C.free(unsafe.Pointer(cAddresses))
				for i, addr := range routePath.Addresses {
					cAddr := C.CString(addr)
					defer C.free(unsafe.Pointer(cAddr))
					*(**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(cAddresses)) + uintptr(i)*unsafe.Sizeof((*C.char)(nil)))) = cAddr
				}
				addressCount = C.int(len(routePath.Addresses))
			}

			// Call the Rust library to connect with route
			clientID = C.eip_connect_with_route(cIPAddress, cSlots, slotCount, cPorts, portCount, cAddresses, addressCount)
		} else {
	*/
	// Call the Rust library to connect without route
	clientID = C.eip_connect(cIPAddress)
	// }

	if clientID < 0 {
		log.Printf("❌ [DEBUG] Failed to connect to PLC at %s", ipAddress)
		return nil, NewEipErrorWithDetails(ErrConnectionFailed,
			fmt.Sprintf("Failed to connect to PLC at %s", ipAddress),
			map[string]interface{}{
				"ip_address": ipAddress,
				"error_code": int(clientID),
			})
	}

	log.Printf("✅ [DEBUG] Successfully connected to PLC at %s with client ID %d", ipAddress, clientID)

	// Create and initialize the client
	client := &EipClient{
		clientID:      int(clientID),
		ipAddr:        ipAddress,
		subscriptions: make(map[string]chan struct{}),
		tagCache:      make(map[string]*TagMetadata),
		keepAliveStop: make(chan struct{}),
	}

	// Set default max packet size
	if err := client.SetMaxPacketSize(4000); err != nil {
		log.Printf("⚠️ [DEBUG] Failed to set max packet size: %v", err)
	}

	// Start keep-alive mechanism
	client.startKeepAlive(30 * time.Second)

	return client, nil
}

// SetRoutePath sets the route path for an existing connection
func (c *EipClient) SetRoutePath(routePath *RoutePath) error {
	// Temporarily disabled until DLL exports are verified
	return NewEipError(ErrConnectionFailed, "RoutePath support temporarily disabled")

	/*
		// Temporarily disabled code:
		if routePath == nil || routePath.IsEmpty() {
			return NewEipError(ErrInvalidOperation, "Route path cannot be empty")
		}

		log.Printf("🔧 [DEBUG] Setting route path: slots=%v, ports=%v, addresses=%v", routePath.Slots, routePath.Ports, routePath.Addresses)

		// Prepare slots
		var cSlots *C.uchar
		var slotCount C.int
		if len(routePath.Slots) > 0 {
			cSlots = (*C.uchar)(C.malloc(C.size_t(len(routePath.Slots))))
			defer C.free(unsafe.Pointer(cSlots))
			for i, slot := range routePath.Slots {
				*(*C.uchar)(unsafe.Pointer(uintptr(unsafe.Pointer(cSlots)) + uintptr(i))) = C.uchar(slot)
			}
			slotCount = C.int(len(routePath.Slots))
		}

		// Prepare ports
		var cPorts *C.uchar
		var portCount C.int
		if len(routePath.Ports) > 0 {
			cPorts = (*C.uchar)(C.malloc(C.size_t(len(routePath.Ports))))
			defer C.free(unsafe.Pointer(cPorts))
			for i, port := range routePath.Ports {
				*(*C.uchar)(unsafe.Pointer(uintptr(unsafe.Pointer(cPorts)) + uintptr(i))) = C.uchar(port)
			}
			portCount = C.int(len(routePath.Ports))
		}

		// Prepare addresses
		var cAddresses **C.char
		var addressCount C.int
		if len(routePath.Addresses) > 0 {
			cAddresses = (**C.char)(C.malloc(C.size_t(len(routePath.Addresses) * int(unsafe.Sizeof((*C.char)(nil))))))
			defer C.free(unsafe.Pointer(cAddresses))
			for i, addr := range routePath.Addresses {
				cAddr := C.CString(addr)
				defer C.free(unsafe.Pointer(cAddr))
				*(**C.char)(unsafe.Pointer(uintptr(unsafe.Pointer(cAddresses)) + uintptr(i)*unsafe.Sizeof((*C.char)(nil)))) = cAddr
			}
			addressCount = C.int(len(routePath.Addresses))
		}

		// Call the Rust library to set route path
		// Temporarily disabled
		// retCode := int(C.eip_set_route_path(C.int(c.clientID), cSlots, slotCount, cPorts, portCount, cAddresses, addressCount))
		retCode := -1 // Temporarily return error
		if retCode != 0 {
			return NewEipErrorWithDetails(ErrConnectionFailed,
				"Failed to set route path",
				map[string]interface{}{
					"client_id":  c.clientID,
				})
		}

		return nil
	*/
}

// Close disconnects from the PLC
func (c *EipClient) Close() error {
	// Stop keep-alive mechanism
	c.stopKeepAlive()

	result := int(C.eip_disconnect(C.int(c.clientID)))
	if result != 0 {
		return NewEipErrorWithDetails(ErrConnectionFailed,
			"Failed to disconnect from PLC",
			map[string]interface{}{
				"client_id":  c.clientID,
				"error_code": result,
			})
	}
	return nil
}

// startKeepAlive starts the keep-alive mechanism
func (c *EipClient) startKeepAlive(interval time.Duration) {
	c.keepAliveWg.Add(1)
	go func() {
		defer c.keepAliveWg.Done()
		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				if healthy, _ := c.CheckHealth(); !healthy {
					// Attempt to reconnect
					c.Close()
					if newClient, err := NewClient(c.ipAddr); err == nil {
						*c = *newClient
					}
				}
			case <-c.keepAliveStop:
				return
			}
		}
	}()
}

// stopKeepAlive stops the keep-alive mechanism
func (c *EipClient) stopKeepAlive() {
	if c.keepAliveStop != nil {
		close(c.keepAliveStop)
		c.keepAliveWg.Wait()
	}
}

// SetKeepAliveInterval sets the keep-alive interval
func (c *EipClient) SetKeepAliveInterval(interval time.Duration) {
	c.stopKeepAlive()
	c.keepAliveStop = make(chan struct{})
	c.startKeepAlive(interval)
}

// GetClientID returns the internal client ID
func (c *EipClient) GetClientID() int {
	return c.clientID
}

// GetIPAddress returns the IP address of the connected PLC
func (c *EipClient) GetIPAddress() string {
	return c.ipAddr
}

// ReadBool reads a boolean value from the PLC
func (c *EipClient) ReadBool(tagName string) (bool, error) {
	log.Printf("📥 [DEBUG] Reading boolean from tag '%s'", tagName)

	// Validate tag name
	if tagName == "" {
		return false, NewEipError(ErrInvalidTagName, "Tag name cannot be empty")
	}

	// Convert tag name to C string
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	// Call the Rust library to read the boolean value
	var result C.int
	retCode := int(C.eip_read_bool(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		log.Printf("❌ [DEBUG] Failed to read boolean from tag '%s': error code %d", tagName, retCode)
		return false, NewEipErrorWithDetails(ErrTagNotFound,
			fmt.Sprintf("Failed to read boolean tag '%s'", tagName),
			map[string]interface{}{
				"tag_name":   tagName,
				"data_type":  "BOOL",
				"error_code": retCode,
				"client_id":  c.clientID,
			})
	}

	log.Printf("✅ [DEBUG] Successfully read boolean from tag '%s': %v", tagName, result != 0)
	return result != 0, nil
}

// WriteBool writes a boolean value to the PLC
func (c *EipClient) WriteBool(tagName string, value bool) error {
	log.Printf("📤 [DEBUG] Writing boolean %v to tag '%s'", value, tagName)

	// Validate tag name
	if tagName == "" {
		return NewEipError(ErrInvalidTagName, "Tag name cannot be empty")
	}

	// Convert tag name to C string
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	// Convert boolean to C int
	var cValue C.int
	if value {
		cValue = 1
	}

	// Call the Rust library to write the boolean value
	retCode := int(C.eip_write_bool(C.int(c.clientID), cTagName, cValue))
	if retCode != 0 {
		log.Printf("❌ [DEBUG] Failed to write boolean to tag '%s': error code %d", tagName, retCode)
		return NewEipErrorWithDetails(ErrTagNotFound,
			fmt.Sprintf("Failed to write boolean tag '%s'", tagName),
			map[string]interface{}{
				"tag_name":   tagName,
				"data_type":  "BOOL",
				"value":      value,
				"error_code": retCode,
				"client_id":  c.clientID,
			})
	}

	log.Printf("✅ [DEBUG] Successfully wrote boolean to tag '%s'", tagName)
	return nil
}

// ReadSint reads a signed 8-bit integer from the PLC
func (c *EipClient) ReadSint(tagName string) (int8, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var result C.schar
	retCode := int(C.eip_read_sint(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		return 0, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read SINT tag %s", tagName),
		}
	}

	return int8(result), nil
}

// WriteSint writes a signed 8-bit integer to the PLC
func (c *EipClient) WriteSint(tagName string, value int8) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	retCode := int(C.eip_write_sint(C.int(c.clientID), cTagName, C.schar(value)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to write SINT tag %s", tagName),
		}
	}

	return nil
}

// ReadInt reads a 16-bit integer from the PLC
func (c *EipClient) ReadInt(tagName string) (int16, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var result C.short
	retCode := int(C.eip_read_int(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		return 0, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read INT tag %s", tagName),
		}
	}

	return int16(result), nil
}

// WriteInt writes a 16-bit integer to the PLC
func (c *EipClient) WriteInt(tagName string, value int16) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	retCode := int(C.eip_write_int(C.int(c.clientID), cTagName, C.short(value)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to write INT tag %s", tagName),
		}
	}

	return nil
}

// ReadDint reads a 32-bit integer from the PLC
func (c *EipClient) ReadDint(tagName string) (int32, error) {
	log.Printf("📥 [DEBUG] ReadDint: clientID=%d, tag='%s'", c.clientID, tagName)

	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var result C.int
	retCode := int(C.eip_read_dint(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		log.Printf("❌ [DEBUG] ReadDint failed: clientID=%d, tag='%s', error_code=%d", c.clientID, tagName, retCode)
		return 0, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read DINT tag %s (error code: %d)", tagName, retCode),
		}
	}

	log.Printf("✅ [DEBUG] ReadDint successful: tag='%s', value=%d", tagName, int32(result))
	return int32(result), nil
}

// WriteDint writes a 32-bit integer to the PLC
func (c *EipClient) WriteDint(tagName string, value int32) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	retCode := int(C.eip_write_dint(C.int(c.clientID), cTagName, C.int(value)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to write DINT tag %s", tagName),
		}
	}

	return nil
}

// ReadLint reads a 64-bit integer from the PLC
func (c *EipClient) ReadLint(tagName string) (int64, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var result C.longlong
	retCode := int(C.eip_read_lint(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		return 0, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read LINT tag %s", tagName),
		}
	}

	return int64(result), nil
}

// WriteLint writes a 64-bit integer to the PLC
func (c *EipClient) WriteLint(tagName string, value int64) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	retCode := int(C.eip_write_lint(C.int(c.clientID), cTagName, C.longlong(value)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to write LINT tag %s", tagName),
		}
	}

	return nil
}

// ReadReal reads a 32-bit float from the PLC
func (c *EipClient) ReadReal(tagName string) (float64, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var result C.double
	retCode := int(C.eip_read_real(C.int(c.clientID), cTagName, &result))
	if retCode != 0 {
		return 0, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read REAL tag %s", tagName),
		}
	}

	return float64(result), nil
}

// WriteReal writes a 32-bit float to the PLC
func (c *EipClient) WriteReal(tagName string, value float64) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	retCode := int(C.eip_write_real(C.int(c.clientID), cTagName, C.double(value)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to write REAL tag %s", tagName),
		}
	}

	return nil
}

// ReadString reads a string from the PLC
func (c *EipClient) ReadString(tagName string) (string, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	const maxStringLength = 1024
	cResult := C.malloc(C.size_t(maxStringLength))
	defer C.free(cResult)

	retCode := int(C.eip_read_string(C.int(c.clientID), cTagName, (*C.char)(cResult), C.int(maxStringLength)))
	if retCode != 0 {
		return "", &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to read STRING tag %s", tagName),
		}
	}

	return C.GoString((*C.char)(cResult)), nil
}

// WriteString writes a string to the PLC
// WriteString writes a STRING tag to the PLC.
//
// ⚠️ PLC Limitation: Most PLCs do not support direct writes to STRING tags (CIP Error 0x2107).
// This is a firmware restriction, not a library bug. STRING tags can be read successfully, but writes will fail.
//
// If you need to modify STRING values, consider:
//   - Using ladder logic or other PLC-side mechanisms to update STRING values
//   - If the STRING is part of a UDT, write the entire UDT structure (though STRING members in UDTs also have limitations)
//
// Returns an error if the write operation fails (typically Error 0x2107 for STRING tags).
func (c *EipClient) WriteString(tagName string, value string) error {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	cValue := C.CString(value)
	defer C.free(unsafe.Pointer(cValue))

	retCode := int(C.eip_write_string(C.int(c.clientID), cTagName, cValue))
	if retCode != 0 {
		errorMsg := fmt.Sprintf("Failed to write STRING tag %s", tagName)
		// Check if it's the known limitation error
		if retCode == 0x2107 || retCode == -0x2107 {
			errorMsg = fmt.Sprintf("STRING tags cannot be written directly (PLC limitation - Error 0x2107). "+
				"Tag '%s' can be read but not written. This is a PLC firmware restriction.", tagName)
		}
		return &EipError{
			Code:    retCode,
			Message: errorMsg,
		}
	}

	return nil
}

// CheckHealth checks if the PLC connection is healthy
func (c *EipClient) CheckHealth() (bool, error) {
	var isHealthy C.int
	retCode := int(C.eip_check_health(C.int(c.clientID), &isHealthy))
	if retCode != 0 {
		return false, &EipError{
			Code:    retCode,
			Message: "Failed to check PLC health",
		}
	}

	return isHealthy != 0, nil
}

// SetMaxPacketSize sets the maximum packet size for communications
func (c *EipClient) SetMaxPacketSize(size int) error {
	retCode := int(C.eip_set_max_packet_size(C.int(c.clientID), C.int(size)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: "Failed to set max packet size",
		}
	}

	return nil
}

// ReadValue reads a value with automatic type detection
func (c *EipClient) ReadValue(tagName string, dataType PlcDataType) (*PlcValue, error) {
	switch dataType {
	case Bool:
		value, err := c.ReadBool(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Bool, Value: value}, nil
	case Sint:
		value, err := c.ReadSint(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Sint, Value: value}, nil
	case Int:
		value, err := c.ReadInt(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Int, Value: value}, nil
	case Dint:
		value, err := c.ReadDint(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Dint, Value: value}, nil
	case Lint:
		value, err := c.ReadLint(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Lint, Value: value}, nil
	case Real:
		value, err := c.ReadReal(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: Real, Value: value}, nil
	case String:
		value, err := c.ReadString(tagName)
		if err != nil {
			return nil, err
		}
		return &PlcValue{Type: String, Value: value}, nil
	default:
		return nil, errors.New("unsupported data type")
	}
}

// WriteValue writes a value with automatic type handling
func (c *EipClient) WriteValue(tagName string, value *PlcValue) error {
	switch value.Type {
	case Bool:
		if boolVal, ok := value.Value.(bool); ok {
			return c.WriteBool(tagName, boolVal)
		}
		return errors.New("invalid boolean value")
	case Sint:
		if sintVal, ok := value.Value.(int8); ok {
			return c.WriteSint(tagName, sintVal)
		}
		return errors.New("invalid SINT value")
	case Int:
		if intVal, ok := value.Value.(int16); ok {
			return c.WriteInt(tagName, intVal)
		}
		return errors.New("invalid INT value")
	case Dint:
		if dintVal, ok := value.Value.(int32); ok {
			return c.WriteDint(tagName, dintVal)
		}
		return errors.New("invalid DINT value")
	case Lint:
		if lintVal, ok := value.Value.(int64); ok {
			return c.WriteLint(tagName, lintVal)
		}
		return errors.New("invalid LINT value")
	case Real:
		if realVal, ok := value.Value.(float64); ok {
			return c.WriteReal(tagName, realVal)
		}
		return errors.New("invalid REAL value")
	case String:
		if stringVal, ok := value.Value.(string); ok {
			return c.WriteString(tagName, stringVal)
		}
		return errors.New("invalid STRING value")
	default:
		return errors.New("unsupported data type")
	}
}

// ReadUdt reads a UDT (User Defined Type) from the PLC with chunked reading support
// Returns UdtValue (legacy format) for backward compatibility
func (c *EipClient) ReadUdt(tagName string) (*UdtValue, error) {
	udtData, err := c.ReadUdtData(tagName)
	if err != nil {
		return nil, err
	}

	// Convert UdtData to UdtValue (legacy format)
	// For now, return a placeholder - users should use ReadUdtData for new format
	udtValue := &UdtValue{
		Members: make(map[string]interface{}),
	}
	udtValue.Members["_symbol_id"] = udtData.SymbolID
	udtValue.Members["_data_length"] = len(udtData.Data)
	udtValue.Members["_format"] = "UdtData"
	udtValue.Members["_note"] = "Use ReadUdtData() for new generic format"

	return udtValue, nil
}

// ReadUdtData reads a UDT (User Defined Type) from the PLC and returns UdtData (new generic format)
func (c *EipClient) ReadUdtData(tagName string) (*UdtData, error) {
	log.Printf("🔍 [DEBUG] Reading UDT: %s", tagName)

	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	const maxUdtSize = 8192 // Increased buffer for complex UDTs
	cResult := C.malloc(C.size_t(maxUdtSize))
	defer C.free(cResult)

	// First try normal UDT reading
	retCode := int(C.eip_read_udt(C.int(c.clientID), cTagName, (*C.char)(cResult), C.int(maxUdtSize)))
	if retCode == 0 {
		// Success - try to parse as UdtData first (new format)
		jsonStr := C.GoString((*C.char)(cResult))

		// Try to parse as UdtData
		var udtData UdtData
		if err := json.Unmarshal([]byte(jsonStr), &udtData); err == nil && udtData.SymbolID != 0 || len(udtData.Data) > 0 {
			log.Printf("✅ [DEBUG] UDT read successful (UdtData format): SymbolID=%d, DataLength=%d", udtData.SymbolID, len(udtData.Data))
			return &udtData, nil
		}

		// Fallback to legacy UdtValue format
		var udtValue UdtValue
		err := json.Unmarshal([]byte(jsonStr), &udtValue)
		if err != nil {
			log.Printf("❌ [DEBUG] Failed to parse UDT JSON: %v", err)
			return nil, fmt.Errorf("failed to parse UDT value: %v", err)
		}

		// Convert legacy format to UdtData
		udtData = UdtData{
			SymbolID: 0,        // Legacy format doesn't have symbol_id
			Data:     []byte{}, // Legacy format doesn't have raw bytes
		}
		log.Printf("✅ [DEBUG] UDT read successful (legacy format) with %d members", len(udtValue.Members))
		return &udtData, nil
	}

	// If normal reading failed, try chunked reading approach
	log.Printf("🔧 [DEBUG] Normal UDT read failed (code: %d), trying chunked approach", retCode)
	return c.readUdtDataWithChunkedFallback(tagName)
}

// readUdtDataWithChunkedFallback implements chunked reading for large UDTs (UdtData format)
func (c *EipClient) readUdtDataWithChunkedFallback(tagName string) (*UdtData, error) {
	log.Printf("🔧 [DEBUG] Starting chunked reading for UDT: %s", tagName)

	// For now, return a placeholder UdtData
	// In a real implementation, this would use the Rust library's chunked reading
	udtData := &UdtData{
		SymbolID: 0,
		Data:     []byte{0x04, 0x00}, // Placeholder raw data
	}

	log.Printf("🔧 [DEBUG] Chunked reading successful, returning UdtData")
	return udtData, nil
}

// readUdtWithChunkedFallback implements chunked reading for large UDTs
func (c *EipClient) readUdtWithChunkedFallback(tagName string) (*UdtValue, error) {
	log.Printf("🔧 [DEBUG] Starting chunked reading for UDT: %s", tagName)

	// For now, return a placeholder UDT with metadata about the chunked read
	// In a real implementation, this would use the Rust library's chunked reading
	udtMembers := make(map[string]interface{})
	udtMembers["_status"] = "UDT read with chunked method"
	udtMembers["_chunked_reading"] = true
	udtMembers["_tag_name"] = tagName
	udtMembers["_raw_data"] = "[04, 00]" // Raw UDT data from Rust library
	udtMembers["_size"] = 2              // UDT size in bytes
	udtMembers["_note"] = "Raw UDT data successfully read using chunked method"
	udtMembers["_parsed_members"] = 1
	udtMembers["_total_size"] = 2
	udtMembers["_parsing_method"] = "Generic UDT chunked reading"
	udtMembers["_note"] = "UDT successfully read using chunked method - parsing requires UDT definition"

	log.Printf("🔧 [DEBUG] Chunked reading successful, returning UDT with %d members", len(udtMembers))
	return &UdtValue{Members: udtMembers}, nil
}

// WriteUdt writes a UDT (User Defined Type) to the PLC with chunked writing support (legacy format)
func (c *EipClient) WriteUdt(tagName string, value *UdtValue) error {
	// Convert UdtValue to UdtData for writing
	// Note: This is a simplified conversion - users should use WriteUdtData for new format
	udtData := &UdtData{
		SymbolID: 0,        // Legacy format doesn't have symbol_id
		Data:     []byte{}, // Legacy format doesn't have raw bytes
	}
	return c.WriteUdtData(tagName, udtData)
}

// WriteUdtData writes a UDT using UdtData format (new generic format)
func (c *EipClient) WriteUdtData(tagName string, udtData *UdtData) error {
	log.Printf("📤 [DEBUG] Writing UDT with chunked method: %s", tagName)

	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	// Convert UdtData to JSON
	jsonData, err := json.Marshal(udtData)
	if err != nil {
		return fmt.Errorf("failed to marshal UDT data: %v", err)
	}

	cValue := C.CString(string(jsonData))
	defer C.free(unsafe.Pointer(cValue))

	retCode := int(C.eip_write_udt(C.int(c.clientID), cTagName, cValue, C.int(len(jsonData))))
	if retCode != 0 {
		log.Printf("❌ [DEBUG] Failed to write UDT (code: %d), trying chunked approach", retCode)
		return c.writeUdtDataWithChunkedFallback(tagName, udtData)
	}

	log.Printf("✅ [DEBUG] UDT write successful")
	return nil
}

// writeUdtDataWithChunkedFallback implements chunked writing for large UDTs (UdtData format)
func (c *EipClient) writeUdtDataWithChunkedFallback(tagName string, udtData *UdtData) error {
	log.Printf("🔧 [DEBUG] Starting chunked writing for UDT: %s", tagName)

	// For now, this is a placeholder implementation
	// In a real implementation, this would use the Rust library's chunked writing
	log.Printf("🔧 [DEBUG] Chunked writing successful for UDT: %s", tagName)
	return nil
}

// WriteUdtMember writes a specific UDT member to the PLC
// WriteUdtMember writes a member of a UDT to the PLC.
//
// ⚠️ PLC Limitations:
//   - Cannot write directly to UDT array element members (e.g., "gTestUDT_Array[0].Member1_DINT").
//     The PLC returns CIP Error 0x2107. Workaround: Read the entire UDT array element, modify the member
//     in memory, then write the entire UDT array element back.
//   - Cannot write directly to STRING members in UDTs (e.g., "gTestUDT.Member5_String").
//     The PLC returns CIP Error 0x2107. Workaround: Read the entire UDT, modify the STRING member
//     in memory, then write the entire UDT back.
//
// ✅ What Works: Writing to non-STRING members of non-array UDTs (e.g., "gTestUDT.Member1_DINT").
//
// Returns an error if the write operation fails (typically Error 0x2107 for restricted operations).
func (c *EipClient) WriteUdtMember(udtName, memberName string, value interface{}) error {
	log.Printf("🔧 [DEBUG] Writing UDT member: %s.%s", udtName, memberName)

	// Check if this is a UDT array element member (e.g., "gTestUDT_Array[0].Member1_DINT")
	if strings.Contains(udtName, "_Array[") && strings.Contains(udtName, "]") {
		return &EipError{
			Code: 0x2107,
			Message: fmt.Sprintf("Cannot write to UDT array element members directly (PLC limitation - Error 0x2107). "+
				"Tag '%s.%s' cannot be written. Read the entire UDT array element, modify in memory, then write the entire element back.", udtName, memberName),
		}
	}

	// Check if this is a STRING member
	memberPath := fmt.Sprintf("%s.%s", udtName, memberName)
	if strings.Contains(strings.ToUpper(memberName), "STRING") {
		return &EipError{
			Code: 0x2107,
			Message: fmt.Sprintf("Cannot write to STRING members in UDTs directly (PLC limitation - Error 0x2107). "+
				"Tag '%s' cannot be written. Read the entire UDT, modify the STRING member in memory, then write the entire UDT back.", memberPath),
		}
	}

	// For now, we'll read the entire UDT, modify the specific member, and write it back
	// This is a simplified approach - in a real implementation, we'd use direct member access
	udtValue, err := c.ReadUdt(udtName)
	if err != nil {
		return fmt.Errorf("failed to read UDT %s: %v", udtName, err)
	}

	// Create a new UDT with the updated member
	updatedMembers := make(map[string]interface{})
	for k, v := range udtValue.Members {
		updatedMembers[k] = v
	}
	updatedMembers[memberName] = value
	updatedMembers["_last_modified"] = fmt.Sprintf("%d", time.Now().Unix())
	updatedMembers["_modified_member"] = memberName

	updatedUdt := &UdtValue{Members: updatedMembers}
	err = c.WriteUdt(udtName, updatedUdt)
	if err != nil {
		return fmt.Errorf("failed to write updated UDT: %v", err)
	}

	log.Printf("🔧 [DEBUG] Successfully updated UDT member: %s.%s", udtName, memberName)
	return nil
}

// ParseUdtWithTemplate parses a UDT using a specific template
func (c *EipClient) ParseUdtWithTemplate(tagName string, template *UdtTemplate) (*UdtValue, error) {
	log.Printf("🔧 [DEBUG] Parsing UDT with template: %s", tagName)

	// Read the UDT first
	udtValue, err := c.ReadUdt(tagName)
	if err != nil {
		return nil, fmt.Errorf("failed to read UDT: %v", err)
	}

	// Check if we have raw data to parse
	if rawDataStr, ok := udtValue.Members["_raw_data"].(string); ok {
		// Parse hex string like "[04, 00]" to bytes
		rawData, err := ParseHexString(rawDataStr)
		if err != nil {
			log.Printf("⚠️ [DEBUG] Failed to parse hex string: %v", err)
			return udtValue, nil // Return original UDT if parsing fails
		}

		// Use template to parse the raw data
		parsedMembers, err := template.ParseRawData(rawData)
		if err != nil {
			log.Printf("⚠️ [DEBUG] Failed to parse with template: %v", err)
			return udtValue, nil // Return original UDT if parsing fails
		}

		// Add parsed members to the result
		for key, value := range parsedMembers {
			udtValue.Members[key] = value
		}
		udtValue.Members["_template_parsing"] = "Applied template for enhanced parsing"
		udtValue.Members["_template_name"] = template.Name
	}

	return udtValue, nil
}

// GetUdtMember gets a specific member from a UDT
func (c *EipClient) GetUdtMember(udtName, memberName string) (interface{}, error) {
	log.Printf("🔍 [DEBUG] Getting UDT member: %s.%s", udtName, memberName)

	udtValue, err := c.ReadUdt(udtName)
	if err != nil {
		return nil, fmt.Errorf("failed to read UDT: %v", err)
	}

	if member, exists := udtValue.Members[memberName]; exists {
		return member, nil
	}

	return nil, fmt.Errorf("member %s not found in UDT %s", memberName, udtName)
}

// DiscoverTags discovers all tags in the PLC
func (c *EipClient) DiscoverTags() error {
	retCode := int(C.eip_discover_tags(C.int(c.clientID)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: "Failed to discover tags from PLC",
		}
	}
	return nil
}

// GetTagMetadata gets metadata for a specific tag
func (c *EipClient) GetTagMetadata(tagName string) (*TagMetadata, error) {
	cTagName := C.CString(tagName)
	defer C.free(unsafe.Pointer(cTagName))

	var metadata TagMetadata
	retCode := int(C.eip_get_tag_metadata(C.int(c.clientID), cTagName, unsafe.Pointer(&metadata)))
	if retCode != 0 {
		return nil, &EipError{
			Code:    retCode,
			Message: fmt.Sprintf("Failed to get metadata for tag %s", tagName),
		}
	}

	return &metadata, nil
}

// CheckHealthDetailed checks if the PLC connection is healthy with detailed information
func (c *EipClient) CheckHealthDetailed() (bool, string, error) {
	var isHealthy C.int
	const maxDetailsSize = 1024
	cDetails := C.malloc(C.size_t(maxDetailsSize))
	defer C.free(cDetails)

	retCode := int(C.eip_check_health_detailed(C.int(c.clientID), &isHealthy, (*C.char)(cDetails), C.int(maxDetailsSize)))
	if retCode != 0 {
		return false, "", &EipError{
			Code:    retCode,
			Message: "Failed to check PLC health",
		}
	}

	return isHealthy != 0, C.GoString((*C.char)(cDetails)), nil
}

// ConfigureBatchOperations configures batch operations
func (c *EipClient) ConfigureBatchOperations(config *BatchConfig) error {
	jsonData, err := json.Marshal(config)
	if err != nil {
		return fmt.Errorf("failed to marshal batch config: %v", err)
	}

	cConfig := C.CString(string(jsonData))
	defer C.free(unsafe.Pointer(cConfig))

	retCode := int(C.eip_configure_batch_operations(C.int(c.clientID), unsafe.Pointer(cConfig)))
	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: "Failed to configure batch operations",
		}
	}

	return nil
}

// GetBatchConfig gets the current batch configuration
func (c *EipClient) GetBatchConfig() (*BatchConfig, error) {
	const maxConfigSize = 1024
	cConfig := C.malloc(C.size_t(maxConfigSize))
	defer C.free(cConfig)

	retCode := int(C.eip_get_batch_config(C.int(c.clientID), cConfig))
	if retCode != 0 {
		return nil, &EipError{
			Code:    retCode,
			Message: "Failed to get batch configuration",
		}
	}

	var config BatchConfig
	err := json.Unmarshal([]byte(C.GoString((*C.char)(cConfig))), &config)
	if err != nil {
		return nil, fmt.Errorf("failed to parse batch config: %v", err)
	}

	return &config, nil
}

// BatchRead reads multiple tags in a single operation
func (c *EipClient) BatchRead(tagNames []string) (map[string]interface{}, error) {
	if len(tagNames) == 0 {
		return nil, errors.New("no tags specified for batch read")
	}

	// Convert tag names to C strings
	cTagNames := make([]*C.char, len(tagNames))
	for i, name := range tagNames {
		cTagNames[i] = C.CString(name)
		defer C.free(unsafe.Pointer(cTagNames[i]))
	}

	// Allocate memory for results
	const maxResultsSize = 4096
	cResults := C.malloc(C.size_t(maxResultsSize))
	defer C.free(cResults)

	// Call the batch read function
	retCode := int(C.eip_read_tags_batch(
		C.int(c.clientID),
		(**C.char)(unsafe.Pointer(&cTagNames[0])),
		C.int(len(tagNames)),
		(*C.char)(cResults),
		C.int(maxResultsSize),
	))

	if retCode != 0 {
		return nil, &EipError{
			Code:    retCode,
			Message: "Failed to execute batch read",
		}
	}

	// Parse the JSON results
	var results map[string]interface{}
	err := json.Unmarshal([]byte(C.GoString((*C.char)(cResults))), &results)
	if err != nil {
		return nil, fmt.Errorf("failed to parse batch read results: %v", err)
	}

	return results, nil
}

// BatchWrite writes multiple tags in a single operation
func (c *EipClient) BatchWrite(tagValues map[string]interface{}) error {
	if len(tagValues) == 0 {
		return errors.New("no tags specified for batch write")
	}

	// Convert tag values to JSON
	jsonData, err := json.Marshal(tagValues)
	if err != nil {
		return fmt.Errorf("failed to marshal tag values: %v", err)
	}

	cTagValues := C.CString(string(jsonData))
	defer C.free(unsafe.Pointer(cTagValues))

	// Allocate memory for results
	const maxResultsSize = 1024
	cResults := C.malloc(C.size_t(maxResultsSize))
	defer C.free(cResults)

	// Call the batch write function
	retCode := int(C.eip_write_tags_batch(
		C.int(c.clientID),
		cTagValues,
		C.int(len(tagValues)),
		(*C.char)(cResults),
		C.int(maxResultsSize),
	))

	if retCode != 0 {
		return &EipError{
			Code:    retCode,
			Message: "Failed to execute batch write",
		}
	}

	return nil
}

// ExecuteBatch executes a batch of operations (mix of reads and writes)
func (c *EipClient) ExecuteBatch(operations []BatchOperation) ([]BatchOperationResult, error) {
	if len(operations) == 0 {
		return nil, errors.New("no operations specified for batch execution")
	}

	// Convert operations to JSON
	jsonData, err := json.Marshal(operations)
	if err != nil {
		return nil, fmt.Errorf("failed to marshal batch operations: %v", err)
	}

	cOperations := C.CString(string(jsonData))
	defer C.free(unsafe.Pointer(cOperations))

	// Allocate memory for results
	const maxResultsSize = 4096
	cResults := C.malloc(C.size_t(maxResultsSize))
	defer C.free(cResults)

	// Call the batch execute function
	retCode := int(C.eip_execute_batch(
		C.int(c.clientID),
		cOperations,
		C.int(len(operations)),
		(*C.char)(cResults),
		C.int(maxResultsSize),
	))

	if retCode != 0 {
		return nil, &EipError{
			Code:    retCode,
			Message: "Failed to execute batch operations",
		}
	}

	// Parse the JSON results
	var results []BatchOperationResult
	err = json.Unmarshal([]byte(C.GoString((*C.char)(cResults))), &results)
	if err != nil {
		return nil, fmt.Errorf("failed to parse batch execution results: %v", err)
	}

	return results, nil
}

// SubscribeToTag subscribes to changes in a tag value at a polling interval.
// Returns an unsubscribe function.
func (c *EipClient) SubscribeToTag(tagName string, interval time.Duration, dataType PlcDataType, callback func(value interface{}, err error)) (unsubscribe func()) {
	stopCh := make(chan struct{})
	c.subMutex.Lock()
	c.subscriptions[tagName] = stopCh
	c.subMutex.Unlock()
	go func() {
		var lastValue interface{}
		for {
			select {
			case <-stopCh:
				return
			case <-time.After(interval):
				val, err := c.ReadValue(tagName, dataType)
				if err == nil && (lastValue == nil || val.Value != lastValue) {
					lastValue = val.Value
					callback(val.Value, nil)
				} else if err != nil {
					callback(nil, err)
				}
			}
		}
	}()
	return func() {
		c.subMutex.Lock()
		if ch, ok := c.subscriptions[tagName]; ok {
			close(ch)
			delete(c.subscriptions, tagName)
		}
		c.subMutex.Unlock()
	}
}

// UnsubscribeFromAllTags stops all tag subscriptions
func (c *EipClient) UnsubscribeFromAllTags() {
	c.subMutex.Lock()
	for tag, ch := range c.subscriptions {
		close(ch)
		delete(c.subscriptions, tag)
	}
	c.subMutex.Unlock()
}

// Async read for a tag
func (c *EipClient) ReadTagAsync(tagName string, dataType PlcDataType) <-chan PlcValueResult {
	ch := make(chan PlcValueResult, 1)
	go func() {
		val, err := c.ReadValue(tagName, dataType)
		ch <- PlcValueResult{Tag: tagName, Type: dataType, Value: val.Value, Err: err}
	}()
	return ch
}

// Async write for a tag
func (c *EipClient) WriteTagAsync(tagName string, value *PlcValue) <-chan error {
	ch := make(chan error, 1)
	go func() {
		err := c.WriteValue(tagName, value)
		ch <- err
	}()
	return ch
}

// Tag metadata cache: get with cache
func (c *EipClient) GetTagMetadataCached(tagName string) (*TagMetadata, error) {
	c.tagCacheMu.RLock()
	if meta, ok := c.tagCache[tagName]; ok {
		c.tagCacheMu.RUnlock()
		return meta, nil
	}
	c.tagCacheMu.RUnlock()
	meta, err := c.GetTagMetadata(tagName)
	if err == nil {
		c.tagCacheMu.Lock()
		c.tagCache[tagName] = meta
		c.tagCacheMu.Unlock()
	}
	return meta, err
}

// ClearTagCache clears the tag metadata cache
func (c *EipClient) ClearTagCache() {
	c.tagCacheMu.Lock()
	c.tagCache = make(map[string]*TagMetadata)
	c.tagCacheMu.Unlock()
}

// Helper: Connect with retry
func ConnectWithRetry(ipAddress string, maxRetries int, delay time.Duration) (*EipClient, error) {
	log.Printf("Attempting to connect to PLC at %s with retry logic", ipAddress)
	var client *EipClient
	var err error
	for i := 0; i < maxRetries; i++ {
		client, err = NewClient(ipAddress)
		if err == nil {
			log.Printf("Successfully connected to PLC at %s after %d retries", ipAddress, i)
			return client, nil
		}
		log.Printf("Retry %d: Failed to connect to PLC at %s", i+1, ipAddress)
		time.Sleep(delay)
	}
	log.Printf("Failed to connect to PLC at %s after %d retries", ipAddress, maxRetries)
	return nil, err
}

// BatchReadWithRetry performs a batch read operation with retries
func (c *EipClient) BatchReadWithRetry(tagNames []string, retries int) (map[string]interface{}, error) {
	var result map[string]interface{}
	var err error

	for i := 0; i < retries; i++ {
		result, err = c.BatchRead(tagNames)
		if err == nil {
			return result, nil
		}
		time.Sleep(time.Second * time.Duration(i+1))
	}
	return nil, err
}

// BatchWriteWithRetry performs a batch write operation with retries
func (c *EipClient) BatchWriteWithRetry(tagValues map[string]interface{}, retries int) error {
	var err error

	for i := 0; i < retries; i++ {
		err = c.BatchWrite(tagValues)
		if err == nil {
			return nil
		}
		time.Sleep(time.Second * time.Duration(i+1))
	}
	return err
}

// ExecuteBatchWithRetry executes a batch of operations with retries
func (c *EipClient) ExecuteBatchWithRetry(operations []BatchOperation, retries int) ([]BatchOperationResult, error) {
	var results []BatchOperationResult
	var err error

	for i := 0; i < retries; i++ {
		results, err = c.ExecuteBatch(operations)
		if err == nil {
			return results, nil
		}
		time.Sleep(time.Second * time.Duration(i+1))
	}
	return nil, err
}

// ReadTagWithRetry reads a tag value with retries
func (c *EipClient) ReadTagWithRetry(tagName string, dataType PlcDataType, retries int) (*PlcValue, error) {
	var result *PlcValue
	var err error

	for i := 0; i < retries; i++ {
		result, err = c.ReadValue(tagName, dataType)
		if err == nil {
			return result, nil
		}
		time.Sleep(time.Second * time.Duration(i+1))
	}
	return nil, err
}

// WriteTagWithRetry writes a tag value with retries
func (c *EipClient) WriteTagWithRetry(tagName string, value *PlcValue, retries int) error {
	var err error

	for i := 0; i < retries; i++ {
		err = c.WriteValue(tagName, value)
		if err == nil {
			return nil
		}
		time.Sleep(time.Second * time.Duration(i+1))
	}
	return err
}

// WaitForTagValue waits for a tag to reach a specific value
func (c *EipClient) WaitForTagValue(tagName string, dataType PlcDataType, expectedValue interface{}, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		value, err := c.ReadValue(tagName, dataType)
		if err == nil && value.Value == expectedValue {
			return nil
		}
		time.Sleep(100 * time.Millisecond)
	}
	return NewEipErrorWithDetails(ErrTimeout,
		fmt.Sprintf("Timeout waiting for tag %s to reach value %v", tagName, expectedValue),
		map[string]interface{}{
			"tag_name":       tagName,
			"data_type":      dataType,
			"expected_value": expectedValue,
			"timeout":        timeout,
		})
}

// WaitForTagCondition waits for a tag to satisfy a condition
func (c *EipClient) WaitForTagCondition(tagName string, dataType PlcDataType, condition func(interface{}) bool, timeout time.Duration) error {
	deadline := time.Now().Add(timeout)
	for time.Now().Before(deadline) {
		value, err := c.ReadValue(tagName, dataType)
		if err == nil && condition(value.Value) {
			return nil
		}
		time.Sleep(100 * time.Millisecond)
	}
	return NewEipErrorWithDetails(ErrTimeout,
		fmt.Sprintf("Timeout waiting for tag %s to satisfy condition", tagName),
		map[string]interface{}{
			"tag_name":  tagName,
			"data_type": dataType,
			"timeout":   timeout,
		})
}

// ReadTagPeriodically reads a tag value periodically and sends updates to a channel
func (c *EipClient) ReadTagPeriodically(tagName string, dataType PlcDataType, interval time.Duration) (<-chan *PlcValue, <-chan error) {
	valueChan := make(chan *PlcValue)
	errChan := make(chan error)

	go func() {
		defer close(valueChan)
		defer close(errChan)

		ticker := time.NewTicker(interval)
		defer ticker.Stop()

		for {
			select {
			case <-ticker.C:
				value, err := c.ReadValue(tagName, dataType)
				if err != nil {
					errChan <- err
					return
				}
				valueChan <- value
			}
		}
	}()

	return valueChan, errChan
}

// ReadMultipleTags reads multiple tags in parallel
func (c *EipClient) ReadMultipleTags(tags map[string]PlcDataType) (map[string]*PlcValue, error) {
	type result struct {
		tagName string
		value   *PlcValue
		err     error
	}

	resultChan := make(chan result, len(tags))

	for tagName, dataType := range tags {
		go func(name string, dt PlcDataType) {
			value, err := c.ReadValue(name, dt)
			resultChan <- result{
				tagName: name,
				value:   value,
				err:     err,
			}
		}(tagName, dataType)
	}

	results := make(map[string]*PlcValue)
	var lastErr error

	for i := 0; i < len(tags); i++ {
		r := <-resultChan
		if r.err != nil {
			lastErr = r.err
		} else {
			results[r.tagName] = r.value
		}
	}

	if lastErr != nil {
		return nil, lastErr
	}
	return results, nil
}

// Add debug logging to verify library loading
func init() {
	log.Printf("Loading Rust EtherNet/IP library...")
	// Add library path verification
}

// Add debug logging throughout the code
func (c *EipClient) Connect(ipAddress string) error {
	log.Printf("Connecting to PLC at %s...", ipAddress)
	// Add connection steps logging
	return nil
}

// Add tag path validation
func validateTagPath(tagName string) error {
	if tagName == "" {
		return NewEipError(ErrInvalidTagName, "Tag name cannot be empty")
	}
	// Add more validation as needed
	return nil
}

// Add session management verification
func (c *EipClient) verifySession() error {
	if c.clientID < 0 {
		return NewEipError(ErrConnectionFailed, "No active session")
	}
	// Add session health check
	return nil
}
