package main

import (
	"context"
	"fmt"
	"os"
	"path/filepath"
	"time"

	"google.golang.org/grpc"
	"google.golang.org/grpc/backoff"
	"google.golang.org/grpc/credentials/insecure"
)

// Arm is one product behind its own gRPC API. Every arm answers the same corpus with the same
// relation names: relationships use the corpus's names, and Check/List ask whether a user can
// view a document (OpenFGA relation `viewer`, SpiceDB permission `view`).
type Arm interface {
	// Setup creates the store and model (OpenFGA) or writes the schema (SpiceDB) when fresh is
	// true, and finds them on a running server otherwise. A failed Setup can be retried.
	Setup(ctx context.Context, fresh bool) error
	Write(ctx context.Context, rels [][]string) error
	// Check and List read at the product's default consistency, or at its strongest one when
	// strong is true (SpiceDB fully_consistent, OpenFGA HIGHER_CONSISTENCY).
	Check(ctx context.Context, user, doc string, strong bool) (bool, error)
	List(ctx context.Context, user string, strong bool) ([]string, error)
	Close() error
}

// dialOpts keeps gRPC's reconnect backoff (1 s base by default) out of readyS: the client retries
// a server that is not listening yet every 10 ms, the same interval waitReady polls at.
func dialOpts() []grpc.DialOption {
	return []grpc.DialOption{
		grpc.WithTransportCredentials(insecure.NewCredentials()),
		grpc.WithConnectParams(grpc.ConnectParams{
			Backoff:           backoff.Config{BaseDelay: 10 * time.Millisecond, Multiplier: 1, MaxDelay: 10 * time.Millisecond},
			MinConnectTimeout: time.Second,
		}),
	}
}

type armFlags struct {
	arm, addr, key, dir string
}

func newArm(f armFlags) (Arm, error) {
	switch f.arm {
	case "openfga":
		dsl, err := os.ReadFile(filepath.Join(f.dir, "model.fga"))
		if err != nil {
			return nil, err
		}
		return newOpenFGA(f.addr, string(dsl))
	case "spicedb":
		schema, err := os.ReadFile(filepath.Join(f.dir, "schema.zed"))
		if err != nil {
			return nil, err
		}
		return newSpiceDB(f.addr, f.key, string(schema))
	}
	return nil, fmt.Errorf("unknown arm %q", f.arm)
}
