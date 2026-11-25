package main

import (
	"context"
	"errors"
	"io"

	v1 "github.com/authzed/authzed-go/proto/authzed/api/v1"
	authzed "github.com/authzed/authzed-go/v1"
	"github.com/authzed/grpcutil"
)

type spiceDB struct {
	c      *authzed.Client
	schema string
	ready  bool
}

func newSpiceDB(addr, key, schema string) (*spiceDB, error) {
	c, err := authzed.NewClient(addr, append(dialOpts(), grpcutil.WithInsecureBearerToken(key))...)
	if err != nil {
		return nil, err
	}
	return &spiceDB{c: c, schema: schema}, nil
}

func (s *spiceDB) Close() error { return s.c.Close() }

func (s *spiceDB) Setup(ctx context.Context, fresh bool) error {
	if !fresh || s.ready {
		return nil
	}
	if _, err := s.c.WriteSchema(ctx, &v1.WriteSchemaRequest{Schema: s.schema}); err != nil {
		return err
	}
	s.ready = true
	return nil
}

func sdbRel(r []string) *v1.Relationship {
	return &v1.Relationship{
		Resource: &v1.ObjectReference{ObjectType: r[0], ObjectId: r[1]},
		Relation: r[2],
		Subject:  &v1.SubjectReference{Object: &v1.ObjectReference{ObjectType: r[3], ObjectId: r[4]}},
	}
}

func (s *spiceDB) Write(ctx context.Context, rels [][]string) error {
	updates := make([]*v1.RelationshipUpdate, len(rels))
	for i, r := range rels {
		updates[i] = &v1.RelationshipUpdate{Operation: v1.RelationshipUpdate_OPERATION_CREATE, Relationship: sdbRel(r)}
	}
	_, err := s.c.WriteRelationships(ctx, &v1.WriteRelationshipsRequest{Updates: updates})
	return err
}

func fullyConsistent() *v1.Consistency {
	return &v1.Consistency{Requirement: &v1.Consistency_FullyConsistent{FullyConsistent: true}}
}

func sdbConsistency(strong bool) *v1.Consistency {
	if strong {
		return fullyConsistent()
	}
	return nil
}

func (s *spiceDB) Check(ctx context.Context, user, doc string, strong bool) (bool, error) {
	resp, err := s.c.CheckPermission(ctx, &v1.CheckPermissionRequest{
		Consistency: sdbConsistency(strong),
		Resource:    &v1.ObjectReference{ObjectType: "document", ObjectId: doc},
		Permission:  "view",
		Subject:     &v1.SubjectReference{Object: &v1.ObjectReference{ObjectType: "user", ObjectId: user}},
	})
	if err != nil {
		return false, err
	}
	return resp.GetPermissionship() == v1.CheckPermissionResponse_PERMISSIONSHIP_HAS_PERMISSION, nil
}

func (s *spiceDB) List(ctx context.Context, user string, strong bool) ([]string, error) {
	stream, err := s.c.LookupResources(ctx, &v1.LookupResourcesRequest{
		Consistency:        sdbConsistency(strong),
		ResourceObjectType: "document",
		Permission:         "view",
		Subject:            &v1.SubjectReference{Object: &v1.ObjectReference{ObjectType: "user", ObjectId: user}},
	})
	if err != nil {
		return nil, err
	}
	var out []string
	for {
		r, err := stream.Recv()
		if errors.Is(err, io.EOF) {
			return out, nil
		}
		if err != nil {
			return nil, err
		}
		if r.GetPermissionship() == v1.LookupPermissionship_LOOKUP_PERMISSIONSHIP_HAS_PERMISSION {
			out = append(out, r.GetResourceObjectId())
		}
	}
}
