package main

import (
	"context"
	"fmt"
	"strings"

	openfgav1 "github.com/openfga/api/proto/openfga/v1"
	"github.com/openfga/language/pkg/go/transformer"
	"google.golang.org/grpc"
)

const benchStore = "bench"

type openFGA struct {
	conn         *grpc.ClientConn
	c            openfgav1.OpenFGAServiceClient
	dsl          string
	store, model string
}

func newOpenFGA(addr, dsl string) (*openFGA, error) {
	conn, err := grpc.NewClient(addr, dialOpts()...)
	if err != nil {
		return nil, err
	}
	return &openFGA{conn: conn, c: openfgav1.NewOpenFGAServiceClient(conn), dsl: dsl}, nil
}

func (o *openFGA) Close() error { return o.conn.Close() }

func writeModel(ctx context.Context, c openfgav1.OpenFGAServiceClient, store, dsl string) (string, error) {
	m, err := transformer.TransformDSLToProto(dsl)
	if err != nil {
		return "", err
	}
	resp, err := c.WriteAuthorizationModel(ctx, &openfgav1.WriteAuthorizationModelRequest{
		StoreId: store, TypeDefinitions: m.GetTypeDefinitions(),
		SchemaVersion: m.GetSchemaVersion(), Conditions: m.GetConditions(),
	})
	if err != nil {
		return "", err
	}
	return resp.GetAuthorizationModelId(), nil
}

func (o *openFGA) Setup(ctx context.Context, fresh bool) error {
	if !fresh {
		stores, err := o.c.ListStores(ctx, &openfgav1.ListStoresRequest{})
		if err != nil {
			return err
		}
		for _, s := range stores.GetStores() {
			if s.GetName() == benchStore {
				o.store = s.GetId()
			}
		}
		if o.store == "" {
			return fmt.Errorf("no store named %q", benchStore)
		}
		models, err := o.c.ReadAuthorizationModels(ctx, &openfgav1.ReadAuthorizationModelsRequest{StoreId: o.store})
		if err != nil {
			return err
		}
		if len(models.GetAuthorizationModels()) == 0 {
			return fmt.Errorf("store %s has no model", o.store)
		}
		o.model = models.GetAuthorizationModels()[0].GetId()
		return nil
	}
	if o.store == "" {
		resp, err := o.c.CreateStore(ctx, &openfgav1.CreateStoreRequest{Name: benchStore})
		if err != nil {
			return err
		}
		o.store = resp.GetId()
	}
	if o.model == "" {
		id, err := writeModel(ctx, o.c, o.store, o.dsl)
		if err != nil {
			return err
		}
		o.model = id
	}
	return nil
}

func ofgaTuple(r []string) *openfgav1.TupleKey {
	return &openfgav1.TupleKey{Object: r[0] + ":" + r[1], Relation: r[2], User: r[3] + ":" + r[4]}
}

func (o *openFGA) Write(ctx context.Context, rels [][]string) error {
	keys := make([]*openfgav1.TupleKey, len(rels))
	for i, r := range rels {
		keys[i] = ofgaTuple(r)
	}
	_, err := o.c.Write(ctx, &openfgav1.WriteRequest{
		StoreId: o.store, AuthorizationModelId: o.model,
		Writes: &openfgav1.WriteRequestWrites{TupleKeys: keys},
	})
	return err
}

func ofgaConsistency(strong bool) openfgav1.ConsistencyPreference {
	if strong {
		return openfgav1.ConsistencyPreference_HIGHER_CONSISTENCY
	}
	return openfgav1.ConsistencyPreference_UNSPECIFIED
}

func (o *openFGA) Check(ctx context.Context, user, doc string, strong bool) (bool, error) {
	resp, err := o.c.Check(ctx, &openfgav1.CheckRequest{
		StoreId: o.store, AuthorizationModelId: o.model,
		TupleKey:    &openfgav1.CheckRequestTupleKey{User: "user:" + user, Relation: "viewer", Object: "document:" + doc},
		Consistency: ofgaConsistency(strong),
	})
	if err != nil {
		return false, err
	}
	return resp.GetAllowed(), nil
}

func (o *openFGA) List(ctx context.Context, user string, strong bool) ([]string, error) {
	resp, err := o.c.ListObjects(ctx, &openfgav1.ListObjectsRequest{
		StoreId: o.store, AuthorizationModelId: o.model,
		Type: "document", Relation: "viewer", User: "user:" + user,
		Consistency: ofgaConsistency(strong),
	})
	if err != nil {
		return nil, err
	}
	out := make([]string, 0, len(resp.GetObjects()))
	for _, obj := range resp.GetObjects() {
		out = append(out, strings.TrimPrefix(obj, "document:"))
	}
	return out, nil
}
