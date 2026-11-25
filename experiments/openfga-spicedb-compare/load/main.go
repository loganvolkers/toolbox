// Command loadgen drives one arm of experiments/openfga-spicedb-compare over gRPC.
//
//	loadgen round       -arm A -round N -out F -log L -- <server command>   one timed round (run.sh)
//	loadgen write       -arm A -addr H                                      write the corpus
//	loadgen equivalence -arm A -addr H                                      2,000 checks, 50 lists, digests
//	loadgen checkload   -arm A -addr H                                      16 workers, 5 s warm-up, 30 s
//	loadgen listload    -arm A -addr H                                      ListObjects / LookupResources p95
//	loadgen probe       -arm A -addr H -http URL -metrics URL -- <server>   modelling and operability probes
//	loadgen traceprobe  -arm A -addr H -otlp ADDR -- <server>               does the server export OTLP traces
//
// Every subcommand prints one JSON object to stdout.
package main

import (
	"context"
	"crypto/sha256"
	"encoding/hex"
	"encoding/json"
	"flag"
	"fmt"
	"math"
	"os"
	"os/exec"
	"path/filepath"
	"sort"
	"strconv"
	"strings"
	"sync"
	"sync/atomic"
	"syscall"
	"time"
)

const (
	batchSize = 100
	workers   = 16
	warmup    = 5 * time.Second
	measure   = 30 * time.Second
	listPass  = 3
)

type corpus struct {
	Relationships [][]string  `json:"relationships"`
	Checks        [][2]string `json:"checks"`
	ListUsers     []string    `json:"listUsers"`
	Expected      struct {
		DecisionsDigest string `json:"decisionsDigest"`
		ListsDigest     string `json:"listsDigest"`
		Decisions       []bool `json:"decisions"`
	} `json:"expected"`
}

func loadCorpus(dir string) (*corpus, error) {
	b, err := os.ReadFile(filepath.Join(dir, "corpus.json"))
	if err != nil {
		return nil, err
	}
	var c corpus
	return &c, json.Unmarshal(b, &c)
}

func ms(d time.Duration) float64 { return math.Round(float64(d.Microseconds())/10) / 100 }

func pct(lat []time.Duration, p float64) float64 {
	if len(lat) == 0 {
		return 0
	}
	s := append([]time.Duration(nil), lat...)
	sort.Slice(s, func(i, j int) bool { return s[i] < s[j] })
	i := int(math.Ceil(p*float64(len(s)))) - 1
	return ms(s[max(i, 0)])
}

type writeResult struct {
	Relationships int     `json:"relationships"`
	Requests      int     `json:"requests"`
	Seconds       float64 `json:"seconds"`
	PerS          float64 `json:"perS"`
}

// writeCorpus writes every relationship, batchSize per request, from one writer.
func writeCorpus(ctx context.Context, a Arm, c *corpus) (writeResult, error) {
	rels := c.Relationships
	start := time.Now()
	n := 0
	for i := 0; i < len(rels); i += batchSize {
		if err := a.Write(ctx, rels[i:min(i+batchSize, len(rels))]); err != nil {
			return writeResult{}, fmt.Errorf("write batch %d: %w", i/batchSize, err)
		}
		n++
	}
	s := time.Since(start).Seconds()
	return writeResult{len(rels), n, round3(s), math.Round(float64(len(rels)) / s)}, nil
}

func round3(f float64) float64 { return math.Round(f*1000) / 1000 }

type equivalenceResult struct {
	DecisionsDigest string         `json:"decisionsDigest"`
	ListsDigest     string         `json:"listsDigest"`
	Allowed         int            `json:"allowed"`
	ListSizes       map[string]int `json:"listSizes"`
	OracleMatch     bool           `json:"oracleMatch"`
	Seconds         float64        `json:"seconds"`
}

// equivalence answers the sample and lists the users' documents at the strongest consistency, and
// digests the answers the same way gen.py does.
func equivalence(ctx context.Context, a Arm, c *corpus) (equivalenceResult, error) {
	start := time.Now()
	decisions := make([]bool, len(c.Checks))
	var next atomic.Int64
	var firstErr error
	var mu sync.Mutex
	var wg sync.WaitGroup
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func() {
			defer wg.Done()
			for {
				i := int(next.Add(1) - 1)
				if i >= len(c.Checks) {
					return
				}
				ok, err := a.Check(ctx, c.Checks[i][0], c.Checks[i][1], true)
				if err != nil {
					mu.Lock()
					firstErr = fmt.Errorf("check %d: %w", i, err)
					mu.Unlock()
					return
				}
				decisions[i] = ok
			}
		}()
	}
	wg.Wait()
	if firstErr != nil {
		return equivalenceResult{}, firstErr
	}
	var dl strings.Builder
	allowed := 0
	for i, ch := range c.Checks {
		v := 0
		if decisions[i] {
			v, allowed = 1, allowed+1
		}
		fmt.Fprintf(&dl, "%s %s %d\n", ch[0], ch[1], v)
	}
	var ll strings.Builder
	sizes := map[string]int{}
	for _, u := range c.ListUsers {
		docs, err := a.List(ctx, u, true)
		if err != nil {
			return equivalenceResult{}, fmt.Errorf("list %s: %w", u, err)
		}
		set := map[string]bool{}
		for _, d := range docs {
			set[d] = true
		}
		uniq := make([]string, 0, len(set))
		for d := range set {
			uniq = append(uniq, d)
		}
		sort.Strings(uniq)
		sizes[u] = len(uniq)
		fmt.Fprintf(&ll, "%s %s\n", u, strings.Join(uniq, ","))
	}
	r := equivalenceResult{
		DecisionsDigest: digest(dl.String()), ListsDigest: digest(ll.String()),
		Allowed: allowed, ListSizes: sizes, Seconds: round3(time.Since(start).Seconds()),
	}
	r.OracleMatch = r.DecisionsDigest == c.Expected.DecisionsDigest && r.ListsDigest == c.Expected.ListsDigest
	return r, nil
}

func digest(s string) string {
	h := sha256.Sum256([]byte(s))
	return hex.EncodeToString(h[:])
}

type checkResult struct {
	Workers    int     `json:"workers"`
	WarmupS    float64 `json:"warmupS"`
	MeasureS   float64 `json:"measureS"`
	Count      int     `json:"count"`
	Errors     int     `json:"errors"`
	Mismatches int     `json:"mismatches"`
	P50Ms      float64 `json:"p50Ms"`
	P95Ms      float64 `json:"p95Ms"`
	P99Ms      float64 `json:"p99Ms"`
	Rps        float64 `json:"rps"`
}

// checkLoad cycles the sample from 16 workers at default consistency. Requests that start in the
// measured window count; mismatches against the expected answers are counted, not fatal.
func checkLoad(ctx context.Context, a Arm, c *corpus) checkResult {
	t0 := time.Now()
	from, to := t0.Add(warmup), t0.Add(warmup+measure)
	var next atomic.Int64
	lats := make([][]time.Duration, workers)
	errs := make([]int, workers)
	bad := make([]int, workers)
	var wg sync.WaitGroup
	for w := 0; w < workers; w++ {
		wg.Add(1)
		go func(w int) {
			defer wg.Done()
			for {
				t := time.Now()
				if !t.Before(to) {
					return
				}
				i := int(next.Add(1)-1) % len(c.Checks)
				ok, err := a.Check(ctx, c.Checks[i][0], c.Checks[i][1], false)
				d := time.Since(t)
				if t.Before(from) {
					continue
				}
				if err != nil {
					errs[w]++
					continue
				}
				if ok != c.Expected.Decisions[i] {
					bad[w]++
				}
				lats[w] = append(lats[w], d)
			}
		}(w)
	}
	wg.Wait()
	var all []time.Duration
	r := checkResult{Workers: workers, WarmupS: warmup.Seconds(), MeasureS: measure.Seconds()}
	for w := range lats {
		all = append(all, lats[w]...)
		r.Errors += errs[w]
		r.Mismatches += bad[w]
	}
	r.Count = len(all)
	r.P50Ms, r.P95Ms, r.P99Ms = pct(all, .50), pct(all, .95), pct(all, .99)
	r.Rps = math.Round(float64(r.Count)/measure.Seconds()*10) / 10
	return r
}

type listResult struct {
	Users   int     `json:"users"`
	Passes  int     `json:"passes"`
	Count   int     `json:"count"`
	Errors  int     `json:"errors"`
	P50Ms   float64 `json:"p50Ms"`
	P95Ms   float64 `json:"p95Ms"`
	MaxMs   float64 `json:"maxMs"`
	Results int     `json:"results"`
}

// listLoad lists each user's documents, one request at a time, at default consistency.
func listLoad(ctx context.Context, a Arm, c *corpus) listResult {
	var lat []time.Duration
	r := listResult{Users: len(c.ListUsers), Passes: listPass}
	for p := 0; p < listPass; p++ {
		for _, u := range c.ListUsers {
			t := time.Now()
			docs, err := a.List(ctx, u, false)
			d := time.Since(t)
			if err != nil {
				r.Errors++
				continue
			}
			r.Results += len(docs)
			lat = append(lat, d)
		}
	}
	r.Count = len(lat)
	r.P50Ms, r.P95Ms, r.MaxMs = pct(lat, .50), pct(lat, .95), pct(lat, 1)
	return r
}

func readLoadavg() []float64 {
	b, err := os.ReadFile("/proc/loadavg")
	if err != nil {
		return nil
	}
	f := strings.Fields(string(b))
	out := make([]float64, 0, 3)
	for _, s := range f[:3] {
		v, _ := strconv.ParseFloat(s, 64)
		out = append(out, v)
	}
	return out
}

// rssMb is VmRSS from /proc/<pid>/status, in MB (2^20 bytes).
func rssMb(pid int) float64 {
	b, err := os.ReadFile(fmt.Sprintf("/proc/%d/status", pid))
	if err != nil {
		return -1
	}
	for _, line := range strings.Split(string(b), "\n") {
		if strings.HasPrefix(line, "VmRSS:") {
			kb, _ := strconv.ParseFloat(strings.Fields(line)[1], 64)
			return math.Round(kb/1024*10) / 10
		}
	}
	return -1
}

func cpuInfo() map[string]any {
	b, _ := os.ReadFile("/proc/cpuinfo")
	model, n := "", 0
	for _, line := range strings.Split(string(b), "\n") {
		if strings.HasPrefix(line, "model name") {
			n++
			model = strings.TrimSpace(strings.SplitN(line, ":", 2)[1])
		}
	}
	return map[string]any{"model": model, "count": n}
}

type server struct {
	cmd  *exec.Cmd
	done chan error
}

func startServer(argv []string, logPath string) (*server, time.Time, error) {
	if len(argv) == 0 {
		return nil, time.Time{}, fmt.Errorf("no server command after --")
	}
	logf, err := os.Create(logPath)
	if err != nil {
		return nil, time.Time{}, err
	}
	cmd := exec.Command(argv[0], argv[1:]...)
	cmd.Stdout, cmd.Stderr = logf, logf
	cmd.SysProcAttr = &syscall.SysProcAttr{Setpgid: true}
	t0 := time.Now()
	if err := cmd.Start(); err != nil {
		return nil, t0, err
	}
	s := &server{cmd: cmd, done: make(chan error, 1)}
	go func() { s.done <- cmd.Wait(); logf.Close() }()
	return s, t0, nil
}

func (s *server) stop() {
	_ = syscall.Kill(-s.cmd.Process.Pid, syscall.SIGTERM)
	select {
	case <-s.done:
	case <-time.After(15 * time.Second):
		_ = syscall.Kill(-s.cmd.Process.Pid, syscall.SIGKILL)
		<-s.done
	}
}

// waitReady retries Setup, then a check, every 10 ms until a check succeeds; it returns the time
// since exec.
func waitReady(ctx context.Context, a Arm, s *server, t0 time.Time) (float64, error) {
	deadline := t0.Add(120 * time.Second)
	setup := false
	for time.Now().Before(deadline) {
		select {
		case err := <-s.done:
			return 0, fmt.Errorf("server exited before ready: %v", err)
		default:
		}
		cctx, cancel := context.WithTimeout(ctx, time.Second)
		if !setup {
			setup = a.Setup(cctx, true) == nil
		}
		if setup {
			if _, err := a.Check(cctx, "u0", "d0", false); err == nil {
				cancel()
				return round3(time.Since(t0).Seconds()), nil
			}
		}
		cancel()
		time.Sleep(10 * time.Millisecond)
	}
	return 0, fmt.Errorf("not ready after 120 s")
}

func emit(v any) {
	enc := json.NewEncoder(os.Stdout)
	enc.SetIndent("", "  ")
	_ = enc.Encode(v)
}

func fail(err error) {
	fmt.Fprintln(os.Stderr, "loadgen:", err)
	os.Exit(1)
}

func splitArgs(args []string) ([]string, []string) {
	for i, a := range args {
		if a == "--" {
			return args[:i], args[i+1:]
		}
	}
	return args, nil
}

func main() {
	if len(os.Args) < 2 {
		fail(fmt.Errorf("usage: loadgen round|write|equivalence|checkload|listload|probe|traceprobe [flags] [-- server command]"))
	}
	sub := os.Args[1]
	flagArgs, serverArgv := splitArgs(os.Args[2:])
	fs := flag.NewFlagSet(sub, flag.ExitOnError)
	var af armFlags
	fs.StringVar(&af.arm, "arm", "", "openfga or spicedb")
	fs.StringVar(&af.addr, "addr", "", "gRPC address")
	fs.StringVar(&af.key, "key", "", "SpiceDB preshared key")
	fs.StringVar(&af.dir, "dir", "..", "experiment directory (corpus.json, model.fga, schema.zed, probes/)")
	roundN := fs.Int("round", 0, "round number")
	order := fs.Int("order", 0, "position of this arm in the round")
	out := fs.String("out", "", "write the JSON here as well as to stdout")
	logPath := fs.String("log", "server.log", "server log")
	httpURL := fs.String("http", "", "probe: URL of the default HTTP API")
	metricsURL := fs.String("metrics", "", "probe: URL of the default metrics endpoint")
	otlp := fs.String("otlp", "", "traceprobe: address to listen on for OTLP/gRPC")
	_ = fs.Parse(flagArgs)
	ctx := context.Background()

	if sub == "traceprobe" {
		emit(traceProbe(ctx, af, *otlp, serverArgv, *logPath))
		return
	}
	a, err := newArm(af)
	if err != nil {
		fail(err)
	}
	defer a.Close()
	if sub == "probe" {
		emit(probe(ctx, af, a, serverArgv, *logPath, *httpURL, *metricsURL))
		return
	}
	c, err := loadCorpus(af.dir)
	if err != nil {
		fail(err)
	}
	var result any
	switch sub {
	case "round":
		result, err = runRound(ctx, af, a, c, *roundN, *order, serverArgv, *logPath)
	case "write", "equivalence", "checkload", "listload":
		if err = a.Setup(ctx, sub == "write"); err != nil {
			break
		}
		switch sub {
		case "write":
			result, err = writeCorpus(ctx, a, c)
		case "equivalence":
			result, err = equivalence(ctx, a, c)
		case "checkload":
			result = checkLoad(ctx, a, c)
		case "listload":
			result = listLoad(ctx, a, c)
		}
	default:
		err = fmt.Errorf("unknown subcommand %q", sub)
	}
	if result != nil {
		emit(result)
		if *out != "" {
			b, _ := json.MarshalIndent(result, "", "  ")
			if werr := os.WriteFile(*out, append(b, '\n'), 0o644); werr != nil {
				fail(werr)
			}
		}
	}
	if err != nil {
		fail(err)
	}
}

type roundResult struct {
	Arm             string            `json:"arm"`
	Round           int               `json:"round"`
	Order           int               `json:"order"`
	Server          []string          `json:"server"`
	StartedAt       string            `json:"startedAt"`
	CPU             map[string]any    `json:"cpu"`
	LoadavgStart    []float64         `json:"loadavgStart"`
	LoadavgEnd      []float64         `json:"loadavgEnd"`
	ReadyS          float64           `json:"readyS"`
	RssIdleMb       float64           `json:"rssIdleMb"`
	Write           writeResult       `json:"write"`
	Equivalence     equivalenceResult `json:"equivalence"`
	Check           checkResult       `json:"check"`
	RssAfterCheckMb float64           `json:"rssAfterCheckMb"`
	List            listResult        `json:"list"`
	RssLoadedMb     float64           `json:"rssLoadedMb"`
	Error           string            `json:"error,omitempty"`
}

// runRound is one timed round: exec, ready, write, equivalence, check load, list latency, stop.
func runRound(ctx context.Context, af armFlags, a Arm, c *corpus, n, order int, argv []string, logPath string) (*roundResult, error) {
	r := &roundResult{Arm: af.arm, Round: n, Order: order, Server: argv, CPU: cpuInfo(),
		StartedAt: time.Now().UTC().Format(time.RFC3339), LoadavgStart: readLoadavg()}
	s, t0, err := startServer(argv, logPath)
	if err != nil {
		return nil, err
	}
	defer s.stop()
	if r.ReadyS, err = waitReady(ctx, a, s, t0); err != nil {
		return nil, err
	}
	pid := s.cmd.Process.Pid
	r.RssIdleMb = rssMb(pid)
	if r.Write, err = writeCorpus(ctx, a, c); err != nil {
		return nil, err
	}
	if r.Equivalence, err = equivalence(ctx, a, c); err != nil {
		return nil, err
	}
	r.Check = checkLoad(ctx, a, c)
	r.RssAfterCheckMb = rssMb(pid)
	r.List = listLoad(ctx, a, c)
	r.RssLoadedMb = rssMb(pid)
	r.LoadavgEnd = readLoadavg()
	return r, nil
}
