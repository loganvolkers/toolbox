package main

import (
	"context"
	"errors"
	"fmt"
	"io"
	"net"
	"net/http"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"sync/atomic"
	"time"

	v1 "github.com/authzed/authzed-go/proto/authzed/api/v1"
	authzed "github.com/authzed/authzed-go/v1"
	"github.com/authzed/grpcutil"
	openfgav1 "github.com/openfga/api/proto/openfga/v1"
	collectortrace "go.opentelemetry.io/proto/otlp/collector/trace/v1"
	"google.golang.org/grpc"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/structpb"
)

// fact is what a probe decided, with what it saw.
type fact struct {
	Value  bool   `json:"value"`
	Detail string `json:"detail"`
}

func fieldNames(m proto.Message) []string {
	fs := m.ProtoReflect().Descriptor().Fields()
	out := make([]string, fs.Len())
	for i := range out {
		out[i] = string(fs.Get(i).Name())
	}
	return out
}

func errString(err error) string {
	if err == nil {
		return "ok"
	}
	return err.Error()
}

// probe starts the server as given, waits until it answers a check, then decides the facts a
// running server can decide. The server runs with its defaults, so the HTTP and metrics URLs
// are its default ports.
func probe(ctx context.Context, af armFlags, a Arm, argv []string, logPath, httpURL, metricsURL string) map[string]any {
	s, t0, err := startServer(argv, logPath)
	if err != nil {
		fail(err)
	}
	defer s.stop()
	ready, err := waitReady(ctx, a, s, t0)
	if err != nil {
		fail(err)
	}
	out := map[string]any{"arm": af.arm, "server": argv, "readyS": ready}
	var facts map[string]fact
	switch af.arm {
	case "openfga":
		facts, err = probeOpenFGA(ctx, af)
	case "spicedb":
		facts, err = probeSpiceDB(ctx, af)
	}
	if err != nil {
		fail(err)
	}
	facts["httpByDefault"] = probeHTTP(httpURL)
	facts["prometheusMetrics"] = probeMetrics(metricsURL)
	out["facts"] = facts
	return out
}

func probeHTTP(url string) fact {
	c := http.Client{Timeout: 3 * time.Second}
	resp, err := c.Get(url)
	if err != nil {
		return fact{false, fmt.Sprintf("GET %s: %v", url, err)}
	}
	defer resp.Body.Close()
	b, _ := io.ReadAll(io.LimitReader(resp.Body, 200))
	return fact{resp.StatusCode < 500, fmt.Sprintf("GET %s: HTTP %d %s", url, resp.StatusCode, strings.TrimSpace(string(b)))}
}

func probeMetrics(url string) fact {
	c := http.Client{Timeout: 3 * time.Second}
	resp, err := c.Get(url)
	if err != nil {
		return fact{false, fmt.Sprintf("GET %s: %v", url, err)}
	}
	defer resp.Body.Close()
	b, _ := io.ReadAll(resp.Body)
	families := 0
	var grpcNames []string
	for _, line := range strings.Split(string(b), "\n") {
		if strings.HasPrefix(line, "# TYPE ") {
			families++
			name := strings.Fields(line)[2]
			if strings.HasPrefix(name, "grpc_server_handled") {
				grpcNames = append(grpcNames, name)
			}
		}
	}
	return fact{resp.StatusCode == 200 && families > 0,
		fmt.Sprintf("GET %s: HTTP %d, %d Prometheus metric families (%s)", url, resp.StatusCode, families, strings.Join(grpcNames, ", "))}
}

func probeOpenFGA(ctx context.Context, af armFlags) (map[string]fact, error) {
	conn, err := grpc.NewClient(af.addr, dialOpts()...)
	if err != nil {
		return nil, err
	}
	defer conn.Close()
	c := openfgav1.NewOpenFGAServiceClient(conn)
	dsl, err := os.ReadFile(filepath.Join(af.dir, "probes", "model.fga"))
	if err != nil {
		return nil, err
	}
	sa, err := c.CreateStore(ctx, &openfgav1.CreateStoreRequest{Name: "probe-a"})
	if err != nil {
		return nil, err
	}
	ma, err := writeModel(ctx, c, sa.GetId(), string(dsl))
	if err != nil {
		return nil, err
	}
	tk := func(obj, rel, user string) *openfgav1.TupleKey {
		return &openfgav1.TupleKey{Object: obj, Relation: rel, User: user}
	}
	cond := tk("doc:d2", "member", "user:dave")
	cond.Condition = &openfgav1.RelationshipCondition{Name: "in_hours"}
	writeResp, err := c.Write(ctx, &openfgav1.WriteRequest{StoreId: sa.GetId(), AuthorizationModelId: ma,
		Writes: &openfgav1.WriteRequestWrites{TupleKeys: []*openfgav1.TupleKey{
			tk("doc:pub", "member", "user:*"),
			tk("doc:d1", "member", "user:bob"), tk("doc:d1", "blocked", "user:bob"),
			tk("doc:d1", "member", "user:carol"), tk("doc:d1", "verified", "user:carol"),
			cond,
		}}})
	if err != nil {
		return nil, err
	}
	type q struct {
		obj, rel, user string
		ctx            map[string]any
		ct             []*openfgav1.TupleKey
	}
	check := func(store, model string, x q) string {
		req := &openfgav1.CheckRequest{StoreId: store, AuthorizationModelId: model,
			TupleKey: &openfgav1.CheckRequestTupleKey{Object: x.obj, Relation: x.rel, User: x.user}}
		if x.ctx != nil {
			req.Context, _ = structpb.NewStruct(x.ctx)
		}
		if x.ct != nil {
			req.ContextualTuples = &openfgav1.ContextualTupleKeys{TupleKeys: x.ct}
		}
		r, err := c.Check(ctx, req)
		if err != nil {
			return "error: " + err.Error()
		}
		return fmt.Sprint(r.GetAllowed())
	}
	ca := func(x q) string { return check(sa.GetId(), ma, x) }
	f := map[string]fact{}

	pub := ca(q{obj: "doc:pub", rel: "viewer", user: "user:alice"})
	f["wildcards"] = fact{pub == "true", "doc:pub#member@user:* written; Check(doc:pub viewer user:alice) = " + pub}

	ex := []string{ca(q{obj: "doc:d1", rel: "viewer", user: "user:bob"}), ca(q{obj: "doc:d1", rel: "viewer", user: "user:carol"}),
		ca(q{obj: "doc:d1", rel: "editor", user: "user:carol"}), ca(q{obj: "doc:d1", rel: "editor", user: "user:bob"})}
	f["exclusion"] = fact{ex[0] == "false" && ex[1] == "true" && ex[2] == "true" && ex[3] == "false",
		fmt.Sprintf("viewer = member but not blocked, editor = member and verified; bob (member, blocked): viewer %s; carol (member, verified): viewer %s, editor %s; bob editor %s", ex[0], ex[1], ex[2], ex[3])}

	in := ca(q{obj: "doc:d2", rel: "viewer", user: "user:dave", ctx: map[string]any{"hour": 10}})
	outOf := ca(q{obj: "doc:d2", rel: "viewer", user: "user:dave", ctx: map[string]any{"hour": 20}})
	f["conditions"] = fact{in == "true" && outOf == "false",
		fmt.Sprintf("doc:d2#member@user:dave with in_hours; Check with context hour=10: %s, hour=20: %s", in, outOf)}

	without := ca(q{obj: "doc:d3", rel: "viewer", user: "user:erin"})
	with := ca(q{obj: "doc:d3", rel: "viewer", user: "user:erin", ct: []*openfgav1.TupleKey{tk("doc:d3", "member", "user:erin")}})
	after := ca(q{obj: "doc:d3", rel: "viewer", user: "user:erin"})
	f["contextualTuples"] = fact{without == "false" && with == "true" && after == "false",
		fmt.Sprintf("Check(doc:d3 viewer user:erin): %s; with contextual tuple doc:d3#member@user:erin: %s; again without: %s", without, with, after)}

	lu, err := c.ListUsers(ctx, &openfgav1.ListUsersRequest{StoreId: sa.GetId(), AuthorizationModelId: ma,
		Object: &openfgav1.Object{Type: "doc", Id: "d1"}, Relation: "viewer",
		UserFilters: []*openfgav1.UserTypeFilter{{Type: "user"}}})
	var users []string
	for _, u := range lu.GetUsers() {
		users = append(users, u.GetObject().GetType()+":"+u.GetObject().GetId())
	}
	sort.Strings(users)
	f["listUsers"] = fact{err == nil && strings.Join(users, ",") == "user:carol",
		fmt.Sprintf("ListUsers(doc:d1 viewer, filter user): %v (%s)", users, errString(err))}

	f["consistencyTokens"] = fact{len(fieldNames(writeResp)) > 0,
		fmt.Sprintf("WriteResponse fields: %v; CheckRequest fields: %v (consistency is an enum: UNSPECIFIED, MINIMIZE_LATENCY, HIGHER_CONSISTENCY)",
			fieldNames(writeResp), fieldNames(&openfgav1.CheckRequest{}))}

	ch1, err := c.ReadChanges(ctx, &openfgav1.ReadChangesRequest{StoreId: sa.GetId(), Type: "doc"})
	if err != nil {
		f["changeFeed"] = fact{false, "ReadChanges: " + err.Error()}
	} else {
		_, werr := c.Write(ctx, &openfgav1.WriteRequest{StoreId: sa.GetId(), AuthorizationModelId: ma,
			Writes: &openfgav1.WriteRequestWrites{TupleKeys: []*openfgav1.TupleKey{tk("doc:d4", "member", "user:frank")}}})
		ch2, err := c.ReadChanges(ctx, &openfgav1.ReadChangesRequest{StoreId: sa.GetId(), Type: "doc", ContinuationToken: ch1.GetContinuationToken()})
		var got []string
		for _, x := range ch2.GetChanges() {
			k := x.GetTupleKey()
			got = append(got, fmt.Sprintf("%s %s#%s@%s", x.GetOperation(), k.GetObject(), k.GetRelation(), k.GetUser()))
		}
		f["changeFeed"] = fact{werr == nil && err == nil && len(got) == 1 && strings.Contains(got[0], "doc:d4#member@user:frank"),
			fmt.Sprintf("ReadChanges: %d changes and a continuation token; after one more write, ReadChanges from that token: %v (%s)", len(ch1.GetChanges()), got, errString(err))}
	}

	dslB, err := os.ReadFile(filepath.Join(af.dir, "probes", "tenant-b.fga"))
	if err != nil {
		return nil, err
	}
	sb, err := c.CreateStore(ctx, &openfgav1.CreateStoreRequest{Name: "probe-b"})
	if err != nil {
		return nil, err
	}
	mb, err := writeModel(ctx, c, sb.GetId(), string(dslB))
	if err != nil {
		return nil, err
	}
	_, err = c.Write(ctx, &openfgav1.WriteRequest{StoreId: sb.GetId(), AuthorizationModelId: mb,
		Writes: &openfgav1.WriteRequestWrites{TupleKeys: []*openfgav1.TupleKey{tk("doc:d1", "member", "user:zed")}}})
	inB := check(sb.GetId(), mb, q{obj: "doc:d1", rel: "member", user: "user:zed"})
	inA := ca(q{obj: "doc:d1", rel: "member", user: "user:zed"})
	viewerB := check(sb.GetId(), mb, q{obj: "doc:d1", rel: "viewer", user: "user:carol"})
	f["multiTenant"] = fact{err == nil && inB == "true" && inA == "false" && strings.HasPrefix(viewerB, "error"),
		fmt.Sprintf("two stores on one server, each with its own model; doc:d1#member@user:zed written to store B: Check in B %s, in A %s; relation viewer (only in A's model) asked in B: %s", inB, inA, viewerB)}
	return f, nil
}

func probeSpiceDB(ctx context.Context, af armFlags) (map[string]fact, error) {
	c, err := authzed.NewClient(af.addr, append(dialOpts(), grpcutil.WithInsecureBearerToken(af.key))...)
	if err != nil {
		return nil, err
	}
	defer c.Close()
	schema, err := os.ReadFile(filepath.Join(af.dir, "probes", "schema.zed"))
	if err != nil {
		return nil, err
	}
	if _, err := c.WriteSchema(ctx, &v1.WriteSchemaRequest{Schema: string(schema)}); err != nil {
		return nil, err
	}
	rel := func(obj, relation, subj string) *v1.RelationshipUpdate {
		o := strings.SplitN(obj, ":", 2)
		s := strings.SplitN(subj, ":", 2)
		return &v1.RelationshipUpdate{Operation: v1.RelationshipUpdate_OPERATION_TOUCH, Relationship: &v1.Relationship{
			Resource: &v1.ObjectReference{ObjectType: o[0], ObjectId: o[1]}, Relation: relation,
			Subject: &v1.SubjectReference{Object: &v1.ObjectReference{ObjectType: s[0], ObjectId: s[1]}}}}
	}
	cond := rel("doc:d2", "member", "user:dave")
	cond.Relationship.OptionalCaveat = &v1.ContextualizedCaveat{CaveatName: "in_hours"}
	wr, err := c.WriteRelationships(ctx, &v1.WriteRelationshipsRequest{Updates: []*v1.RelationshipUpdate{
		rel("doc:pub", "member", "user:*"),
		rel("doc:d1", "member", "user:bob"), rel("doc:d1", "blocked", "user:bob"),
		rel("doc:d1", "member", "user:carol"), rel("doc:d1", "verified", "user:carol"),
		cond,
	}})
	if err != nil {
		return nil, err
	}
	token := wr.GetWrittenAt()
	fresh := &v1.Consistency{Requirement: &v1.Consistency_AtLeastAsFresh{AtLeastAsFresh: token}}
	check := func(obj, perm, user string, cctx map[string]any) string {
		o := strings.SplitN(obj, ":", 2)
		u := strings.SplitN(user, ":", 2)
		req := &v1.CheckPermissionRequest{Consistency: fresh,
			Resource:   &v1.ObjectReference{ObjectType: o[0], ObjectId: o[1]},
			Permission: perm,
			Subject:    &v1.SubjectReference{Object: &v1.ObjectReference{ObjectType: u[0], ObjectId: u[1]}}}
		if cctx != nil {
			req.Context, _ = structpb.NewStruct(cctx)
		}
		r, err := c.CheckPermission(ctx, req)
		if err != nil {
			return "error: " + err.Error()
		}
		switch r.GetPermissionship() {
		case v1.CheckPermissionResponse_PERMISSIONSHIP_HAS_PERMISSION:
			return "true"
		case v1.CheckPermissionResponse_PERMISSIONSHIP_NO_PERMISSION:
			return "false"
		}
		return r.GetPermissionship().String()
	}
	f := map[string]fact{}

	pub := check("doc:pub", "viewer", "user:alice", nil)
	f["wildcards"] = fact{pub == "true", "doc:pub#member@user:* written; CheckPermission(doc:pub viewer user:alice) = " + pub}

	ex := []string{check("doc:d1", "viewer", "user:bob", nil), check("doc:d1", "viewer", "user:carol", nil),
		check("doc:d1", "editor", "user:carol", nil), check("doc:d1", "editor", "user:bob", nil)}
	f["exclusion"] = fact{ex[0] == "false" && ex[1] == "true" && ex[2] == "true" && ex[3] == "false",
		fmt.Sprintf("viewer = member - blocked, editor = member & verified; bob (member, blocked): viewer %s; carol (member, verified): viewer %s, editor %s; bob editor %s", ex[0], ex[1], ex[2], ex[3])}

	in := check("doc:d2", "viewer", "user:dave", map[string]any{"hour": 10})
	outOf := check("doc:d2", "viewer", "user:dave", map[string]any{"hour": 20})
	f["conditions"] = fact{in == "true" && outOf == "false",
		fmt.Sprintf("doc:d2#member@user:dave with caveat in_hours; CheckPermission with context hour=10: %s, hour=20: %s", in, outOf)}

	without := check("doc:d3", "viewer", "user:erin", nil)
	f["contextualTuples"] = fact{false,
		fmt.Sprintf("CheckPermissionRequest fields: %v; LookupResourcesRequest fields: %v; none carries request-scoped relationships, so the probe cannot send one (Check(doc:d3 viewer user:erin) = %s)",
			fieldNames(&v1.CheckPermissionRequest{}), fieldNames(&v1.LookupResourcesRequest{}), without)}

	ls, err := c.LookupSubjects(ctx, &v1.LookupSubjectsRequest{Consistency: fresh,
		Resource: &v1.ObjectReference{ObjectType: "doc", ObjectId: "d1"}, Permission: "viewer", SubjectObjectType: "user"})
	var users []string
	if err == nil {
		for {
			r, rerr := ls.Recv()
			if errors.Is(rerr, io.EOF) {
				break
			}
			if rerr != nil {
				err = rerr
				break
			}
			users = append(users, "user:"+r.GetSubject().GetSubjectObjectId())
		}
	}
	sort.Strings(users)
	f["listUsers"] = fact{err == nil && strings.Join(users, ",") == "user:carol",
		fmt.Sprintf("LookupSubjects(doc:d1 viewer, subject type user): %v (%s)", users, errString(err))}

	tokCheck := check("doc:d1", "viewer", "user:carol", nil)
	f["consistencyTokens"] = fact{token.GetToken() != "" && tokCheck == "true",
		fmt.Sprintf("WriteRelationships returned written_at ZedToken %q; CheckPermission with at_least_as_fresh that token: %s", token.GetToken(), tokCheck)}

	wctx, cancel := context.WithTimeout(ctx, 15*time.Second)
	defer cancel()
	ws, err := c.Watch(wctx, &v1.WatchRequest{OptionalStartCursor: token})
	if err != nil {
		f["changeFeed"] = fact{false, "Watch: " + err.Error()}
	} else {
		got := make(chan string, 1)
		go func() {
			for {
				r, err := ws.Recv()
				if err != nil {
					got <- "error: " + err.Error()
					return
				}
				for _, u := range r.GetUpdates() {
					x := u.GetRelationship()
					if x.GetResource().GetObjectId() == "d4" {
						got <- fmt.Sprintf("%s %s:%s#%s@%s:%s", u.GetOperation(), x.GetResource().GetObjectType(), x.GetResource().GetObjectId(),
							x.GetRelation(), x.GetSubject().GetObject().GetObjectType(), x.GetSubject().GetObject().GetObjectId())
						return
					}
				}
			}
		}()
		time.Sleep(500 * time.Millisecond)
		_, werr := c.WriteRelationships(ctx, &v1.WriteRelationshipsRequest{Updates: []*v1.RelationshipUpdate{rel("doc:d4", "member", "user:frank")}})
		var seen string
		select {
		case seen = <-got:
		case <-time.After(10 * time.Second):
			seen = "nothing within 10 s"
		}
		f["changeFeed"] = fact{werr == nil && strings.Contains(seen, "doc:d4#member@user:frank"),
			fmt.Sprintf("Watch from the first write's ZedToken; after one more write it streamed: %s", seen)}
	}

	tenantB := string(schema) + "\ndefinition tenantb/doc {\n\trelation member: user\n}\n"
	_, err = c.WriteSchema(ctx, &v1.WriteSchemaRequest{Schema: tenantB})
	rs, rerr := c.ReadSchema(ctx, &v1.ReadSchemaRequest{})
	defs := strings.Count(rs.GetSchemaText(), "definition ")
	f["multiTenant"] = fact{false,
		fmt.Sprintf("WriteSchemaRequest fields: %v (no store or tenant); a second tenant's definition can only be added to the one schema under a prefix: WriteSchema %s, ReadSchema returns one schema with %d definitions (%s)",
			fieldNames(&v1.WriteSchemaRequest{}), errString(err), defs, errString(rerr))}
	return f, nil
}

type traceCollector struct {
	collectortrace.UnimplementedTraceServiceServer
	spans atomic.Int64
	names atomic.Value
}

func (t *traceCollector) Export(_ context.Context, req *collectortrace.ExportTraceServiceRequest) (*collectortrace.ExportTraceServiceResponse, error) {
	var names []string
	for _, rs := range req.GetResourceSpans() {
		for _, ss := range rs.GetScopeSpans() {
			for _, sp := range ss.GetSpans() {
				t.spans.Add(1)
				if len(names) < 5 {
					names = append(names, sp.GetName())
				}
			}
		}
	}
	if len(names) > 0 {
		t.names.Store(names)
	}
	return &collectortrace.ExportTraceServiceResponse{}, nil
}

// traceProbe runs an OTLP/gRPC trace collector, starts the server configured to export to it,
// sends checks, and reports the spans that arrive.
func traceProbe(ctx context.Context, af armFlags, otlpAddr string, argv []string, logPath string) map[string]any {
	lis, err := net.Listen("tcp", otlpAddr)
	if err != nil {
		fail(err)
	}
	col := &traceCollector{}
	gs := grpc.NewServer()
	collectortrace.RegisterTraceServiceServer(gs, col)
	go func() { _ = gs.Serve(lis) }()
	defer gs.Stop()
	a, err := newArm(af)
	if err != nil {
		fail(err)
	}
	defer a.Close()
	s, t0, err := startServer(argv, logPath)
	if err != nil {
		fail(err)
	}
	if _, err := waitReady(ctx, a, s, t0); err != nil {
		s.stop()
		fail(err)
	}
	for i := 0; i < 50; i++ {
		_, _ = a.Check(ctx, "u0", "d0", false)
	}
	deadline := time.Now().Add(20 * time.Second)
	for time.Now().Before(deadline) && col.spans.Load() == 0 {
		time.Sleep(200 * time.Millisecond)
	}
	s.stop()
	names, _ := col.names.Load().([]string)
	n := col.spans.Load()
	return map[string]any{"arm": af.arm, "server": argv, "facts": map[string]fact{
		"otelTracing": {n > 0, fmt.Sprintf("OTLP/gRPC collector at %s received %d spans after 50 checks (first: %v)", otlpAddr, n, names)},
	}}
}
