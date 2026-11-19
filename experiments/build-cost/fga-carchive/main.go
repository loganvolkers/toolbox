// A minimal c-archive embedding the OpenFGA server, as authorization.md §4.3 proposes.
// Used only to measure build time and archive size; the datastore is OpenFGA's in-memory one.
package main

import "C"

import (
	"github.com/openfga/openfga/pkg/server"
	"github.com/openfga/openfga/pkg/storage/memory"
)

//export tb_fga_init
func tb_fga_init() C.int {
	s, err := server.NewServerWithOpts(server.WithDatastore(memory.New()))
	if err != nil {
		return 1
	}
	s.Close()
	return 0
}

func main() {}
