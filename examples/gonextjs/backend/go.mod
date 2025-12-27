module github.com/sergiogallegos/rust-ethernet-ip/examples/gonextjs/backend

go 1.23.0

toolchain go1.24.3

require (
	github.com/gorilla/mux v1.8.1
	github.com/gorilla/websocket v1.5.1
	github.com/sergiogallegos/rust-ethernet-ip/gowrapper v0.0.0
)

require golang.org/x/net v0.17.0 // indirect

replace github.com/sergiogallegos/rust-ethernet-ip/gowrapper => ../../../gowrapper

replace github.com/sergiogallegos/rust-ethernet-ip/gowrapper/ethernetip => ../../../gowrapper/ethernetip/ethernetip
