//! Workflow run statistics (docs/design/github-actions-standards.md §5): for each workflow, the recent
//! runs on one branch reduced to the reliability figures GitHub's Actions performance metrics report
//! (failure rate, run time) plus reruns and the current failure streak.

use serde_json::{Value, json};

/// Conclusions that count as failures, and those that count at all (cancelled and skipped runs say
/// nothing about whether the workflow works).
const FAILED: &[&str] = &["failure", "timed_out", "startup_failure"];
const DECIDED: &[&str] = &["success", "failure", "timed_out", "startup_failure"];

fn seconds(run: &Value) -> Option<i64> {
    let parse = |k: &str| run[k].as_str().and_then(parse_rfc3339);
    Some(parse("updated_at")? - parse("run_started_at")?)
}

/// Seconds since the epoch for `YYYY-MM-DDTHH:MM:SSZ` (what the API returns).
fn parse_rfc3339(s: &str) -> Option<i64> {
    let b = s.as_bytes();
    if b.len() < 20 || b[4] != b'-' || b[10] != b'T' {
        return None;
    }
    let n = |r: std::ops::Range<usize>| s.get(r)?.parse::<i64>().ok();
    let (y, m, d) = (n(0..4)?, n(5..7)?, n(8..10)?);
    let (hh, mm, ss) = (n(11..13)?, n(14..16)?, n(17..19)?);
    // Days from civil date (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3_600 + mm * 60 + ss)
}

fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

fn percentile(sorted: &[i64], p: usize) -> Option<i64> {
    // Nearest rank, rounding up: the p90 of three runs is the slowest.
    (!sorted.is_empty()).then(|| sorted[((sorted.len() - 1) * p).div_ceil(100)])
}

/// Statistics over `runs` (newest first, as the API lists them). Percentages are whole numbers;
/// `failRate` is `null` when no run has succeeded or failed yet.
pub fn stats(runs: &[Value]) -> Value {
    let completed: Vec<&Value> = runs.iter().filter(|r| r["status"] == "completed").collect();
    let conclusion = |r: &&Value| r["conclusion"].as_str().unwrap_or("").to_owned();
    let decided: Vec<&Value> = completed
        .iter()
        .copied()
        .filter(|r| DECIDED.contains(&conclusion(r).as_str()))
        .collect();
    let failed = decided
        .iter()
        .filter(|r| FAILED.contains(&conclusion(r).as_str()))
        .count();
    let reruns = completed
        .iter()
        .filter(|r| r["run_attempt"].as_u64().unwrap_or(1) > 1)
        .count();
    // A run that passed on a later attempt failed on its first: counting it keeps a re-run from
    // hiding a failure from the fail rate.
    let passed_on_retry = decided
        .iter()
        .filter(|r| !FAILED.contains(&conclusion(r).as_str()))
        .filter(|r| r["run_attempt"].as_u64().unwrap_or(1) > 1)
        .count();
    let streak = decided
        .iter()
        .take_while(|r| FAILED.contains(&conclusion(r).as_str()))
        .count();
    let mut durations: Vec<i64> = completed.iter().filter_map(|r| seconds(r)).collect();
    durations.sort_unstable();
    let pct = |n: usize, of: usize| (of > 0).then(|| (n * 100 + of / 2) / of);
    json!({
        "runs": runs.len(),
        "completed": completed.len(),
        "decided": decided.len(),
        "failed": failed,
        // Shown beside the fail rate, which leaves them out: cancelling a run that is about to fail
        // would otherwise lower the rate unnoticed.
        "cancelled": completed.iter().filter(|r| conclusion(r) == "cancelled").count(),
        "failRate": pct(failed, decided.len()),
        "firstTryFailRate": pct(failed + passed_on_retry, decided.len()),
        // Where to start: the newest runs that failed on their first attempt (the fail rate's
        // numerator), so the evidence names runs a reader can open. A run that passed on a re-run
        // links to its failed first attempt.
        "failedRuns": decided
            .iter()
            .filter_map(|r| {
                let url = r["html_url"].as_str()?;
                if FAILED.contains(&conclusion(r).as_str()) {
                    Some(url.to_owned())
                } else if r["run_attempt"].as_u64().unwrap_or(1) > 1 {
                    Some(format!("{url}/attempts/1"))
                } else {
                    None
                }
            })
            .take(5)
            .collect::<Vec<_>>(),
        "rerunRate": pct(reruns, completed.len()),
        "failingStreak": streak,
        "lastConclusion": decided.first().map(conclusion),
        "lastRunAt": runs.first().and_then(|r| r["created_at"].as_str()),
        // Whole days from the newest run to when the history was read: run history is live data,
        // read at refresh, so this goes stale with it.
        "daysSinceLastRun": runs
            .first()
            .and_then(|r| r["created_at"].as_str())
            .and_then(parse_rfc3339)
            .map(|t| (now() - t).max(0) / 86_400),
        "oldestRunAt": runs.last().and_then(|r| r["created_at"].as_str()),
        "medianSeconds": percentile(&durations, 50),
        "p90Seconds": percentile(&durations, 90),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(conclusion: &str, attempt: u64, start: &str, end: &str) -> Value {
        json!({"status": "completed", "conclusion": conclusion, "run_attempt": attempt,
               "created_at": start, "run_started_at": start, "updated_at": end})
    }

    #[test]
    fn dates_parse() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(
            parse_rfc3339("2026-11-20T07:27:38Z").map(|s| s % 86_400),
            Some(7 * 3600 + 27 * 60 + 38)
        );
    }

    #[test]
    fn this_repositorys_ci_history() {
        // loganvolkers/toolbox ci.yml on main, runs 12 → 1 as the API listed them on 2026-11-20:
        // eleven failures, then the first run (M0), which succeeded.
        let mut runs: Vec<Value> = (0..11)
            .map(|i| {
                run(
                    "failure",
                    1,
                    &format!("2026-11-20T07:{:02}:00Z", 30 - i),
                    &format!("2026-11-20T07:{:02}:00Z", 40 - i),
                )
            })
            .collect();
        runs.push(run(
            "success",
            1,
            "2026-11-20T03:16:52Z",
            "2026-11-20T03:18:51Z",
        ));
        let s = stats(&runs);
        assert_eq!(
            (s["failRate"].as_u64(), s["failingStreak"].as_u64()),
            (Some(92), Some(11))
        );
        assert_eq!(s["lastConclusion"], "failure");
        assert_eq!(s["medianSeconds"], 600);
        assert_eq!(s["rerunRate"], 0);
        assert_eq!(
            s["failedRuns"],
            json!([]),
            "these fixtures carry no html_url"
        );
        let mut failed = run("failure", 1, "2026-01-01T00:00:00Z", "2026-01-01T00:01:00Z");
        failed["html_url"] = json!("https://github.com/o/r/actions/runs/1");
        assert_eq!(
            stats(&[failed])["failedRuns"],
            json!(["https://github.com/o/r/actions/runs/1"])
        );
    }

    #[test]
    fn cancelled_runs_and_reruns() {
        let runs = vec![
            run(
                "cancelled",
                1,
                "2026-01-01T00:00:00Z",
                "2026-01-01T00:01:00Z",
            ),
            run("success", 2, "2026-01-01T00:00:00Z", "2026-01-01T00:02:00Z"),
            json!({"status": "in_progress"}),
        ];
        let s = stats(&runs);
        assert_eq!(
            (
                s["failRate"].as_u64(),
                s["decided"].as_u64(),
                s["rerunRate"].as_u64()
            ),
            (Some(0), Some(1), Some(50))
        );
        assert_eq!(s["failingStreak"], 0);
        assert_eq!(
            s["firstTryFailRate"], 100,
            "the only decided run passed on its second attempt"
        );
        let mut retried = run("success", 2, "2026-01-01T00:00:00Z", "2026-01-01T00:02:00Z");
        retried["html_url"] = json!("https://github.com/o/r/actions/runs/7");
        let s = stats(&[retried]);
        assert_eq!(
            s["failedRuns"],
            json!(["https://github.com/o/r/actions/runs/7/attempts/1"])
        );
        assert!(s["daysSinceLastRun"].as_i64().unwrap() > 300);
        assert!(stats(&[])["failRate"].is_null());
    }
}
